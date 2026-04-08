use std::time::{Duration, Instant};

/// Estado do VAD após processar um chunk de áudio.
#[derive(Debug, Clone, PartialEq)]
pub enum VadState {
    /// Voz detectada no chunk atual (nível RMS reportado).
    Speaking { rms: f32 },
    /// Silêncio, mas ainda dentro do threshold de tolerância.
    Silence { rms: f32 },
    /// Silêncio prolongado — o usuário parou de falar.
    SpeechEnded,
}

/// Detector de atividade de voz baseado em energia (RMS).
///
/// # Melhorias v1.0.2
/// - Requer duração mínima de voz (300ms) antes de considerar SpeechEnded
///   Evita que ruídos curtos disparem o pipeline e desperdicem tempo
/// - Threshold dinâmico mais rápido de calibrar (noise_alpha = 0.05)
/// - Margem maior para voz vs ruído (4x o noise_floor ao invés de 3x)
pub struct VoiceActivityDetector {
    silence_duration: Duration,
    last_voice_time: Option<Instant>,
    /// Timestamp do início da fala (para garantir duração mínima)
    speech_start_time: Option<Instant>,
    has_detected_voice: bool,
    smoothed_rms: f32,
    smoothing_factor: f32,
    noise_floor: f32,
    noise_alpha: f32,
    initial_threshold: f32,
    /// Duração mínima de fala antes de considerar válida
    min_speech_duration: Duration,
}

impl VoiceActivityDetector {
    /// Cria um novo VAD adaptativo.
    ///
    /// # Argumentos
    /// - `silence_threshold`: RMS mínimo considerado "não silêncio" (0.0–1.0)
    /// - `silence_duration_ms`: ms de silêncio contínuo para disparar SpeechEnded
    pub fn new(silence_threshold: f32, silence_duration_ms: u64) -> Self {
        Self {
            silence_duration: Duration::from_millis(silence_duration_ms),
            last_voice_time: None,
            speech_start_time: None,
            has_detected_voice: false,
            smoothed_rms: 0.0,
            smoothing_factor: 0.15, // Levemente mais rápido que antes
            noise_floor: silence_threshold,
            noise_alpha: 0.05,       // ✅ Calibra mais rápido o ruído ambiente
            initial_threshold: silence_threshold,
            // ✅ Precisa de ao menos 300ms de fala antes de considerar válido
            min_speech_duration: Duration::from_millis(300),
        }
    }

    /// Processa um chunk de amostras e retorna o estado atual.
    pub fn process(&mut self, samples: &[f32]) -> VadState {
        if samples.is_empty() {
            return VadState::Silence { rms: 0.0 };
        }

        let rms = calculate_rms(samples);

        // Suavizar RMS com EMA
        self.smoothed_rms = self.smoothing_factor * rms
            + (1.0 - self.smoothing_factor) * self.smoothed_rms;

        // Atualizar noise floor apenas quando sinal é baixo
        if self.smoothed_rms < self.noise_floor * 2.5 {
            self.noise_floor = self.noise_alpha * rms
                + (1.0 - self.noise_alpha) * self.noise_floor;
            if self.noise_floor < 0.001 {
                self.noise_floor = 0.001;
            }
        }

        // ✅ Threshold 4x o noise floor (mais discriminativo)
        let dynamic_threshold = (self.noise_floor * 4.0).max(self.initial_threshold);

        if self.smoothed_rms > dynamic_threshold {
            // Voz detectada
            let now = Instant::now();
            if self.speech_start_time.is_none() {
                self.speech_start_time = Some(now);
            }
            self.last_voice_time = Some(now);

            // Só marca como "detectou voz" após duração mínima
            if let Some(start) = self.speech_start_time {
                if start.elapsed() >= self.min_speech_duration {
                    self.has_detected_voice = true;
                }
            }

            VadState::Speaking { rms: self.smoothed_rms }
        } else if self.has_detected_voice {
            // Já detectou voz — verificar timeout de silêncio
            if let Some(last_voice) = self.last_voice_time {
                if last_voice.elapsed() >= self.silence_duration {
                    VadState::SpeechEnded
                } else {
                    VadState::Silence { rms: self.smoothed_rms }
                }
            } else {
                VadState::Silence { rms: self.smoothed_rms }
            }
        } else {
            // Nunca detectou voz suficiente
            // ✅ Reset speech_start_time se silêncio longo (evita acumulação de ruído)
            if let Some(start) = self.speech_start_time {
                if start.elapsed() > Duration::from_millis(500) {
                    self.speech_start_time = None;
                }
            }
            VadState::Silence { rms: self.smoothed_rms }
        }
    }

    /// Reseta o estado para nova sessão de gravação.
    pub fn reset(&mut self) {
        self.last_voice_time = None;
        self.speech_start_time = None;
        self.has_detected_voice = false;
        self.smoothed_rms = 0.0;
        self.noise_floor = self.initial_threshold;
    }

    #[allow(dead_code)]
    pub fn has_detected_voice(&self) -> bool {
        self.has_detected_voice
    }

    #[allow(dead_code)]
    pub fn current_rms(&self) -> f32 {
        self.smoothed_rms
    }
}

/// Calcula o RMS de uma slice de samples f32.
fn calculate_rms(samples: &[f32]) -> f32 {
    if samples.is_empty() {
        return 0.0;
    }
    let sum_sq: f64 = samples.iter().map(|&s| (s as f64) * (s as f64)).sum();
    (sum_sq / samples.len() as f64).sqrt() as f32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_silence_detection() {
        let mut vad = VoiceActivityDetector::new(0.01, 500);
        let silence = vec![0.0f32; 800];
        assert_eq!(vad.process(&silence), VadState::Silence { rms: 0.0 });
        assert!(!vad.has_detected_voice());
    }

    #[test]
    fn test_speech_detection() {
        let mut vad = VoiceActivityDetector::new(0.01, 500);
        let speech: Vec<f32> = (0..800).map(|i| (i as f32 * 0.01).sin() * 0.5).collect();
        let state = vad.process(&speech);
        match state {
            VadState::Speaking { rms } => assert!(rms > 0.01),
            _ => panic!("Deveria detectar voz"),
        }
    }

    #[test]
    fn test_speech_ended_after_silence() {
        let mut vad = VoiceActivityDetector::new(0.01, 100);

        // Simular voz por 400ms (acima do min_speech_duration de 300ms)
        let speech: Vec<f32> = (0..6400).map(|i| (i as f32 * 0.01).sin() * 0.5).collect();
        for chunk in speech.chunks(800) {
            vad.process(chunk);
        }
        assert!(vad.has_detected_voice());

        // Silêncio
        let silence = vec![0.0f32; 800];
        for _ in 0..10 {
            vad.process(&silence);
        }
        std::thread::sleep(std::time::Duration::from_millis(150));
        assert_eq!(vad.process(&silence), VadState::SpeechEnded);
    }

    #[test]
    fn test_reset() {
        let mut vad = VoiceActivityDetector::new(0.01, 500);
        let speech: Vec<f32> = (0..6400).map(|i| (i as f32 * 0.01).sin() * 0.5).collect();
        for chunk in speech.chunks(800) {
            vad.process(chunk);
        }
        vad.reset();
        assert!(!vad.has_detected_voice());
        assert_eq!(vad.current_rms(), 0.0);
    }
}
