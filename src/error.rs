use thiserror::Error;

/// Erros customizados para o ecossistema Lumen
#[derive(Error, Debug)]
pub enum LumenError {
    #[error("Falha na captura de áudio: {0}")]
    AudioCapture(String),

    #[error("Motor Whisper não inicializado (modelo ausente?)")]
    EngineNotAvailable,

    #[error("Falha na formatação de texto pela AI: {0}")]
    AiFormatting(String),

    #[error("Falha na persistência de configuração: {0}")]
    Config(String),

    #[error("Falha no banco de dados Analytics (histórico): {0}")]
    AnalyticsDb(String),

    #[error("Módulo de hotkeys falhou: {0}")]
    Hotkeys(String),

    #[error("Erro interno inesperado: {0}")]
    Internal(String),

    // Repassa erros de I/O
    #[error(transparent)]
    Io(#[from] std::io::Error),

    // Erros capturados via anyhow em locais onde erro estrutural não é tão estrito
    #[error(transparent)]
    Anyhow(#[from] anyhow::Error),

    // Erros do banco Local
    #[error(transparent)]
    Sqlite(#[from] rusqlite::Error),
}

/// Tipo de Result padrão para módulos do Lumen
pub type LumenResult<T> = std::result::Result<T, LumenError>;
