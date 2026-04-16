mod config;
mod audio;
mod transcription;
mod text;
mod ai;
mod dictionary;
mod hotkeys;
mod ui;
mod state;
pub mod error;
mod analytics;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::sync::Arc;
use tokio::sync::RwLock;

use audio::vad::{VoiceActivityDetector, VadState};
use state::{LumenEvent, LumenState};
use transcription::pipeline::TranscriptionPipeline;

/// Lumen — Ecossistema de Produtividade por Voz para Linux
#[derive(Parser)]
#[command(name = "lumen", version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Mostra o caminho do arquivo de configuração
    Config,
    /// Abre o dashboard no navegador
    Dashboard,
    /// Lista dispositivos de áudio disponíveis
    Devices,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Inicializar logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .with_target(false)
        .init();

    // Banner
    print_banner();

    // Carregar configuração
    let config = config::LumenConfig::load()
        .context("Falha ao carregar configuração")?;

    // Validar configuração
    config.validate().context("Configuração inválida")?;

    // Processar subcomandos
    match cli.command {
        Some(Commands::Config) => {
            println!("📁 Arquivo de configuração: {}", config::LumenConfig::config_path().display());
            println!("📂 Diretório de dados: {}", config::LumenConfig::data_dir().display());
            return Ok(());
        }
        Some(Commands::Dashboard) => {
            let url = format!("http://localhost:{}", config.ui.dashboard_port);
            println!("🌐 Abrindo dashboard: {}", url);
            let _ = std::process::Command::new("xdg-open").arg(&url).spawn();
            return Ok(());
        }
        Some(Commands::Devices) => {
            println!("🎤 Dispositivos de áudio disponíveis:");
            match audio::capture::AudioCapture::list_devices() {
                Ok(devices) => {
                    if devices.is_empty() {
                        println!("  (nenhum dispositivo encontrado)");
                    } else {
                        for (i, d) in devices.iter().enumerate() {
                            println!("  [{}] {}", i, d.1);
                        }
                    }
                }
                Err(e) => println!("  Erro: {}", e),
            }
            return Ok(());
        }
        None => {} // Executar normalmente
    }

    tracing::info!("Iniciando Lumen v{}", env!("CARGO_PKG_VERSION"));

    // ═══════════════════════════════════════════════════════════
    // Inicializar DB e State Hub centralizado
    // ═══════════════════════════════════════════════════════════
    let dashboard_port = config.ui.dashboard_port;
    let open_on_start = config.ui.open_on_start;
    let show_tray = config.ui.show_tray;
    let silence_threshold_ms = config.transcription.silence_threshold_ms;
    // Always Listening DESATIVADO - comportamento 100% manual via hotkey (2x Enter)
    let mut always_listening = false;
    let mut wake_word = config.transcription.wake_word.to_lowercase();

    // 1. Analytics DB
    let analytics_db = Arc::new(analytics::Analytics::init_default().context("Falha ao inicializar banco de analytics")?);

    // 2. State Hub
    let lumen_state = LumenState::new(config.clone(), Arc::clone(&analytics_db));

    // ═══════════════════════════════════════════════════════════
    // Inicializar componentes
    // ═══════════════════════════════════════════════════════════

    // 3. Captura de áudio
    let audio_capture = Arc::new(RwLock::new(
        audio::capture::AudioCapture::new(
            config.audio.device.clone(),
            config.audio.sample_rate,
            config.audio.noise_suppression,
        )
    ));

    // 2. Motor de transcrição
    let model_path = config.model_path();
    let transcription_engine = if model_path.exists() {
        match transcription::engine::TranscriptionEngine::new(
            &model_path,
            &config.transcription.language,
            config.transcription.lightning_mode,
        ) {
            Ok(engine) => Some(Arc::new(engine)),
            Err(e) => {
                tracing::warn!("⚠️ Motor de transcrição não disponível: {}", e);
                tracing::warn!("   Baixe o modelo com:");
                tracing::warn!("   mkdir -p {}", model_path.parent().unwrap_or(&model_path).display());
                tracing::warn!("   curl -L -o {} https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-small.bin", model_path.display());
                None
            }
        }
    } else {
        tracing::warn!("⚠️ Modelo Whisper não encontrado em: {}", model_path.display());
        tracing::warn!("   O Lumen iniciará sem transcrição.");
        tracing::warn!("   Baixe o modelo com:");
        tracing::warn!("   mkdir -p {}", model_path.parent().unwrap_or(&model_path).display());
        tracing::warn!("   curl -L -o {} https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-small.bin", model_path.display());
        None
    };

    // 3. Filtro de fillers
    let filler_filter = transcription::filler_filter::FillerFilter::new(
        &config.transcription.filler_words,
    );

    // 5. Injetor de texto
    let text_injector = Arc::new(RwLock::new(
        text::injector::TextInjector::new(
            Some(&config.text_injection.method),
            config.text_injection.delay_ms,
        ).await
    ));

    // 6. Pipeline de transcrição (NOVO — integra VAD, Commands, Auto-Send)
    let pipeline = transcription_engine.as_ref().map(|engine| {
        Arc::new(TranscriptionPipeline::new(
            Arc::clone(&lumen_state),
            Arc::clone(engine),
            filler_filter,
            Arc::clone(&text_injector),
            Arc::clone(&analytics_db),
        ))
    });

    // 6. VAD — Voice Activity Detector (NOVO)
    let vad = Arc::new(RwLock::new(
        VoiceActivityDetector::new(0.02, silence_threshold_ms) // threshold RMS = 0.02
    ));

    // 7. Web dashboard
    ui::web::server::start_server(Arc::clone(&lumen_state), dashboard_port).await
        .context("Falha ao iniciar servidor web")?;

    // 8. Overlay
    let mut overlay = ui::overlay::Overlay::new();

    // 9. Hotkeys
    let (hotkey_manager, mut hotkey_rx) = hotkeys::manager::HotkeyManager::new(
        &config.hotkeys.toggle_recording,
        &config.hotkeys.lightning_mode,
        &config.hotkeys.open_dashboard,
    ).context("Falha ao registrar hotkeys")?;

    let _hotkey_manager = hotkey_manager;

    // 10. System tray
    let tray_rx = if show_tray {
        match ui::tray::TrayIcon::new() {
            Ok((tray, rx)) => {
                Box::leak(Box::new(tray));
                Some(rx)
            }
            Err(e) => {
                tracing::warn!("System tray não disponível: {}", e);
                None
            }
        }
    } else {
        None
    };

    // Abrir dashboard no navegador (se configurado)
    if open_on_start {
        let url = format!("http://localhost:{}", dashboard_port);
        let _ = std::process::Command::new("xdg-open").arg(&url).spawn();
    }

    tracing::info!("✅ Lumen pronto! Pressione Enter duas vezes rapidamente para gravar.");
    tracing::info!("🌐 Dashboard: http://localhost:{}", dashboard_port);

    // ═══════════════════════════════════════════════════════════
    // Event loop principal
    // ═══════════════════════════════════════════════════════════
    let mut recording = false;
    let mut last_toggle = tokio::time::Instant::now() - tokio::time::Duration::from_secs(1);

    // Canal para VAD enviar sinal de fim de fala
    let (vad_tx, mut vad_rx) = tokio::sync::mpsc::unbounded_channel::<()>();

    // Modo 100% manual - aguardando hotkey (2x Enter) para iniciar

    // Subscrever ao event bus para reagir a mudanças de config em tempo real
    let mut config_event_rx = lumen_state.event_tx.subscribe();

    loop {
        tokio::select! {
            // Reagir a mudanças de config (always_listening ignorado - comportamento 100% manual)
            Ok(event) = config_event_rx.recv() => {
                if matches!(event, LumenEvent::ConfigChanged) {
                    let new_config = lumen_state.config.read().await;
                    let _new_always_listening = new_config.transcription.always_listening;
                    let new_wake_word = new_config.transcription.wake_word.to_lowercase();
                    drop(new_config);

                    // Always Listening está DESATIVADO - sempre força false
                    // Ignora qualquer tentativa de ativação via dashboard
                    if always_listening && recording {
                        // Se por algum motivo estiver gravando, para
                        tracing::info!("🎧 Modo manual: parando gravação ativa");
                        handle_stop_and_process(
                            &mut recording,
                            &lumen_state,
                            &audio_capture,
                            &pipeline,
                            &mut overlay,
                            false,
                            "lumen",
                        ).await;
                    }
                    always_listening = false;
                    wake_word = new_wake_word;
                }
            }

            // Hotkey events
            Some(event) = hotkey_rx.recv() => {
                match event {
                    hotkeys::manager::LumenHotkey::ToggleRecording => {
                        let now = tokio::time::Instant::now();
                        if now.duration_since(last_toggle) < tokio::time::Duration::from_millis(800) {
                            tracing::debug!("Hotkey debounce: ignorando toggle rápido");
                            continue;
                        }
                        last_toggle = now;

                        if always_listening {
                            tracing::info!("🎧 Always Listening ativo: hotkey de gravação ignorada");
                        } else {
                            handle_toggle_recording(
                                &mut recording,
                                &lumen_state,
                                &audio_capture,
                                &pipeline,
                                &mut overlay,
                                &vad,
                                &vad_tx,
                            ).await;
                        }
                    }
                    hotkeys::manager::LumenHotkey::LightningMode => {
                        tracing::info!("⚡ Lightning mode toggle (será implementado com Arc<RwLock<Engine>>)");
                    }
                    hotkeys::manager::LumenHotkey::OpenDashboard => {
                        let url = format!("http://localhost:{}", dashboard_port);
                        let _ = std::process::Command::new("xdg-open").arg(&url).spawn();
                    }
                }
            }

            // VAD: sinal de fim de fala → parar e processar automaticamente
            Some(()) = vad_rx.recv() => {
                if recording {
                    tracing::info!("🎤 VAD: Fim de fala detectado — processando...");
                    handle_stop_and_process(
                        &mut recording,
                        &lumen_state,
                        &audio_capture,
                        &pipeline,
                        &mut overlay,
                        always_listening,
                        &wake_word,
                    ).await;
                    // Modo 100% manual - sem reinício automático
                }
            }

            // Ctrl+C
            _ = tokio::signal::ctrl_c() => {
                tracing::info!("🛑 Encerrando Lumen...");
                break;
            }
        }

        // Poll tray events (non-blocking)
        if let Some(ref tray_rx) = tray_rx {
            while let Ok(event) = tray_rx.try_recv() {
                match event {
                    ui::tray::TrayEvent::ToggleRecording => {
                        // Debounce compartilhado com hotkey para evitar toggles duplos
                        let now = tokio::time::Instant::now();
                        if now.duration_since(last_toggle) < tokio::time::Duration::from_millis(800) {
                            tracing::debug!("Tray debounce: ignorando toggle rápido");
                            continue;
                        }
                        last_toggle = now;

                        if always_listening {
                            tracing::info!("🎧 Always Listening ativo: toggle do tray ignorado");
                        } else {
                            handle_toggle_recording(
                                &mut recording,
                                &lumen_state,
                                &audio_capture,
                                &pipeline,
                                &mut overlay,
                                &vad,
                                &vad_tx,
                            ).await;
                        }
                    }
                    ui::tray::TrayEvent::OpenDashboard => {
                        let url = format!("http://localhost:{}", dashboard_port);
                        let _ = std::process::Command::new("xdg-open").arg(&url).spawn();
                    }
                    ui::tray::TrayEvent::Quit => {
                        tracing::info!("🛑 Saindo via tray...");
                        return Ok(());
                    }
                }
            }
        }
    }

    Ok(())
}

// ═══════════════════════════════════════════════════════════════
// Toggle recording (hotkey / tray)
// ═══════════════════════════════════════════════════════════════

async fn handle_toggle_recording(
    recording: &mut bool,
    state: &Arc<LumenState>,
    audio_capture: &Arc<RwLock<audio::capture::AudioCapture>>,
    pipeline: &Option<Arc<TranscriptionPipeline>>,
    overlay: &mut ui::overlay::Overlay,
    vad: &Arc<RwLock<VoiceActivityDetector>>,
    vad_tx: &tokio::sync::mpsc::UnboundedSender<()>,
) {
    if *recording {
        // ── Parar manualmente (hotkey pressionada novamente) ──
        handle_stop_and_process(
            recording,
            state,
            audio_capture,
            pipeline,
            overlay,
            false,
            "lumen",
        ).await;
    } else {
        // ── Iniciar gravação ──
        
        // ✅ ANTES de mostrar o overlay: capturar a janela focada do usuário
        // Isso é necessário porque o overlay pode roubar o foco,
        // e precisamos refocá-la antes de colar o texto transcrito.
        {
            let window_id = std::process::Command::new("xdotool")
                .arg("getactivewindow")
                .output()
                .ok()
                .and_then(|o| if o.status.success() {
                    String::from_utf8(o.stdout).ok().map(|s| s.trim().to_string())
                } else {
                    None
                });
            
            if let Some(ref id) = window_id {
                tracing::info!("🎯 Janela alvo capturada: {}", id);
            } else {
                tracing::debug!("Não foi possível capturar janela alvo (xdotool indisponível ou Wayland puro)");
            }
            *state.target_window_id.write().await = window_id;
        }
        
        *recording = true;
        *state.is_recording.write().await = true;
        state.emit(LumenEvent::RecordingStarted);
        overlay.show_recording().ok();

        // Resetar VAD para nova sessão
        {
            let mut v = vad.write().await;
            v.reset();
        }

        let mut capture = audio_capture.write().await;
        
        // Sincronizar dispositivo da config atual (pode ter mudado via dashboard)
        {
            let current_config = state.config.read().await;
            capture.set_device(current_config.audio.device.clone());
        }
        
        if let Err(e) = capture.start() {
            tracing::error!("Falha ao iniciar gravação: {}", e);
            *recording = false;
            *state.is_recording.write().await = false;
            state.emit(LumenEvent::RecordingStopped);
            overlay.hide_recording().ok();
            return;
        }

        // Spawn task de monitoramento VAD
        let vad_clone = Arc::clone(vad);
        let capture_samples = capture.samples_ref();
        let vad_tx_clone = vad_tx.clone();
        let state_clone = Arc::clone(state);
        let overlay_sender = overlay.clone_sender();

        tokio::spawn(async move {
            let overlay_sender = overlay_sender.clone(); // Clone do Sender<OverlayMessage>
            let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(30));
            let mut last_len = 0usize;

            loop {
                interval.tick().await;

                // Checar se ainda estamos gravando
                if !*state_clone.is_recording.read().await {
                    break;
                }

                // Ler novos samples do buffer compartilhado
                let new_samples = {
                    if let Ok(samples) = capture_samples.lock() {
                        // Proteção contra pânico de índice se o buffer for reduzido/limpo externamente
                        if samples.len() < last_len {
                            last_len = 0;
                        }

                        if samples.len() > last_len {
                            let chunk = samples[last_len..].to_vec();
                            last_len = samples.len();
                            Some(chunk)
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                };

                if let Some(chunk) = new_samples {
                    let mut v = vad_clone.write().await;
                    let vad_state = v.process(&chunk);

                    match vad_state {
                        VadState::Speaking { rms } => {
                            state_clone.emit(LumenEvent::AudioLevel { rms });
                            let _ = overlay_sender.try_send(ui::overlay::OverlayMessage::SetVolume(rms));
                        }
                        VadState::Silence { rms } => {
                            state_clone.emit(LumenEvent::AudioLevel { rms });
                            let _ = overlay_sender.try_send(ui::overlay::OverlayMessage::SetVolume(rms));
                        }
                        VadState::SpeechEnded => {
                            tracing::debug!("VAD: SpeechEnded signal");
                            let _ = vad_tx_clone.send(());
                            break;
                        }
                    }
                }
            }
        });

        tracing::info!("🎙️ Gravação iniciada (VAD ativo)");
    }
}

// ═══════════════════════════════════════════════════════════════
// Parar e processar (compartilhado entre toggle e VAD)
// ═══════════════════════════════════════════════════════════════

async fn handle_stop_and_process(
    recording: &mut bool,
    state: &Arc<LumenState>,
    audio_capture: &Arc<RwLock<audio::capture::AudioCapture>>,
    pipeline: &Option<Arc<TranscriptionPipeline>>,
    overlay: &mut ui::overlay::Overlay,
    always_listening: bool,
    wake_word: &str,
) {
    *recording = false;
    *state.is_recording.write().await = false;
    state.emit(LumenEvent::RecordingStopped);
    overlay.hide_recording().ok();

    // Obter samples do buffer
    let samples = {
        let mut capture = audio_capture.write().await;
        capture.stop().unwrap_or_default()
    };

    if samples.is_empty() {
        tracing::warn!("Nenhuma amostra capturada");
        return;
    }

    // ✅ Verificar duração mínima (evita processar cliques acidentais/rápidos)
    let duration_ms = (samples.len() as f32 / state.config.read().await.audio.sample_rate as f32) * 1000.0;
    if duration_ms < 200.0 {
        tracing::info!("🎤 Gravação muito curta ({:.0}ms) — descartando", duration_ms);
        return;
    }

    tracing::info!("Capturadas {} amostras ({:.1}s de áudio)", samples.len(), duration_ms / 1000.0);

    // Processar via Pipeline (SPAWE TASK — não bloqueia o loop de hotkeys)
    if let Some(pipe) = pipeline {
        let pipe = Arc::clone(pipe);
        let overlay_sender = overlay.clone_sender();
        let always = always_listening;
        let wake = wake_word.to_string();

        tokio::spawn(async move {
            tracing::info!("🧠 Iniciando transcrição em segundo plano...");
            match pipe.process(samples).await {
                Ok(record) => {
                    // Wake word gate no modo always listening
                    if always {
                        let lower = record.raw_text.to_lowercase();
                        let wake_lower = wake.to_lowercase();
                        let has_wake = lower.starts_with(&wake_lower)
                            || lower.contains(&format!("{} ", wake_lower))
                            || lower.contains(&format!("{}:", wake_lower));

                        if !has_wake {
                            tracing::info!("🎧 Always Listening: wake word não detectada");
                            tracing::info!("🎧 Always Listening: wake word detectada");
                        }
                    } else if record.processed_text.is_empty() {
                        tracing::debug!("Modo normal: texto processado está vazio");
                    }
                }
                Err(e) => {
                    tracing::warn!("Erro na transcrição em segundo plano: {}", e);
                }
            }
        });
    } else {
        tracing::error!("Motor de transcrição não disponível");
    }
}

/// Imprime o banner do Lumen
fn print_banner() {
    println!(r#"
   ██╗     ██╗   ██╗███╗   ███╗███████╗███╗   ██╗
   ██║     ██║   ██║████╗ ████║██╔════╝████╗  ██║
   ██║     ██║   ██║██╔████╔██║█████╗  ██╔██╗ ██║
   ██║     ██║   ██║██║╚██╔╝██║██╔══╝  ██║╚██╗██║
   ███████╗╚██████╔╝██║ ╚═╝ ██║███████╗██║ ╚████║
   ╚══════╝ ╚═════╝ ╚═╝     ╚═╝╚══════╝╚═╝  ╚═══╝
    Voice Productivity Ecosystem for Linux
    "#);
}
