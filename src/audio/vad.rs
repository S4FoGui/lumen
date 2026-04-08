use std::time::{Duration, Instant};

/// Estado do VAD após processar um chunk de áudio.
#[derive(Debug, Clone, PartialEq)]
pub enum VadState {
    /// Voz detectada no chunk atual (nível RMS reportado).
    Speaking { rms: f32 },
    /// Silêncio, mas ainda dentro do threshold de tolerância.
    Silence { rms: f32 },
    /// Silêncio prolongado — o usuário parou de falar.
    /// Este é o trigger para parar a gravação e transcrever.
    SpeechEnded,
}

/// Detector de atividade de voz baseado em energia (RMS).
///
/// Monitora chunks de áudio em tempo real e determina quando
/// o usuário parou de falar, baseado em um threshold dinâmico e adaptativo.
///
/// Fluxo:
/// 1. Chunks com RMS > threshold dinâmico → `Speaking`
/// 2. Chunks de silêncio calibram o nível de ruído ambiente (`noise_floor`)
/// 3. Silêncio por mais de `silence_duration_ms` → `SpeechEnded`
pub struct VoiceActivityDetector {
    /// Duração de silêncio contínuo para considerar fim de fala
    silence_duration: Duration,
    /// Timestamp do último sample com voz detectada
    last_voice_time: Option<Instant>,
    /// Se o VAD detectou voz pelo menos uma vez nesta sessão
    has_detected_voice: bool,
    /// RMS suavizado (exponential moving average)
    smoothed_rms: f32,
    /// Fator de suavização principal para o áudio
    smoothing_factor: f32,
    /// Piso de ruído adaptável (Noise floor)
    noise_floor: f32,
    /// Fator de suavização lenta para calibragem do piso de ruído
    noise_alpha: f32,
    /// Valor base original do limiar, para resets
    initial_threshold: f32,
}

impl VoiceActivityDetector {
    /// Cria um novo VAD adaptativo.
    pub fn new(silence_threshold: f32, silence_duration_ms: u64) -> Self {
        Self {
            silence_duration: Duration::from_millis(silence_duration_ms),
            last_voice_time: None,
            has_detected_voice: false,
            smoothed_rms: 0.0,
            smoothing_factor: 0.1, // Reduzido de 0.3 para ser mais lento e estável
            noise_floor: silence_threshold, // inicializa com o mínimo configurado
            noise_alpha: 0.02,              // Reduzido de 0.05 para calibragem mais estável
            initial_threshold: silence_threshold,
        }
    }

    /// Processa um chunk de amostras de áudio e retorna o estado do VAD.
    pub fn process(&mut self, samples: &[f32]) -> VadState {
        if samples.is_empty() {
            return VadState::Silence { rms: 0.0 };
        }

        let rms = calculate_rms(samples);

        // 1. Suavizar RMS com EMA 
        self.smoothed_rms = self.smoothing_factor * rms
            + (1.0 - self.smoothing_factor) * self.smoothed_rms;

        // 2. Atualizar o piso de ruído adaptativo (somente se estiver perto ou abaixo do piso atual)
        // Isso impede que a fala contamine a percepção de silêncio
        if self.smoothed_rms < self.noise_floor * 2.0 {
            self.noise_floor = self.noise_alpha * rms
                + (1.0 - self.noise_alpha) * self.noise_floor;
            
            // Define limite inferior de segurança
            if self.noise_floor < 0.001 {
                self.noise_floor = 0.001;
            }
        }

        // 3. O limiar para detectar fala passa a ser dinâmico (por ex: 3x o piso de ruído)
        // E no mínimo igual ao initial_threshold pra evitar sensibilidade extrema em hiper silêncio.
        let dynamic_threshold = (self.noise_floor * 3.0).max(self.initial_threshold);

        if self.smoothed_rms > dynamic_threshold {
            // Voz detectada
            self.last_voice_time = Some(Instant::now());
            self.has_detected_voice = true;
            VadState::Speaking { rms: self.smoothed_rms }
        } else if self.has_detected_voice {
            // Já detectou voz antes — verificar timeout de silêncio
            if let Some(last_voice) = self.last_voice_time {
                if last_voice.elapsed() >= self.silence_duration {
                    // Silêncio prolongado → fim de fala
                    VadState::SpeechEnded
                } else {
                    // Ainda dentro do threshold de tolerância
                    VadState::Silence { rms: self.smoothed_rms }
                }
            } else {
                VadState::Silence { rms: self.smoothed_rms }
            }
        } else {
            // Nunca detectou voz — silêncio puro
            VadState::Silence { rms: self.smoothed_rms }
        }
    }

    /// Reseta o estado do VAD para uma nova sessão de gravação.
    pub fn reset(&mut self) {
        self.last_voice_time = None;
        self.has_detected_voice = false;
        self.smoothed_rms = 0.0;
        self.noise_floor = self.initial_threshold; // Restaura o piso original
    }

    /// Retorna se o VAD já detectou voz desde o último reset.
    pub fn has_detected_voice(&self) -> bool {
        self.has_detected_voice
    }

    /// Retorna o RMS suavizado atual.
    pub fn current_rms(&self) -> f32 {
        self.smoothed_rms
    }
}

/// Calcula o RMS (Root Mean Square) de uma slice de samples.
/// Retorna um valor entre 0.0 (silêncio total) e 1.0 (volume máximo).
fn calculate_rms(samples: &[f32]) -> f32 {
    if samples.is_empty() {
        return 0.0;
    }
    let sum_squares: f64 = samples.iter().map(|&s| (s as f64) * (s as f64)).sum();
    (sum_squares / samples.len() as f64).sqrt() as f32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_silence_detection() {
        let mut vad = VoiceActivityDetector::new(0.01, 500);

        // Silêncio puro — sem voz prévia
        let silence = vec![0.0f32; 800];
        assert_eq!(vad.process(&silence), VadState::Silence { rms: 0.0 });
        assert!(!vad.has_detected_voice());
    }

    #[test]
    fn test_speech_detection() {
        let mut vad = VoiceActivityDetector::new(0.01, 500);

        // Simular voz (samples com amplitude)
        let speech: Vec<f32> = (0..800).map(|i| (i as f32 * 0.01).sin() * 0.5).collect();
        let state = vad.process(&speech);

        match state {
            VadState::Speaking { rms } => {
                assert!(rms > 0.01);
                assert!(vad.has_detected_voice());
            }
            _ => panic!("Deveria detectar voz: {:?}", state),
        }
    }

    #[test]
    fn test_speech_ended_after_silence() {
        let mut vad = VoiceActivityDetector::new(0.01, 100); // 100ms para teste rápido

        // 1. Detectar voz
        let speech: Vec<f32> = (0..800).map(|i| (i as f32 * 0.01).sin() * 0.5).collect();
        vad.process(&speech);
        assert!(vad.has_detected_voice());

        // 2. Enviar vários chunks de silêncio para o EMA-smoothed RMS decair
        let silence = vec![0.0f32; 800];
        for _ in 0..10 {
            vad.process(&silence);
        }

        // 3. Esperar mais que o threshold
        std::thread::sleep(std::time::Duration::from_millis(150));

        // 4. Enviar silêncio final → deve retornar SpeechEnded
        assert_eq!(vad.process(&silence), VadState::SpeechEnded);
    }

    #[test]
    fn test_reset() {
        let mut vad = VoiceActivityDetector::new(0.01, 500);

        let speech: Vec<f32> = (0..800).map(|i| (i as f32 * 0.01).sin() * 0.5).collect();
        vad.process(&speech);
        assert!(vad.has_detected_voice());

        vad.reset();
        assert!(!vad.has_detected_voice());
        assert_eq!(vad.current_rms(), 0.0);
    }

    #[test]
    fn test_rms_calculation() {
        // Silêncio
        assert_eq!(calculate_rms(&[0.0, 0.0, 0.0, 0.0]), 0.0);

        // Volume máximo constante
        let rms = calculate_rms(&[1.0, 1.0, 1.0, 1.0]);
        assert!((rms - 1.0).abs() < 0.001);

        // Array vazio
        assert_eq!(calculate_rms(&[]), 0.0);
    }
}
