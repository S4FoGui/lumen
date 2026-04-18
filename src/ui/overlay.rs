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
    is_owner: bool,
}

impl Overlay {
    pub fn new() -> Self {
        let (sender, receiver) = unbounded::<OverlayMessage>();

        let handle = std::thread::spawn(move || {
            let app = gtk4::Application::builder()
                .application_id("com.github.lumen.overlay")
                .flags(gtk4::gio::ApplicationFlags::FLAGS_NONE)
                .build();

            app.connect_activate(move |app| {
                let window = build_pill_window(app);
                let app_ref = app.clone();

                let container = window
                    .child()
                    .and_then(|c| c.downcast::<gtk4::Box>().ok())
                    .expect("Falha ao inicializar container do overlay (Box)");

                // Status dot (replaces the old DrawingArea waveform)
                let status_dot: gtk4::Image = container
                    .first_child()
                    .and_then(|c| c.downcast::<gtk4::Image>().ok())
                    .expect("Falha ao inicializar status dot do overlay");

                let label: gtk4::Label = container
                    .last_child()
                    .and_then(|c| c.downcast::<gtk4::Label>().ok())
                    .expect("Falha ao inicializar label de transcrição do overlay");

                let receiver_clone = receiver.clone();
                let target_opacity = Arc::new(std::sync::atomic::AtomicU32::new(0));
                let is_recording_state = Arc::new(std::sync::atomic::AtomicBool::new(false));
                let last_activity = Arc::new(std::sync::atomic::AtomicU64::new(glib::monotonic_time() as u64));
                let is_visual_active = Arc::new(std::sync::atomic::AtomicBool::new(false));

                // Instanciar callback de animação contínua (Roda todos os frames)
                let win = window.clone();
                let opacity_target = Arc::clone(&target_opacity);
                let rec_state = Arc::clone(&is_recording_state);
                let activity_log = Arc::clone(&last_activity);
                let visual_active_tick = Arc::clone(&is_visual_active);

                window.add_tick_callback(move |_widget, _clock| {
                    let now_us = glib::monotonic_time() as u64;

                    // Auto-Dismiss Logic
                    let recording = rec_state.load(std::sync::atomic::Ordering::Relaxed);
                    let last = activity_log.load(std::sync::atomic::Ordering::Relaxed);
                    let is_visually_active = visual_active_tick.load(std::sync::atomic::Ordering::Relaxed);

                    if !recording && !is_visually_active && (now_us - last > 3_000_000) {
                        opacity_target.store(0.0f32.to_bits(), std::sync::atomic::Ordering::Relaxed);
                    }

                    // Smooth Opacity Transition
                    let target_o = f32::from_bits(opacity_target.load(std::sync::atomic::Ordering::Relaxed)) as f64;
                    let current_o = win.opacity();
                    if (current_o - target_o).abs() > 0.01 {
                        let next_o = if current_o < target_o {
                            (current_o + 0.15).min(target_o)
                        } else {
                            (current_o - 0.06).max(target_o)
                        };
                        win.set_opacity(next_o);
                    }

                    // Hide window completely when opacity reaches 0
                    if current_o <= 0.01 && win.is_visible() {
                        win.set_visible(false);
                    }

                    // Show window when needed (without present() to avoid focus steal)
                    if target_o > 0.1 && !win.is_visible() {
                        win.set_visible(true);
                    }

                    glib::ControlFlow::Continue
                });

                glib::spawn_future_local(async move {
                    while let Ok(msg) = receiver_clone.recv().await {
                        match msg {
                            OverlayMessage::ShowRecording => {
                                let home = std::env::var("HOME").unwrap_or_else(|_| "/home/gui".into());
                                status_dot.set_from_file(Some(format!("{}/.local/share/lumen/lumen_circle.png", home).as_str()));
                                label.set_text("Ouvindo...");
                                is_recording_state.store(true, std::sync::atomic::Ordering::Relaxed);
                                is_visual_active.store(true, std::sync::atomic::Ordering::Relaxed);
                                target_opacity.store(1.0f32.to_bits(), std::sync::atomic::Ordering::Relaxed);
                                last_activity.store(glib::monotonic_time() as u64, std::sync::atomic::Ordering::Relaxed);

                                window.set_visible(true);
                                if window.opacity() < 0.4 {
                                    window.set_opacity(0.4);
                                }
                            }
                            OverlayMessage::HideRecording => {
                                is_recording_state.store(false, std::sync::atomic::Ordering::Relaxed);
                                is_visual_active.store(false, std::sync::atomic::Ordering::Relaxed);
                                target_opacity.store(0.0f32.to_bits(), std::sync::atomic::Ordering::Relaxed);
                                last_activity.store(glib::monotonic_time() as u64, std::sync::atomic::Ordering::Relaxed);
                            }
                            OverlayMessage::UpdateTranscription(text) => {
                                let preview = if text.chars().count() > 55 {
                                    format!("{}...", text.chars().take(55).collect::<String>())
                                } else {
                                    text.clone()
                                };
                                let home = std::env::var("HOME").unwrap_or_else(|_| "/home/gui".into());
                                status_dot.set_from_file(Some(format!("{}/.local/share/lumen/lumen_circle.png", home).as_str()));
                                label.set_text(&preview);
                                is_recording_state.store(false, std::sync::atomic::Ordering::Relaxed);
                                is_visual_active.store(false, std::sync::atomic::Ordering::Relaxed); // let it auto-dismiss
                                target_opacity.store(0.9f32.to_bits(), std::sync::atomic::Ordering::Relaxed);
                                last_activity.store(glib::monotonic_time() as u64, std::sync::atomic::Ordering::Relaxed);
                                window.set_visible(true); // Garante que a overlay ative.
                            }
                            OverlayMessage::SetVolume(v) => {
                                // Volume data is still received but NOT visualized (no waveform)
                                // Only used to keep the overlay alive during active speech
                                if v > 0.02 {
                                    last_activity.store(glib::monotonic_time() as u64, std::sync::atomic::Ordering::Relaxed);
                                }
                            }
                            OverlayMessage::Shutdown => {
                                window.close();
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
            is_owner: true,
        }
    }

    pub fn from_sender(sender: Sender<OverlayMessage>) -> Self {
        Self {
            sender,
            _handle: None,
            is_owner: false,
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

    #[allow(dead_code)]
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
        if self.is_owner {
            let _ = self.sender.try_send(OverlayMessage::Shutdown);
            if let Some(handle) = self._handle.take() {
                let _ = handle.join();
            }
        }
    }
}

fn build_pill_window(app: &gtk4::Application) -> gtk4::Window {
    let window = gtk4::Window::builder()
        .application(app)
        .decorated(false)
        .resizable(false)
        .focusable(false)          // ✅ Never steal focus
        .focus_on_click(false)     // ✅ Never steal focus on click
        .build();

    window.set_opacity(0.0);
    window.set_visible(true);
    window.set_can_target(false);  // ✅ Click-through: don't intercept mouse events

    #[cfg(feature = "wayland-overlay")]
    {
        use gtk4_layer_shell::{Edge, Layer, LayerShell, KeyboardMode};
        window.init_layer_shell();
        window.set_layer(Layer::Overlay);
        window.set_anchor(Edge::Bottom, true);
        window.set_anchor(Edge::Left, false);
        window.set_anchor(Edge::Right, false);
        window.set_anchor(Edge::Top, false);
        window.set_margin(Edge::Bottom, 25);
        window.set_keyboard_mode(KeyboardMode::None);  // ✅ Never grab keyboard
    }

    let provider = gtk4::CssProvider::new();
    provider.load_from_data(PILL_CSS);
    gtk4::style_context_add_provider_for_display(
        &gtk4::gdk::Display::default().expect("Could not connect to a display."),
        &provider,
        gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );

    let container = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
    container.add_css_class("pill-container");
    container.set_halign(gtk4::Align::Center);
    container.set_valign(gtk4::Align::Center);
    container.set_can_target(false);

    // Status dot indicator (replaces waveform DrawingArea)
    let home = std::env::var("HOME").unwrap_or_else(|_| "/home/gui".into());
    let status_dot = gtk4::Image::builder()
        .file(format!("{}/.local/share/lumen/lumen_circle.png", home))
        .pixel_size(24)
        .build();
    status_dot.add_css_class("status-dot");
    status_dot.set_can_target(false);
    container.append(&status_dot);

    let label = gtk4::Label::new(Some("Ouvindo..."));
    label.add_css_class("transcription-label");
    label.set_can_target(false);
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
    .status-dot {
        font-size: 14px;
        min-width: 20px;
        border-radius: 999px;
    }
    .transcription-label {
        color: #f0f0f0;
        font-family: inherit;
        font-size: 17px;
        font-weight: 600;
        letter-spacing: 0.4px;
    }
"#;
