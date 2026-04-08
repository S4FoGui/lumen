use anyhow::Result;
use gtk4::glib;
use gtk4::prelude::*;
use async_channel::{unbounded, Sender};
use std::sync::Arc;

pub enum OverlayMessage {
    ShowRecording,
    HideRecording,
    UpdateTranscription(String),
    SetVolume(f32),
    Shutdown,
}

pub struct Overlay {
    sender: Sender<OverlayMessage>,
    _handle: Option<std::thread::JoinHandle<()>>,
}

impl Overlay {
    pub fn new() -> Self {
        let (sender, receiver) = unbounded::<OverlayMessage>();

        let handle = std::thread::spawn(move || {
            let app = gtk4::Application::builder()
                .application_id("com.github.lumen.overlay")
                .flags(gtk4::gio::ApplicationFlags::NON_UNIQUE)
                .build();

            app.connect_activate(move |app| {
                let window = build_pill_window(app);
                let app_ref = app.clone();

                let container = window
                    .child()
                    .and_then(|c| c.downcast::<gtk4::Box>().ok())
                    .expect("Falha ao inicializar container do overlay (Box)");

                let drawing_area: gtk4::DrawingArea = container
                    .first_child()
                    .and_then(|c| c.downcast::<gtk4::DrawingArea>().ok())
                    .expect("Falha ao inicializar área de desenho do overlay");

                let label: gtk4::Label = container
                    .last_child()
                    .and_then(|c| c.downcast::<gtk4::Label>().ok())
                    .expect("Falha ao inicializar label de transcrição do overlay");

                let receiver_clone = receiver.clone();
                let mut anim_tick_id: Option<gtk4::TickCallbackId> = None;

                let phase = Arc::new(std::sync::atomic::AtomicU64::new(0));
                let volume = Arc::new(std::sync::atomic::AtomicU32::new(0)); // f32 como u32 bits

                // Registrar draw func antes do async move
                let phase_draw = Arc::clone(&phase);
                let volume_draw = Arc::clone(&volume);
                let drawing_area_draw = drawing_area.clone();
                
                drawing_area_draw.set_draw_func(move |_da, cr, width, height| {
                    let t = f64::from_bits(phase_draw.load(std::sync::atomic::Ordering::Relaxed));
                    let vol = f32::from_bits(volume_draw.load(std::sync::atomic::Ordering::Relaxed));

                    // Normalizar volume (RMS costuma ser baixo, ex: 0.01-0.2)
                    let vol_boost = (vol * 15.0).clamp(0.0, 1.5) as f64;

                    let bar_count = 12i32;
                    let bar_w = 3.0f64;
                    let gap = 3.0f64;
                    let total_w = bar_count as f64 * (bar_w + gap) - gap;
                    let start_x = (width as f64 - total_w) / 2.0;
                    let center_y = height as f64 / 2.0;
                    let max_h = height as f64 * 0.8;

                    for i in 0..bar_count {
                        let norm = i as f64 / bar_count as f64;
                        let bell = (std::f64::consts::PI * norm).sin();
                        let wave = (t * 6.0 + norm * std::f64::consts::TAU * 1.5).sin() * 0.5 + 0.5;
                        
                        // Altura reativa: base mínima de 3px + pulso do volume
                        let h = (bell * (0.2 + vol_boost) * wave * max_h).max(3.0);

                        let x = start_x + i as f64 * (bar_w + gap);
                        let y = center_y - h / 2.0;

                        let alpha = 0.5 + wave * 0.5;

                        cr.set_source_rgba(163.0 / 255.0, 230.0 / 255.0, 53.0 / 255.0, alpha * 0.3);
                        cr.rectangle(x - 1.0, y - 1.0, bar_w + 2.0, h + 2.0);
                        let _ = cr.fill();

                        cr.set_source_rgba(163.0 / 255.0, 230.0 / 255.0, 53.0 / 255.0, alpha);
                        cr.rectangle(x, y, bar_w, h);
                        let _ = cr.fill();
                    }
                });

                glib::spawn_future_local(async move {
                    while let Ok(msg) = receiver_clone.recv().await {
                        match msg {
                            OverlayMessage::ShowRecording => {
                                label.set_text("Ouvindo...");
                                window.set_visible(true);

                                if anim_tick_id.is_none() {
                                    let da = drawing_area.clone();
                                    let phase_clone = Arc::clone(&phase);
                                    let tick_id = drawing_area.add_tick_callback(move |_widget, clock| {
                                        let t = clock.frame_time() as f64 / 1_000_000.0;
                                        phase_clone.store(t.to_bits(), std::sync::atomic::Ordering::Relaxed);
                                        da.queue_draw();
                                        glib::ControlFlow::Continue
                                    });
                                    anim_tick_id = Some(tick_id);
                                }
                            }
                            OverlayMessage::HideRecording => {
                                window.set_visible(false);
                                if let Some(tick) = anim_tick_id.take() {
                                    tick.remove();
                                }
                            }
                            OverlayMessage::UpdateTranscription(text) => {
                                let preview = if text.chars().count() > 55 {
                                    format!("{}...", text.chars().take(55).collect::<String>())
                                } else {
                                    text.clone()
                                };
                                label.set_text(&preview);
                                window.set_visible(true);

                                if anim_tick_id.is_none() {
                                    let da = drawing_area.clone();
                                    let phase_clone = Arc::clone(&phase);
                                    let tick_id = drawing_area.add_tick_callback(move |_widget, clock| {
                                        let t = clock.frame_time() as f64 / 1_000_000.0;
                                        phase_clone.store(t.to_bits(), std::sync::atomic::Ordering::Relaxed);
                                        da.queue_draw();
                                        glib::ControlFlow::Continue
                                    });
                                    anim_tick_id = Some(tick_id);
                                }
                            }
                            OverlayMessage::SetVolume(v) => {
                                volume.store(v.to_bits(), std::sync::atomic::Ordering::Relaxed);
                                drawing_area.queue_draw();
                            }
                            OverlayMessage::Shutdown => {
                                window.close();
                                // Forçar saída do loop GTK
                                app_ref.quit();
                                break;
                            }
                        }
                    }
                });
            });

            app.run_with_args::<String>(&[]);
        });

        Self {
            sender,
            _handle: Some(handle),
        }
    }

    /// Cria uma instância do controlador de overlay a partir de um sender existente.
    /// Útil para tarefas que precisam enviar mensagens sem deter a posse da thread GUI.
    pub fn from_sender(sender: Sender<OverlayMessage>) -> Self {
        Self {
            sender,
            _handle: None,
        }
    }

    pub fn show_recording(&mut self) -> Result<()> {
        let _ = self.sender.try_send(OverlayMessage::ShowRecording);
        Ok(())
    }

    pub fn hide_recording(&mut self) -> Result<()> {
        let _ = self.sender.try_send(OverlayMessage::HideRecording);
        Ok(())
    }

    pub fn show_transcription(&mut self, text: &str) -> Result<()> {
        let _ = self.sender.try_send(OverlayMessage::UpdateTranscription(text.to_string()));
        Ok(())
    }

    pub fn clone_sender(&self) -> Sender<OverlayMessage> {
        self.sender.clone()
    }
}

impl Drop for Overlay {
    fn drop(&mut self) {
        let _ = self.sender.try_send(OverlayMessage::Shutdown);
        if let Some(handle) = self._handle.take() {
            let _ = handle.join();
        }
    }
}

fn build_pill_window(app: &gtk4::Application) -> gtk4::Window {
    let window = gtk4::Window::builder()
        .application(app)
        .decorated(false)
        .resizable(false)
        .build();

    #[cfg(feature = "wayland-overlay")]
    {
        use gtk4_layer_shell::{Edge, Layer, LayerShell, KeyboardMode};
        window.init_layer_shell();
        window.set_layer(Layer::Overlay);
        window.set_anchor(Edge::Bottom, true);
        window.set_anchor(Edge::Left, false);
        window.set_anchor(Edge::Right, false);
        window.set_anchor(Edge::Top, false);
        window.set_margin(Edge::Bottom, 60);
        window.set_keyboard_mode(KeyboardMode::None);
    }

    let provider = gtk4::CssProvider::new();
    provider.load_from_data(PILL_CSS);
    gtk4::style_context_add_provider_for_display(
        &gtk4::gdk::Display::default().expect("Could not connect to a display."),
        &provider,
        gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );

    let container = gtk4::Box::new(gtk4::Orientation::Horizontal, 10);
    container.add_css_class("pill-container");
    container.set_halign(gtk4::Align::Center);
    container.set_valign(gtk4::Align::Center);

    let drawing_area = gtk4::DrawingArea::new();
    drawing_area.set_size_request(72, 32);
    container.append(&drawing_area);

    let label = gtk4::Label::new(Some("Ouvindo..."));
    label.add_css_class("transcription-label");
    container.append(&label);

    window.set_child(Some(&container));
    window
}

const PILL_CSS: &str = r#"
    window {
        background: transparent;
    }
    .pill-container {
        background-color: rgba(18, 18, 18, 0.90);
        border: 1px solid rgba(163, 230, 53, 0.25);
        border-radius: 999px;
        padding: 8px 20px 8px 14px;
        box-shadow: 0px 4px 24px rgba(163, 230, 53, 0.12);
    }
    .transcription-label {
        color: #f0f0f0;
        font-family: inherit;
        font-size: 13px;
        font-weight: 600;
        letter-spacing: 0.4px;
    }
"#;
