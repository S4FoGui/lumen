use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Configuração principal do Lumen
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LumenConfig {
    pub audio: AudioConfig,
    pub transcription: TranscriptionConfig,
    pub hotkeys: HotkeyConfig,
    pub text_injection: TextInjectionConfig,
    pub ai: AiConfig,
    pub snippets: SnippetsConfig,
    pub dictionary: DictionaryConfig,
    pub ui: UiConfig,
    pub logging: LoggingConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioConfig {
    pub device: Option<String>,
    pub sample_rate: u32,
    pub channels: u16,
    #[serde(default = "default_true")]
    pub noise_suppression: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptionConfig {
    pub model_path: Option<String>,
    pub language: String,
    pub lightning_mode: bool,
    pub filler_words: Vec<String>,
    /// Pressionar Enter automaticamente ao terminar de falar
    #[serde(default)]
    pub auto_send: bool,
    /// Tempo de silêncio (ms) para considerar fim de fala (VAD)
    #[serde(default = "default_silence_threshold")]
    pub silence_threshold_ms: u64,
    /// Habilitar detecção de comandos de voz ("envie", "torne profissional")
    #[serde(default = "default_true")]
    pub voice_commands_enabled: bool,
    /// Modo sempre escutando (sem precisar hotkey)
    #[serde(default)]
    pub always_listening: bool,
    /// Palavra de ativação para processar no modo sempre escutando
    #[serde(default = "default_wake_word")]
    pub wake_word: String,
}

fn default_silence_threshold() -> u64 {
    1500
}

fn default_true() -> bool {
    true
}

fn default_false() -> bool {
    false
}

fn default_wake_word() -> String {
    "lumen".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotkeyConfig {
    pub toggle_recording: String,
    pub lightning_mode: String,
    pub open_dashboard: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextInjectionConfig {
    pub method: String,
    pub delay_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiConfig {
    pub provider: String,
    #[serde(default = "default_false")]
    pub auto_formatting: bool,
    #[serde(default)]
    pub ollama: OllamaConfig,
    #[serde(default)]
    pub openai: OpenAiConfig,
    #[serde(default)]
    pub gemini: GeminiConfig,
    #[serde(default)]
    pub groq: OpenAiConfig,
    #[serde(default)]
    pub omniroute: OmniRouteConfig,
    pub default_instruction: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OllamaConfig {
    pub url: String,
    pub api_key: String,
    pub model: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OpenAiConfig {
    pub api_key: String,
    pub model: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GeminiConfig {
    pub api_key: String,
    pub model: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OmniRouteConfig {
    pub url: String,
    pub api_key: String,
    pub model: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnippetsConfig {
    pub entries: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DictionaryEntryData {
    pub value: String,
    pub context: Option<String>,
    pub icon_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DictionaryConfig {
    pub entries: HashMap<String, DictionaryEntryData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiConfig {
    pub dashboard_port: u16,
    pub open_on_start: bool,
    pub show_overlay: bool,
    pub show_tray: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub file: Option<String>,
}

impl LumenConfig {
    /// Valida campos críticos da configuração.
    pub fn validate(&self) -> Result<()> {
        if self.audio.sample_rate != 16000 {
            anyhow::bail!("sample_rate deve ser 16000 para Whisper (atual: {})", self.audio.sample_rate);
        }
        if self.audio.channels != 1 {
            anyhow::bail!("channels deve ser 1 (mono) para Whisper (atual: {})", self.audio.channels);
        }
        if self.transcription.language.is_empty() {
            anyhow::bail!("language não pode ser vazio");
        }
        if self.transcription.silence_threshold_ms < 500 {
            anyhow::bail!("silence_threshold_ms deve ser >= 500ms (atual: {}ms)", self.transcription.silence_threshold_ms);
        }
        if self.transcription.always_listening && self.transcription.wake_word.trim().is_empty() {
            anyhow::bail!("wake_word não pode ser vazio quando always_listening está ativo");
        }
        if self.ui.dashboard_port == 0 {
            anyhow::bail!("dashboard_port não pode ser 0");
        }
        Ok(())
    }

    /// Carrega a configuração do arquivo YAML.
    /// Prioridade: ~/.config/lumen/config.yaml > config/default.yaml (bundled)
    pub fn load() -> Result<Self> {
        let config_path = Self::config_path();

        let config_str = if config_path.exists() {
            tracing::info!("Carregando configuração de: {}", config_path.display());
            std::fs::read_to_string(&config_path)
                .with_context(|| format!("Falha ao ler {}", config_path.display()))?
        } else {
            tracing::info!("Usando configuração padrão embutida");
            include_str!("../config/default.yaml").to_string()
        };

        let config: LumenConfig = serde_yaml::from_str(&config_str)
            .context("Falha ao fazer parse da configuração YAML")?;

        Ok(config)
    }

    /// Salva a configuração atual para o arquivo YAML do usuário
    pub fn save(&self) -> Result<()> {
        let config_path = Self::config_path();
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("Falha ao criar diretório {}", parent.display()))?;
        }
        let yaml = serde_yaml::to_string(self)
            .context("Falha ao serializar configuração para YAML")?;
        std::fs::write(&config_path, yaml)
            .with_context(|| format!("Falha ao escrever {}", config_path.display()))?;
        tracing::info!("Configuração salva em: {}", config_path.display());
        Ok(())
    }

    /// Retorna o caminho do arquivo de configuração do usuário
    pub fn config_path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("~/.config"))
            .join("lumen")
            .join("config.yaml")
    }

    /// Retorna o diretório de dados do Lumen (modelos, logs, etc.)
    pub fn data_dir() -> PathBuf {
        dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("~/.local/share"))
            .join("lumen")
    }

    /// Retorna o caminho do modelo Whisper
    pub fn model_path(&self) -> PathBuf {
        match &self.transcription.model_path {
            Some(path) => PathBuf::from(path),
            None => Self::data_dir().join("models").join("ggml-small.bin"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test 3: Config Load and Validation
    /// Loads default config, validates fields, and tests serialization roundtrip.

    #[test]
    fn test_default_config_loads_and_validates() {
        // Parse the bundled default.yaml
        let yaml_str = include_str!("../config/default.yaml");
        let config: LumenConfig = serde_yaml::from_str(yaml_str)
            .expect("Default config YAML should parse without errors");

        // Validate all fields
        assert!(config.validate().is_ok(), "Default config should pass validation");

        // Check critical fields have expected defaults
        assert_eq!(config.audio.sample_rate, 16000, "sample_rate must be 16000 for Whisper");
        assert_eq!(config.audio.channels, 1, "channels must be 1 (mono)");
        assert!(!config.transcription.language.is_empty(), "language must not be empty");
        assert!(config.transcription.silence_threshold_ms >= 500, "silence_threshold_ms must be >= 500");
        assert!(config.ui.dashboard_port > 0, "dashboard_port must be > 0");
    }

    #[test]
    fn test_config_serialization_roundtrip() {
        let yaml_str = include_str!("../config/default.yaml");
        let config: LumenConfig = serde_yaml::from_str(yaml_str)
            .expect("Failed to parse default config");

        // Serialize back to YAML
        let serialized = serde_yaml::to_string(&config)
            .expect("Failed to serialize config to YAML");

        // Parse again
        let config2: LumenConfig = serde_yaml::from_str(&serialized)
            .expect("Failed to re-parse serialized config");

        // Compare key fields
        assert_eq!(config.audio.sample_rate, config2.audio.sample_rate);
        assert_eq!(config.audio.channels, config2.audio.channels);
        assert_eq!(config.transcription.language, config2.transcription.language);
        assert_eq!(config.transcription.silence_threshold_ms, config2.transcription.silence_threshold_ms);
        assert_eq!(config.ui.dashboard_port, config2.ui.dashboard_port);
        assert_eq!(config.hotkeys.toggle_recording, config2.hotkeys.toggle_recording);
        assert_eq!(config.ai.provider, config2.ai.provider);
    }

    #[test]
    fn test_config_validation_rejects_invalid() {
        let yaml_str = include_str!("../config/default.yaml");
        let mut config: LumenConfig = serde_yaml::from_str(yaml_str).unwrap();

        // Invalid sample rate
        config.audio.sample_rate = 44100;
        assert!(config.validate().is_err(), "Should reject sample_rate != 16000");
        config.audio.sample_rate = 16000;

        // Invalid channels
        config.audio.channels = 2;
        assert!(config.validate().is_err(), "Should reject channels != 1");
        config.audio.channels = 1;

        // Invalid silence threshold
        config.transcription.silence_threshold_ms = 100;
        assert!(config.validate().is_err(), "Should reject silence_threshold_ms < 500");
        config.transcription.silence_threshold_ms = 1200;

        // Invalid dashboard port
        config.ui.dashboard_port = 0;
        assert!(config.validate().is_err(), "Should reject dashboard_port == 0");
    }
}
