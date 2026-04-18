use crate::error::LumenResult as Result;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::ai::commands::{CommandDetector, VoiceCommand};
use crate::analytics::Analytics;
use crate::state::{LumenEvent, LumenState, TranscriptionRecord};
use crate::text::injector::TextInjector;
use crate::transcription::engine::TranscriptionEngine;
use crate::transcription::filler_filter::FillerFilter;

/// Pipeline completo de processamento de transcrição.
///
/// Encapsula toda a lógica que antes estava inline no main.rs:
/// Whisper → Filler Filter → Command Detection → Dictionary → Snippets → AI → Inject → Auto-Send
pub struct TranscriptionPipeline {
    state: Arc<LumenState>,
    engine_short: Option<Arc<TranscriptionEngine>>,
    engine_long: Option<Arc<TranscriptionEngine>>,
    duration_threshold_sec: f64,
    filler_filter: FillerFilter,
    command_detector: CommandDetector,
    text_injector: Arc<RwLock<TextInjector>>,
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
        engine_short: Option<Arc<TranscriptionEngine>>,
        engine_long: Option<Arc<TranscriptionEngine>>,
        filler_filter: FillerFilter,
        text_injector: Arc<RwLock<TextInjector>>,
        analytics_db: Arc<Analytics>,
        duration_threshold_sec: f64,
    ) -> Self {
        Self {
            state,
            engine_short,
            engine_long,
            duration_threshold_sec,
            filler_filter,
            command_detector: CommandDetector::new(),
            text_injector,
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

        // ── Seleção Dinâmica do Motor baseado na duração do áudio ──
        let duration_sec = samples.len() as f32 / 16000.0;
        // Lê as configurações de roteamento e wake word ao vivo
        let cfg = self.state.config.read().await;
        let threshold = cfg.transcription.duration_threshold_sec;
        let always_listening = cfg.transcription.always_listening;
        let wake_word = cfg.transcription.wake_word.clone();
        drop(cfg);

        // Em modo Always Listening, força o modelo Curto (primário) ignorando a duração
        let route_to_long = if always_listening {
            false
        } else {
            duration_sec > threshold as f32
        };

        let engine = if route_to_long {
            if let Some(e) = &self.engine_long {
                 tracing::info!("Áudio longo ({:.1}s) -> Roteando para Whisper Longo", duration_sec);
                 e
            } else if let Some(e) = &self.engine_short {
                 tracing::info!("Áudio longo ({:.1}s) -> Whisper Longo ausente, fallback para Curto", duration_sec);
                 e
            } else { unreachable!() }
        } else {
            if let Some(e) = &self.engine_short {
                 tracing::info!("Áudio curto/AlwaysListening ({:.1}s) -> Roteando para Whisper Curto", duration_sec);
                 e
            } else if let Some(e) = &self.engine_long {
                 tracing::info!("Áudio curto ({:.1}s) -> Whisper Curto ausente, fallback para Longo", duration_sec);
                 e
            } else { unreachable!() }
        };

        // ── Fase 1: Síncrona e Ultra Rápida (Whisper -> Wake Word -> Dicionário -> Snippets) ──
        let raw_text = engine.transcribe(&samples)?;
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
                    is_wake_word_only: false,
            });
        }
        tracing::debug!("Raw transcription: \"{}\"", raw_text);

        let mut processed = raw_text.clone();

        // ── Wake Word Gate ──
        if always_listening {
            let lower = processed.to_lowercase();
            let wake_lower = wake_word.to_lowercase();
            
            let mut found_idx = lower.find(&wake_lower);
            let mut actual_wake_len = wake_lower.len();
            
            // Tratamento especial para "lumen" pois o Whisper frequentemente adiciona acento ou entende errado
            if found_idx.is_none() && wake_lower == "lumen" {
                let variations = ["lúmen", "lumi", "lume", "lumens"];
                for var in variations {
                    if let Some(idx) = lower.find(var) {
                        found_idx = Some(idx);
                        actual_wake_len = var.len();
                        break;
                    }
                }
            }
            
            if let Some(idx) = found_idx {
                // Extrai apenas a parte da string APÓS a palavra de ativação
                let end_idx = idx + actual_wake_len;
                processed = processed[end_idx..].trim().to_string();
                // Limpar pontuação residual que pode ter vindo colada (ex: "Lumen, me ajude" -> ", me ajude" -> "me ajude")
                processed = processed.trim_start_matches(|c: char| c.is_ascii_punctuation()).trim().to_string();
                tracing::info!("🎧 Wake word detectada. Texto restante: '{}'", processed);
                
                if processed.is_empty() {
                    return Ok(TranscriptionRecord {
                        id: uuid::Uuid::new_v4().to_string(),
                        timestamp: chrono::Utc::now(),
                        raw_text,
                        processed_text: String::new(),
                        word_count: 0,
                        processing_time_ms: start.elapsed().as_millis() as u64,
                        ai_used: false,
                        auto_sent: false,
                        is_wake_word_only: true, // Apenas a palavra de ativação foi dita
                    });
                }
            } else {
                tracing::info!("🎧 Wake Word não detectada no áudio: '{}' — descartando silenciosamente.", raw_text);
                return Ok(TranscriptionRecord {
                    id: uuid::Uuid::new_v4().to_string(),
                    timestamp: chrono::Utc::now(),
                    raw_text,
                    processed_text: String::new(),
                    word_count: 0,
                    processing_time_ms: start.elapsed().as_millis() as u64,
                    ai_used: false,
                    auto_sent: false,
                    is_wake_word_only: false,
                });
            }
        }

        processed = self.filler_filter.filter(&processed);

        let voice_commands_enabled = self.state.config.read().await.transcription.voice_commands_enabled;
        let (clean_text, command) = if voice_commands_enabled {
            self.command_detector.detect(&processed)
        } else {
            (processed.clone(), VoiceCommand::None)
        };

        // ── Comandos de voz que executam ações (sem injetar texto) ──
        match &command {
            VoiceCommand::Delete => {
                tracing::info!("🗑️ Comando de voz: Delete (Ctrl+A → Delete)");
                self.state.emit(LumenEvent::VoiceCommandDetected {
                    command_type: "delete".into(),
                    command: "apague".into(),
                });
                // Selecionar tudo e deletar no campo ativo
                let _ = tokio::process::Command::new("xdotool")
                    .args(["key", "ctrl+a", "Delete"])
                    .output().await;
                return Ok(TranscriptionRecord {
                    id: uuid::Uuid::new_v4().to_string(),
                    timestamp: chrono::Utc::now(),
                    raw_text,
                    processed_text: String::new(),
                    word_count: 0,
                    processing_time_ms: start.elapsed().as_millis() as u64,
                    ai_used: false,
                    auto_sent: false,
                    is_wake_word_only: false,
                });
            }
            VoiceCommand::SelectAll => {
                tracing::info!("📋 Comando de voz: SelectAll (Ctrl+A)");
                self.state.emit(LumenEvent::VoiceCommandDetected {
                    command_type: "select_all".into(),
                    command: "selecionar tudo".into(),
                });
                let _ = tokio::process::Command::new("xdotool")
                    .args(["key", "ctrl+a"])
                    .output().await;
                return Ok(TranscriptionRecord {
                    id: uuid::Uuid::new_v4().to_string(),
                    timestamp: chrono::Utc::now(),
                    raw_text,
                    processed_text: String::new(),
                    word_count: 0,
                    processing_time_ms: start.elapsed().as_millis() as u64,
                    ai_used: false,
                    auto_sent: false,
                    is_wake_word_only: false,
                });
            }
            VoiceCommand::Copy => {
                tracing::info!("📋 Comando de voz: Copy (Ctrl+C)");
                self.state.emit(LumenEvent::VoiceCommandDetected {
                    command_type: "copy".into(),
                    command: "copiar".into(),
                });
                let _ = tokio::process::Command::new("xdotool")
                    .args(["key", "ctrl+c"])
                    .output().await;
                return Ok(TranscriptionRecord {
                    id: uuid::Uuid::new_v4().to_string(),
                    timestamp: chrono::Utc::now(),
                    raw_text,
                    processed_text: String::new(),
                    word_count: 0,
                    processing_time_ms: start.elapsed().as_millis() as u64,
                    ai_used: false,
                    auto_sent: false,
                    is_wake_word_only: false,
                });
            }
            VoiceCommand::Improve => {
                tracing::info!("✨ Comando de voz: Improve (Ctrl+A → Ctrl+C → IA → Ctrl+V)");
                self.state.emit(LumenEvent::VoiceCommandDetected {
                    command_type: "improve".into(),
                    command: "melhorar".into(),
                });
                // 1. Selecionar tudo + Copiar
                let _ = tokio::process::Command::new("xdotool")
                    .args(["key", "ctrl+a"])
                    .output().await;
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                let _ = tokio::process::Command::new("xdotool")
                    .args(["key", "ctrl+c"])
                    .output().await;
                tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

                // 2. Ler clipboard
                let clipboard_text = tokio::process::Command::new("xclip")
                    .args(["-selection", "clipboard", "-o"])
                    .output().await
                    .ok()
                    .and_then(|o| if o.status.success() {
                        String::from_utf8(o.stdout).ok()
                    } else { None })
                    .unwrap_or_default();

                if clipboard_text.trim().is_empty() {
                    tracing::warn!("Clipboard vazio, nada para melhorar");
                    return Ok(TranscriptionRecord {
                        id: uuid::Uuid::new_v4().to_string(),
                        timestamp: chrono::Utc::now(),
                        raw_text,
                        processed_text: String::new(),
                        word_count: 0,
                        processing_time_ms: start.elapsed().as_millis() as u64,
                        ai_used: false,
                        auto_sent: false,
                    is_wake_word_only: false,
                    });
                }

                // 3. Enviar para IA
                self.state.emit(LumenEvent::AiProcessing);
                let improved = {
                    let cfg = self.state.config.read().await;
                    let instruction = if !cfg.ai.active_prompt_name.is_empty() {
                        cfg.ai.custom_prompts.get(&cfg.ai.active_prompt_name).cloned()
                    } else {
                        None
                    };
                    drop(cfg);
                    match tokio::time::timeout(
                        tokio::time::Duration::from_secs(15),
                        self.state.ai_formatter.format_text(&clipboard_text, instruction.as_deref()),
                    ).await {
                        Ok(Ok(refined)) => refined,
                        Ok(Err(e)) => {
                            tracing::warn!("IA falhou ao melhorar: {e}");
                            clipboard_text.clone()
                        }
                        Err(_) => {
                            tracing::warn!("IA timeout ao melhorar");
                            clipboard_text.clone()
                        }
                    }
                };

                // 4. Colar nova versão via clipboard
                let mut child = tokio::process::Command::new("xclip")
                    .args(["-selection", "clipboard"])
                    .stdin(std::process::Stdio::piped())
                    .spawn()
                    .ok();
                if let Some(ref mut c) = child {
                    use tokio::io::AsyncWriteExt;
                    if let Some(ref mut stdin) = c.stdin {
                        let _ = stdin.write_all(improved.as_bytes()).await;
                    }
                    let _ = c.wait().await;
                }
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                let _ = tokio::process::Command::new("xdotool")
                    .args(["key", "ctrl+v"])
                    .output().await;

                tracing::info!("✅ Texto melhorado e colado");
                return Ok(TranscriptionRecord {
                    id: uuid::Uuid::new_v4().to_string(),
                    timestamp: chrono::Utc::now(),
                    raw_text,
                    processed_text: improved,
                    word_count: 0,
                    processing_time_ms: start.elapsed().as_millis() as u64,
                    ai_used: true,
                    auto_sent: false,
                    is_wake_word_only: false,
                });
            }
            _ => {} // Send, Transform, NewLine, None → fluxo normal abaixo
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

        // ✅ Verificar se o texto processado ficou vazio (ex: só continha [MUSIC], fillers, etc)
        if processed.trim().is_empty() {
            tracing::debug!("Texto processado ficou vazio após filtros, ignorando pipeline");
            return Ok(TranscriptionRecord {
                id: uuid::Uuid::new_v4().to_string(),
                timestamp: chrono::Utc::now(),
                raw_text,
                processed_text: String::new(),
                word_count: 0,
                processing_time_ms: start.elapsed().as_millis() as u64,
                ai_used: false,
                auto_sent: false,
                    is_wake_word_only: false,
            });
        }

        // ── FASE 2: Injeção Imediata (Antes da atualização visual para proteger o foco) ──
        let strategy = self.determine_strategy(&command).await;
        let fast_text = processed.clone();
        let cmd = command.clone();
        let rid = uuid::Uuid::new_v4().to_string();
        let raw_for_bg = raw_text.clone();

        // ✅ Buffer de segurança para garantir que eventos físicos (Enter) terminaram
        tokio::time::sleep(tokio::time::Duration::from_millis(300)).await;

        let word_count = processed.split_whitespace().count() as u64;
        let processing_time_ms = start.elapsed().as_millis() as u64;
        let ai_used = matches!(strategy, InjectionStrategy::DeferredAfterAi);

        if processing_time_ms > 15000 {
            tracing::warn!("⚠️ Transcrição lenta ({}s). Se estiver sem GPU, considere mudar o model_path para 'small' ou 'base' no config.yaml para resposta instantânea.", processing_time_ms / 1000);
        }

        // Atualizar estatísticas síncronamente
        {
            let mut session = self.state.session.write().await;
            session.record_transcription(word_count);
        }

        if let InjectionStrategy::Immediate = strategy {
            if !fast_text.trim().is_empty() {
                let injector = self.text_injector.read().await;
                let target_win = self.state.target_window_id.read().await.clone();
                if let Err(e) = injector.inject_with_refocus(&fast_text, target_win.as_deref()).await {
                    tracing::error!("Falha ao injetar texto imediato: {}", e);
                } else {
                    self.state.emit(LumenEvent::InjectionComplete { text: fast_text.clone() });
                }
            }
        }

        // Emite evento de overlay informando o final do Whisper (Agora APÓS a injeção imediata)
        self.state.emit(LumenEvent::TranscriptionComplete {
            id: rid.clone(),
            raw_text: raw_text.clone(),
            processed_text: processed.clone(),
            word_count,
            processing_time_ms,
            ai_used,
            auto_sent: false,
        });

        // ── FASE 3: IA Assíncrona ou Background Tasks ──
        let state_clone = Arc::clone(&self.state);
        let injector_clone = Arc::clone(&self.text_injector);
        let db_clone = Arc::clone(&self.analytics_db);
        
        match strategy {
            InjectionStrategy::Immediate => {
                tokio::spawn(async move {
                    Self::handle_auto_send(&cmd, fast_text.clone(), state_clone.clone(), injector_clone).await;
                    Self::save_history(rid, raw_for_bg, fast_text, word_count, processing_time_ms, false, false, db_clone).await;
                });
            }
            InjectionStrategy::DeferredAfterAi => {
                // Notificar overlay que estamos reescrevendo com IA
                state_clone.emit(LumenEvent::AiProcessing);
                
                tokio::spawn(async move {
                    let mut final_text = fast_text.clone();
                    let mut ai_used = false;

                    let instruction_opt = match &cmd {
                        VoiceCommand::Transform { instruction } => {
                            state_clone.emit(LumenEvent::VoiceCommandDetected {
                                command_type: "transform".into(),
                                command: instruction.clone(),
                            });
                            Some(instruction.clone())
                        }
                        _ => None,
                    };

                    // Se não veio instrução de comando de voz, verifica prompt customizado ativo
                    let active_instruction = if instruction_opt.is_some() {
                        instruction_opt
                    } else {
                        let cfg = state_clone.config.read().await;
                        let name = &cfg.ai.active_prompt_name;
                        if !name.is_empty() {
                            cfg.ai.custom_prompts.get(name).cloned()
                        } else {
                            None
                        }
                    };

                    let format_future = state_clone.ai_formatter.format_text(&fast_text, active_instruction.as_deref());
                    
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

                    {
                        let injector = injector_clone.read().await;
                        let target_win = state_clone.target_window_id.read().await.clone();
                        if let Err(e) = injector.inject_with_refocus(&final_text, target_win.as_deref()).await {
                            tracing::error!("Falha ao injetar texto (Pós-IA): {}", e);
                        } else {
                            state_clone.emit(LumenEvent::InjectionComplete { text: final_text.clone() });
                        }
                    }

                    Self::handle_auto_send(&cmd, final_text.clone(), state_clone.clone(), injector_clone).await;
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
                    is_wake_word_only: false,
        })
    }

    async fn handle_auto_send(
        command: &VoiceCommand,
        text: String,
        state: Arc<LumenState>,
        injector: Arc<RwLock<TextInjector>>,
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
            let injector_lock = injector.read().await;
            match injector_lock.send_enter().await {
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
            is_wake_word_only: false,
        };

        if let Err(e) = db.save_transcription(&record) {
            tracing::warn!("Falha ao salvar histórico no SQLite: {}", e);
        }
    }
}
