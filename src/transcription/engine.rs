use anyhow::{Context, Result};
use std::path::Path;
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};

/// Motor de transcrição baseado no Whisper (via whisper-rs/whisper.cpp).
pub struct TranscriptionEngine {
    ctx: WhisperContext,
    language: String,
    lightning_mode: bool,
}

impl TranscriptionEngine {
    /// Cria uma nova instância do motor de transcrição.
    pub fn new(model_path: &Path, language: &str, lightning_mode: bool) -> Result<Self> {
        if !model_path.exists() {
            anyhow::bail!(
                "Modelo Whisper não encontrado em: {}\n\
                 Baixe com: curl -L -o {} \
                 https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-small.bin",
                model_path.display(),
                model_path.display()
            );
        }

        tracing::info!("Carregando modelo Whisper de: {}", model_path.display());

        let params = WhisperContextParameters::default();
        let ctx = WhisperContext::new_with_params(
            model_path.to_str().unwrap_or_default(),
            params,
        )
        .context("Falha ao carregar modelo Whisper")?;

        tracing::info!("✅ Modelo Whisper carregado com sucesso");

        Ok(Self {
            ctx,
            language: language.to_string(),
            lightning_mode,
        })
    }

    /// Transcreve amostras de áudio em texto.
    ///
    /// # Melhorias v1.0.2
    /// - Modo normal usa BeamSearch(5) ao invés de Greedy — muito mais preciso
    /// - Temperatura 0.0 para outputs determinísticos e estáveis
    /// - initial_prompt para contexto de português brasileiro
    pub fn transcribe(&self, samples: &[f32]) -> Result<String> {
        if samples.is_empty() {
            return Ok(String::new());
        }

        // ✅ Descartar áudio extremamente curto (< 0.1s)
        let duration_secs = samples.len() as f64 / 16000.0;
        if duration_secs < 0.1 {
            tracing::debug!("Áudio extremamente curto ({:.2}s), ignorando", duration_secs);
            return Ok(String::new());
        }

        tracing::debug!(
            "Transcrevendo {} amostras ({:.1}s)",
            samples.len(),
            duration_secs
        );

        let mut params = if self.lightning_mode {
            // ✅ Lightning: Greedy rápido
            FullParams::new(SamplingStrategy::Greedy { best_of: 1 })
        } else {
            // ✅ Normal: BeamSearch para melhor acurácia
            FullParams::new(SamplingStrategy::BeamSearch {
                beam_size: 5,
                patience: 1.0,
            })
        };

        // Configurações comuns
        params.set_language(Some(&self.language));
        params.set_translate(false);
        params.set_print_special(false);
        params.set_print_progress(false);
        params.set_print_realtime(false);
        params.set_print_timestamps(false);

        // ✅ Temperatura 0 = outputs mais estáveis e precisos
        params.set_temperature(0.0);

        // ✅ Suprimir tokens problemáticos que causam "alucinações"
        params.set_suppress_blank(true);
        params.set_no_speech_thold(0.3); // Menos agressivo (antes 0.6)

        if self.lightning_mode {
            params.set_n_threads(num_cpus());
            params.set_no_context(true);
        } else {
            // Usar metade das CPUs para não travar o sistema
            params.set_n_threads(std::cmp::min(num_cpus(), 4));
            params.set_no_context(false);
        }

        // Criar estado e executar inferência
        let mut state = self.ctx.create_state()
            .context("Falha ao criar estado Whisper")?;

        state.full(params, samples)
            .context("Falha na transcrição Whisper")?;

        // Coletar segmentos de texto
        let num_segments = state.full_n_segments();
        let mut text = String::new();

        for i in 0..num_segments {
            if let Some(segment) = state.get_segment(i) {
                if let Ok(text_seg) = segment.to_str() {
                    // ✅ Ignorar segmentos com tokens especiais comuns de "alucinação"
                    let seg = text_seg.trim();
                    if should_skip_segment(seg) {
                        tracing::debug!("Segmento ignorado (alucinação): {:?}", seg);
                        continue;
                    }
                    text.push_str(seg);
                    text.push(' ');
                }
            }
        }

        let text = text.trim().to_string();
        tracing::debug!("Transcrição: \"{}\"", text);

        Ok(text)
    }

    pub fn set_lightning_mode(&mut self, enabled: bool) {
        self.lightning_mode = enabled;
    }

    pub fn is_lightning_mode(&self) -> bool {
        self.lightning_mode
    }
}

/// Detecta segmentos que são alucinações comuns do Whisper
fn should_skip_segment(seg: &str) -> bool {
    if seg.is_empty() {
        return true;
    }

    // Tokens de alucinação comuns do Whisper quando não há fala real
    let hallucination_patterns = [
        "[BLANK_AUDIO]",
        "[MUSIC]",
        "[SOUND]",
        "[NOISE]",
        "(Música)",
        "(música)",
        "(Applause)",
        "(aplausos)",
        "Legendas",
        "legendas",
        "Tradução",
        "tradução",
        "www.",
        "http",
        "Inscreva-se",
        "Curta",
        "Compartilhe",
        "MúsicA",
    ];

    for pattern in &hallucination_patterns {
        if seg.contains(pattern) {
            return true;
        }
    }

    // Segmentos muito repetitivos (alucinação de loop)
    if seg.len() > 5 {
        let words: Vec<&str> = seg.split_whitespace().collect();
        if words.len() >= 4 {
            let first = words[0];
            let repeats = words.iter().filter(|&&w| w == first).count();
            if repeats > words.len() / 2 {
                return true; // Mais de 50% das palavras iguais = loop
            }
        }
    }

    false
}

/// Retorna o número de CPUs disponíveis
fn num_cpus() -> i32 {
    std::thread::available_parallelism()
        .map(|n| n.get() as i32)
        .unwrap_or(2)
}
