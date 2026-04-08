use crate::error::LumenResult as Result;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::ai::commands::{CommandDetector, VoiceCommand};
use crate::analytics::Analytics;
use crate::state::{LumenEvent, LumenState, TranscriptionRecord};
use crate::text::auto_send::AutoSender;
use crate::text::injector::TextInjector;
use crate::transcription::engine::TranscriptionEngine;
use crate::transcription::filler_filter::FillerFilter;

/// Pipeline completo de processamento de transcrição.
///
/// Encapsula toda a lógica que antes estava inline no main.rs:
/// Whisper → Filler Filter → Command Detection → Dictionary → Snippets → AI → Inject → Auto-Send
pub struct TranscriptionPipeline {
    state: Arc<LumenState>,
    engine: Arc<TranscriptionEngine>,
    filler_filter: FillerFilter,
    command_detector: CommandDetector,
    text_injector: Arc<RwLock<TextInjector>>,
    auto_sender: AutoSender,
    analytics_db: Arc<Analytics>,
}

pub enum InjectionStrategy {
    Immediate,
    DeferredAfterAi,
}

impl TranscriptionPipeline {
    /// Cria um novo pipeline de transcrição.
    pub fn new(
        state: Arc<LumenState>,
        engine: Arc<TranscriptionEngine>,
        filler_filter: FillerFilter,
        text_injector: Arc<RwLock<TextInjector>>,
        analytics_db: Arc<Analytics>,
    ) -> Self {
        Self {
            state,
            engine,
            filler_filter,
            command_detector: CommandDetector::new(),
            text_injector,
            auto_sender: AutoSender::new(150),
            analytics_db,
        }
    }

    async fn determine_strategy(&self, command: &VoiceCommand) -> InjectionStrategy {
        // Se a instrução do comando de voz for transform, ou auto-formatting, joga pra AI
        if matches!(command, VoiceCommand::Transform { .. }) {
            return InjectionStrategy::DeferredAfterAi;
        }
        
        let config = self.state.config.read().await;
        if config.ai.auto_formatting {
            if self.state.ai_formatter.is_enabled() {
                InjectionStrategy::DeferredAfterAi
            } else {
                // Auto-formatting ativo mas nenhuma API configurada — notificar usuário
                self.state.emit(LumenEvent::Error {
                    message: "Auto-Improve ativo mas nenhuma API key configurada. Configure um provedor de IA nas configurações.".into(),
                });
                InjectionStrategy::Immediate
            }
        } else {
            InjectionStrategy::Immediate
        }
    }

    /// Processa samples de áudio pelo pipeline em duas fases (Rápida + Lenta/Assíncrona).
    pub async fn process(&self, samples: Vec<f32>) -> Result<TranscriptionRecord> {
        let start = std::time::Instant::now();

        // ── Fase 1: Síncrona e Ultra Rápida (Whisper -> Fitlros -> Dicionário -> Snippets) ──
        let raw_text = self.engine.transcribe(&samples)?;
        if raw_text.is_empty() {
            tracing::debug!("Transcrição vazia, ignorando pipeline");
            return Ok(TranscriptionRecord {
                id: uuid::Uuid::new_v4().to_string(),
                timestamp: chrono::Utc::now(),
                raw_text: String::new(),
                processed_text: String::new(),
                word_count: 0,
                processing_time_ms: start.elapsed().as_millis() as u64,
                ai_used: false,
                auto_sent: false,
            });
        }
        tracing::debug!("Raw transcription: \"{}\"", raw_text);

        let mut processed = self.filler_filter.filter(&raw_text);

        let voice_commands_enabled = self.state.config.read().await.transcription.voice_commands_enabled;
        let (clean_text, command) = if voice_commands_enabled {
            self.command_detector.detect(&processed)
        } else {
            (processed.clone(), VoiceCommand::None)
        };

        if command == VoiceCommand::Delete {
            tracing::info!("🗑️ Comando de voz: Delete");
            self.state.emit(LumenEvent::VoiceCommandDetected {
                command_type: "delete".into(),
                command: "apague".into(),
            });
            return Ok(TranscriptionRecord {
                id: uuid::Uuid::new_v4().to_string(),
                timestamp: chrono::Utc::now(),
                raw_text,
                processed_text: String::new(),
                word_count: 0,
                processing_time_ms: start.elapsed().as_millis() as u64,
                ai_used: false,
                auto_sent: false,
            });
        }
        processed = clean_text;

        {
            let dict = self.state.dictionary.read().await;
            processed = dict.apply(&processed);
        }
        {
            let snippets = self.state.snippets.read().await;
            processed = snippets.process(&processed);
        }

        let strategy = self.determine_strategy(&command).await;
        let word_count = processed.split_whitespace().count() as u64;
        let processing_time_ms = start.elapsed().as_millis() as u64;

        // Atualizar estatísticas síncronamente
        {
            let mut session = self.state.session.write().await;
            session.record_transcription(word_count);
        }

        // Emite evento de overlay informando o final do Whisper
        self.state.emit(LumenEvent::TranscriptionComplete {
            id: uuid::Uuid::new_v4().to_string(),
            raw: raw_text.clone(),
            processed: processed.clone(),
            words: word_count,
            processing_time_ms,
            auto_sent: false,
        });

        // ── FASE 2: Injeção ou IA Assíncrona ──
        let state_clone = Arc::clone(&self.state);
        let injector_clone = Arc::clone(&self.text_injector);
        let sender_clone = self.auto_sender.clone();
        let db_clone = Arc::clone(&self.analytics_db);
        let fast_text = processed.clone();
        let cmd = command.clone();
        let rid = uuid::Uuid::new_v4().to_string();
        let raw_for_bg = raw_text.clone();

        match strategy {
            InjectionStrategy::Immediate => {
                // Se for comando Send isolado (sem texto), não tenta injeção de string vazia.
                if !fast_text.trim().is_empty() {
                    let injector = injector_clone.read().await;
                    if let Err(e) = injector.inject(&fast_text).await {
                        tracing::error!("Falha ao injetar texto imediato: {}", e);
                    } else {
                        state_clone.emit(LumenEvent::InjectionComplete { text: fast_text.clone() });
                    }
                }

                // Dispara Background apenas para banco e auto-sender
                tokio::spawn(async move {
                    Self::handle_auto_send(&cmd, fast_text.clone(), state_clone.clone(), sender_clone).await;
                    Self::save_history(rid, raw_for_bg, fast_text, word_count, processing_time_ms, false, false, db_clone).await;
                });
            }
            InjectionStrategy::DeferredAfterAi => {
                // Notificar overlay que estamos reescrevendo com IA
                state_clone.emit(LumenEvent::AiProcessing);
                
                // Libera a thread principal, processa IA em background
                tokio::spawn(async move {
                    let mut final_text = fast_text.clone();
                    let mut ai_used = false;

                    let instruction_opt = match &cmd {
                        VoiceCommand::Transform { instruction } => {
                            state_clone.emit(LumenEvent::VoiceCommandDetected {
                                command_type: "transform".into(),
                                command: instruction.clone(),
                            });
                            Some(instruction.as_str())
                        }
                        _ => None,
                    };

                    // Implementação de Timeout Anti-Zombie + Fallback Obrigatório
                    let format_future = state_clone.ai_formatter.format_text(&fast_text, instruction_opt);
                    
                    match tokio::time::timeout(tokio::time::Duration::from_secs(15), format_future).await {
                        Ok(Ok(refined)) => {
                            final_text = refined;
                            ai_used = true;
                        }
                        Ok(Err(e)) => {
                            tracing::warn!("AI falhou internamente ({e}), injetando texto bruto como fallback");
                        }
                        Err(_) => {
                            tracing::warn!("AI deu Timeout (>15s), injetando texto bruto como fallback");
                        }
                    }

                    // Injeta a String final (Refinada ou Bruta fallback)
                    let injector = injector_clone.read().await;
                    if let Err(e) = injector.inject(&final_text).await {
                        tracing::error!("Falha ao injetar texto (Pós-IA): {}", e);
                    } else {
                        state_clone.emit(LumenEvent::InjectionComplete { text: final_text.clone() });
                    }

                    Self::handle_auto_send(&cmd, final_text.clone(), state_clone.clone(), sender_clone).await;
                    Self::save_history(rid, raw_for_bg, final_text, word_count, processing_time_ms, ai_used, false, db_clone).await;
                });
            }
        }

        // Process finaliza instanteaneamente! Overlay reflete o "fast_text" e o usuário desliza sem trava!
        Ok(TranscriptionRecord {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: chrono::Utc::now(),
            raw_text,
            processed_text: processed,
            word_count,
            processing_time_ms,
            ai_used: matches!(strategy, InjectionStrategy::DeferredAfterAi),
            auto_sent: false,
        })
    }

    async fn handle_auto_send(
        command: &VoiceCommand,
        text: String,
        state: Arc<LumenState>,
        sender: AutoSender,
    ) -> bool {
        let auto_send_config = state.config.read().await.transcription.auto_send;
        let should_send = match command {
            VoiceCommand::Send => {
                tracing::info!("📤 Comando de voz: Send");
                state.emit(LumenEvent::VoiceCommandDetected {
                    command_type: "send".into(),
                    command: "envie".into(),
                });
                true
            }
            _ => auto_send_config,
        };

        if should_send && !text.is_empty() {
            match sender.send_enter() {
                Ok(()) => {
                    tracing::info!("⏎ Enter pressionado automaticamente");
                    true
                }
                Err(e) => {
                    tracing::warn!("Falha no auto-send: {}", e);
                    false
                }
            }
        } else {
            false
        }
    }

    #[allow(clippy::too_many_arguments)]
    async fn save_history(
        id: String,
        raw_text: String,
        processed_text: String,
        word_count: u64,
        processing_time_ms: u64,
        ai_used: bool,
        auto_sent: bool,
        db: Arc<Analytics>
    ) {
        let record = TranscriptionRecord {
            id,
            timestamp: chrono::Utc::now(),
            raw_text,
            processed_text,
            word_count,
            processing_time_ms,
            ai_used,
            auto_sent,
        };

        if let Err(e) = db.save_transcription(&record) {
            tracing::warn!("Falha ao salvar histórico no SQLite: {}", e);
        }
    }
}
