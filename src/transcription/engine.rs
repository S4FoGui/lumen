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
    /// `model_path`: caminho para o arquivo .bin do modelo Whisper.
    /// `language`: código ISO 639-1 do idioma (ex: "pt", "en").
    /// `lightning_mode`: se true, pula pós-processamento para velocidade máxima.
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
    /// `samples`: vetor de amostras f32, 16kHz, mono.
    /// Retorna o texto transcrito.
    pub fn transcribe(&self, samples: &[f32]) -> Result<String> {
        if samples.is_empty() {
            return Ok(String::new());
        }

        tracing::debug!(
            "Transcrevendo {} amostras ({:.1}s)",
            samples.len(),
            samples.len() as f64 / 16000.0
        );

        let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });

        // Configurar idioma
        params.set_language(Some(&self.language));

        // Sem tradução (manter idioma original)
        params.set_translate(false);

        // Desabilitar timestamps para texto limpo
        params.set_print_special(false);
        params.set_print_progress(false);
        params.set_print_realtime(false);
        params.set_print_timestamps(false);

        if self.lightning_mode {
            // Lightning mode: velocidade máxima
            params.set_n_threads(num_cpus());
            // Sem detecção de idioma automática
            params.set_no_context(true);
        } else {
            params.set_n_threads(std::cmp::min(num_cpus(), 4));
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
                    text.push_str(text_seg);
                }
            }
        }

        let text = text.trim().to_string();
        tracing::debug!("Transcrição: \"{}\"", text);

        Ok(text)
    }

    /// Atualiza o modo lightning (requer &mut self)
    pub fn set_lightning_mode(&mut self, enabled: bool) {
        self.lightning_mode = enabled;
    }

    /// Retorna se está no modo lightning
    pub fn is_lightning_mode(&self) -> bool {
        self.lightning_mode
    }
}

/// Retorna o número de CPUs disponíveis
fn num_cpus() -> i32 {
    std::thread::available_parallelism()
        .map(|n| n.get() as i32)
        .unwrap_or(2)
}
