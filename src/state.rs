use chrono::{DateTime, Utc};
use serde::Serialize;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};

use crate::ai::formatter::AiFormatter;
use crate::config::LumenConfig;
use crate::dictionary::custom::CustomDictionary;
use crate::text::snippets::SnippetManager;
use crate::analytics::Analytics;

/// Hub central de estado do Lumen.
/// Todo estado mutable compartilhado entre event loop, API web e overlay
/// vive aqui — eliminando a fragmentação de Arc<RwLock<>> soltos.
pub struct LumenState {
    // ── Config (fonte de verdade do YAML) ──
    pub config: RwLock<LumenConfig>,

    // ── Componentes vivos (operados pela API E pelo engine) ──
    pub dictionary: RwLock<CustomDictionary>,
    pub snippets: RwLock<SnippetManager>,
    pub ai_formatter: Arc<AiFormatter>,
    
    // ── Analytics DB ──
    pub db: Arc<Analytics>,

    // ── Runtime status ──
    pub is_recording: RwLock<bool>,
    pub session: RwLock<SessionStats>,

    // ── Target window (janela que deve receber o texto colado) ──
    /// Armazena o ID da janela focada antes de iniciar gravação.
    /// Usado para refocá-la antes do Ctrl+V, evitando que o overlay roube o foco.
    pub target_window_id: RwLock<Option<String>>,

    // ── Event Bus (broadcast → WebSocket, overlay, log) ──
    pub event_tx: broadcast::Sender<LumenEvent>,
}

impl LumenState {
    /// Cria um novo hub de estado a partir da configuração carregada.
    pub fn new(config: LumenConfig, db_instance: Arc<Analytics>) -> Arc<Self> {
        let (event_tx, _) = broadcast::channel(256);

        // Criar componentes vivos a partir do config
        let dictionary = CustomDictionary::new(config.dictionary.entries.clone());
        let snippets = SnippetManager::new(config.snippets.entries.clone());
        let ai_formatter = Arc::new(AiFormatter::new(
            &config.ai.provider,
            &config.ai.ollama.url,
            &config.ai.ollama.api_key,
            &config.ai.ollama.model,
            &config.ai.openai.api_key,
            &config.ai.openai.model,
            &config.ai.gemini.api_key,
            &config.ai.gemini.model,
            &config.ai.groq.api_key,
            &config.ai.groq.model,
            &config.ai.omniroute.url,
            &config.ai.omniroute.api_key,
            &config.ai.omniroute.model,
            &config.ai.default_instruction,
        ));

        Arc::new(Self {
            config: RwLock::new(config),
            dictionary: RwLock::new(dictionary),
            snippets: RwLock::new(snippets),
            ai_formatter,
            db: db_instance,
            is_recording: RwLock::new(false),
            session: RwLock::new(SessionStats::new()),
            target_window_id: RwLock::new(None),
            event_tx,
        })
    }

    /// Emite um evento no bus de broadcast.
    /// Ignora erros silenciosamente (nenhum receiver conectado = ok).
    pub fn emit(&self, event: LumenEvent) {
        let _ = self.event_tx.send(event);
    }

    /// Persiste o config atual para o YAML do usuário.
    /// Deve ser chamado após mutações via API.
    pub async fn save_config(&self) {
        let config = self.config.read().await;
        if let Err(e) = config.save() {
            tracing::error!("Falha ao salvar configuração: {}", e);
            self.emit(LumenEvent::Error {
                message: format!("Auto-save falhou: {}", e),
            });
        } else {
            tracing::debug!("Config salvo automaticamente");
        }
    }

    /// Sincroniza os componentes "vivos" com a configuração atual.
    /// Deve ser chamado após atualizar o config via Dashboard/API para evitar inconsistências.
    pub async fn sync_live_components(&self) {
        let config = self.config.read().await;
        
        // 1. Sincronizar Dicionário
        {
            let mut dict = self.dictionary.write().await;
            dict.reload(config.dictionary.entries.clone());
        }

        // 2. Sincronizar Snippets
        {
            let mut snippets = self.snippets.write().await;
            snippets.reload(config.snippets.entries.clone());
        }

        // 3. O AiFormatter é o único componente que precisa de reinicialização completa
        // se as chaves/provedores mudarem, mas como ele é Arc, mantemos o ponteira
        // e apenas atualizamos os campos internos se necessário (nesta fase, focado em dict/snippets)
        
        tracing::info!("🔄 Componentes vivos sincronizados com a nova configuração");
    }
}

// ═══════════════════════════════════════════════════════════════
// Event Bus
// ═══════════════════════════════════════════════════════════════

/// Eventos que transitam pelo bus de broadcast.
/// Consumidos pelo WebSocket, overlay, e log interno.
#[derive(Clone, Debug, Serialize)]
#[serde(tag = "type", content = "data")]
pub enum LumenEvent {
    RecordingStarted,
    RecordingStopped,

    TranscriptionComplete {
        id: String,
        raw_text: String,
        processed_text: String,
        word_count: u64,
        processing_time_ms: u64,
        ai_used: bool,
        auto_sent: bool,
    },

    VoiceCommandDetected {
        command_type: String,
        command: String,
    },

    AiProcessing,
    InjectionComplete {
        text: String,
    },

    AudioLevel {
        rms: f32,
    },

    DictionaryUpdated,
    SnippetsUpdated,
    ConfigChanged,

    Error {
        message: String,
    },
}

// ═══════════════════════════════════════════════════════════════
// Session Stats
// ═══════════════════════════════════════════════════════════════

/// Métricas de runtime da sessão atual.
pub struct SessionStats {
    pub uptime_start: DateTime<Utc>,
    pub total_transcriptions: u64,
    pub total_words: u64,
}

impl SessionStats {
    pub fn new() -> Self {
        Self {
            uptime_start: Utc::now(),
            total_transcriptions: 0,
            total_words: 0,
        }
    }

    /// Registra uma transcrição completada.
    pub fn record_transcription(&mut self, word_count: u64) {
        self.total_transcriptions += 1;
        self.total_words += word_count;
    }

    /// Retorna o tempo de uptime em segundos.
    pub fn uptime_seconds(&self) -> i64 {
        (Utc::now() - self.uptime_start).num_seconds()
    }
}

// ═══════════════════════════════════════════════════════════════
// Transcription Record (para histórico futuro — Fase 4)
// ═══════════════════════════════════════════════════════════════

/// Registro completo de uma transcrição processada.
#[derive(Clone, Debug, Serialize)]
pub struct TranscriptionRecord {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub raw_text: String,
    pub processed_text: String,
    pub word_count: u64,
    pub processing_time_ms: u64,
    pub ai_used: bool,
    pub auto_sent: bool,
}
