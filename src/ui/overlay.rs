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
                let volume = Arc::new(std::sync::atomic::AtomicU32::new(0));
                let smoothed_volume = Arc::new(std::sync::atomic::AtomicU32::new(0));
                let target_opacity = Arc::new(std::sync::atomic::AtomicU32::new(0));
                let is_recording_state = Arc::new(std::sync::atomic::AtomicBool::new(false));
                let last_activity = Arc::new(std::sync::atomic::AtomicU64::new(glib::monotonic_time() as u64));
                // Estado visual: ativo (gravando) vs inativo (idle/processing)
                let is_visual_active = Arc::new(std::sync::atomic::AtomicBool::new(false));

                // Registrar draw func antes do async move
                let phase_draw = Arc::clone(&phase);
                let volume_draw = Arc::clone(&volume);
                let visual_active_draw = Arc::clone(&is_visual_active);
                let drawing_area_draw = drawing_area.clone();

                drawing_area_draw.set_draw_func(move |_da, cr, width, height| {
                    let t = f64::from_bits(phase_draw.load(std::sync::atomic::Ordering::Relaxed));
                    let vol = f32::from_bits(volume_draw.load(std::sync::atomic::Ordering::Relaxed));
                    let is_active = visual_active_draw.load(std::sync::atomic::Ordering::Relaxed);

                    let center_y = height as f64 / 2.0;
                    let w = width as f64;

                    // Ganho dinâmico baseado no volume suavizado
                    let activity = (vol * 15.0).min(1.5) as f64;

                    // Cor baseada no estado: verde (ativo) ou cinza (inativo)
                    let (r, g, b) = if is_active {
                        (0.2, 1.0, 0.4) // Verde vivo (ativo)
                    } else {
                        (0.5, 0.5, 0.5) // Cinza (inativo)
                    };

                    let draw_layer = |opacity: f64, freq_mult: f64, speed_mult: f64, phase_off: f64| {
                        cr.set_source_rgba(r, g, b, opacity);
                        cr.move_to(0.0, height as f64);

                        // Se inativo, usar animação muito mais lenta e sutil
                        let activity_level = if is_active {
                            activity.max(0.005)
                        } else {
                            0.1 // Mínimo para mostrar algo quando inativo
                        };

                        let phase = t;
                        let amp_base = height as f64 * 0.3 * activity_level;
                        let p = if is_active {
                            phase * speed_mult + phase_off
                        } else {
                            phase * 0.2 + phase_off // Animação 5x mais lenta quando inativo
                        };

                        // Passo mais fino (1px) e inclusivo (<= width) para suavidade máxima
                        for x in 0..=width {
                            let x_f = x as f64;
                            let norm_x = x_f / width as f64 * std::f64::consts::PI * 2.0 * freq_mult;

                            let y1 = (norm_x + p).sin();
                            let y2 = (norm_x * 1.6 - p * 1.3).cos() * 0.45;
                            let y3 = (norm_x * 2.5 + p * 0.8).sin() * 0.25;

                            let y_offset = (y1 + y2 + y3) * amp_base;
                            cr.line_to(x_f, center_y + y_offset);
                        }

                        cr.line_to(width as f64, height as f64);
                        cr.close_path();
                        let _ = cr.fill();
                    };

                    // Quando inativo, sempre mostrar linha sutil (não sumir completamente)
                    if is_active && activity > 0.02 {
                        draw_layer(0.12, 0.9, 1.1, 0.0);
                        draw_layer(0.22, 1.4, 0.9, 2.3);
                        draw_layer(0.45, 0.8, 1.4, 4.7);
                    } else {
                        // Estado inativo: linha cinza/opaca permanente
                        let idle_opacity = if is_active { 0.3 } else { 0.5 };
                        cr.set_source_rgba(r, g, b, idle_opacity);
                        cr.set_line_width(2.0);
                        cr.move_to(0.0, center_y);
                        cr.line_to(w, center_y);
                        let _ = cr.stroke();
                    }
                });

                glib::spawn_future_local(async move {
                    while let Ok(msg) = receiver_clone.recv().await {
                        match msg {
                            OverlayMessage::ShowRecording => {
                                label.set_text("Ouvindo...");
                                is_recording_state.store(true, std::sync::atomic::Ordering::Relaxed);
                                is_visual_active.store(true, std::sync::atomic::Ordering::Relaxed);
                                target_opacity.store(1.0f32.to_bits(), std::sync::atomic::Ordering::Relaxed);
                                last_activity.store(glib::monotonic_time() as u64, std::sync::atomic::Ordering::Relaxed);

                                // SEMPRE garantir que a janela está visível e ativa
                                window.present();
                                window.set_visible(true);
                                if let Some(surface) = window.surface() {
                                    surface.beep(); // Atenção ao usuário (se suportado)
                                }

                                // Reset manual imediato para gatilho visual se estiver invisível
                                if window.opacity() < 0.4 {
                                    window.set_opacity(0.4);
                                }

                                if anim_tick_id.is_none() {
                                    let da = drawing_area.clone();
                                    let win = window.clone();

                                    // Clones para o ticker
                                    let phase_tick = Arc::clone(&phase);
                                    let vol_raw = Arc::clone(&volume);
                                    let vol_smooth = Arc::clone(&smoothed_volume);
                                    let opacity_target = Arc::clone(&target_opacity);
                                    let rec_state = Arc::clone(&is_recording_state);
                                    let activity_log = Arc::clone(&last_activity);
                                    let visual_active_tick = Arc::clone(&is_visual_active);

                                    let tick_id = drawing_area.add_tick_callback(move |_widget, clock| {
                                        let now_us = glib::monotonic_time() as u64;
                                        let t = clock.frame_time() as f64 / 1_000_000.0;
                                        phase_tick.store(t.to_bits(), std::sync::atomic::Ordering::Relaxed);

                                        // 1. Suavizar Volume (EMA)
                                        let target_v = f32::from_bits(vol_raw.load(std::sync::atomic::Ordering::Relaxed));
                                        let current_v = f32::from_bits(vol_smooth.load(std::sync::atomic::Ordering::Relaxed));
                                        let next_v = current_v * 0.85 + target_v * 0.15;
                                        vol_smooth.store(next_v.to_bits(), std::sync::atomic::Ordering::Relaxed);

                                        // 2. Auto-Dismiss Logic - FECHAR completamente quando idle
                                        let recording = rec_state.load(std::sync::atomic::Ordering::Relaxed);
                                        let last = activity_log.load(std::sync::atomic::Ordering::Relaxed);
                                        let is_visually_active = visual_active_tick.load(std::sync::atomic::Ordering::Relaxed);

                                        if !recording && !is_visually_active && (now_us - last > 3_000_000) {
                                            // Fechar completamente após 3 segundos de idle
                                            opacity_target.store(0.0f32.to_bits(), std::sync::atomic::Ordering::Relaxed);
                                        }

                                        // 3. Smooth Opacity Transition
                                        let target_o = f32::from_bits(opacity_target.load(std::sync::atomic::Ordering::Relaxed)) as f64;
                                        let current_o = win.opacity();
                                        if (current_o - target_o).abs() > 0.01 {
                                            let next_o = if current_o < target_o {
                                                // Acelerar subida (aparecer rápido)
                                                (current_o + 0.15).min(target_o)
                                            } else {
                                                // Descida normal para fechar
                                                (current_o - 0.06).max(target_o)
                                            };
                                            win.set_opacity(next_o);
                                        }

                                        // Quando opacidade chega a 0, esconder janela completamente
                                        if current_o <= 0.01 && win.is_visible() {
                                            win.set_visible(false);
                                        }

                                        // Mostrar janela quando precisa aparecer
                                        if target_o > 0.1 && !win.is_visible() {
                                            win.set_visible(true);
                                            win.present();
                                        }

                                        da.queue_draw();
                                        glib::ControlFlow::Continue
                                    });
                                    anim_tick_id = Some(tick_id);
                                }
                            }
                            OverlayMessage::HideRecording => {
                                // FECHAR completamente - sumir da tela
                                is_recording_state.store(false, std::sync::atomic::Ordering::Relaxed);
                                is_visual_active.store(false, std::sync::atomic::Ordering::Relaxed);
                                // Opacidade 0 = invisível
                                target_opacity.store(0.0f32.to_bits(), std::sync::atomic::Ordering::Relaxed);
                                last_activity.store(glib::monotonic_time() as u64, std::sync::atomic::Ordering::Relaxed);
                            }
                            OverlayMessage::UpdateTranscription(text) => {
                                let preview = if text.chars().count() > 55 {
                                    format!("{}...", text.chars().take(55).collect::<String>())
                                } else {
                                    text.clone()
                                };
                                label.set_text(&preview);
                                is_recording_state.store(false, std::sync::atomic::Ordering::Relaxed);
                                is_visual_active.store(false, std::sync::atomic::Ordering::Relaxed);
                                // Manter visível durante exibição da transcrição
                                target_opacity.store(0.9f32.to_bits(), std::sync::atomic::Ordering::Relaxed);
                                last_activity.store(glib::monotonic_time() as u64, std::sync::atomic::Ordering::Relaxed);
                            }
                            OverlayMessage::SetVolume(v) => {
                                volume.store(v.to_bits(), std::sync::atomic::Ordering::Relaxed);
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
        .focusable(false)
        .focus_on_click(false)
        .build();

    window.set_opacity(0.0);
    window.set_visible(true);

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

    // X11 sem layer-shell: não é possível ter true overlay sem WM support
    // O overlay vai funcionar mas pode ser minimizado pelo WM quando outra janela ganha foco
    // Para experiência completa de overlay, use Wayland ou um WM que suporte _NET_WM_STATE_ABOVE

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
    container.set_can_target(false);

    let drawing_area = gtk4::DrawingArea::new();
    drawing_area.set_size_request(72, 32);
    drawing_area.set_can_target(false);
    container.append(&drawing_area);

    let label = gtk4::Label::new(Some("Ouvindo..."));
    label.add_css_class("transcription-label");
    label.set_can_target(false);
    container.append(&label);

    window.set_child(Some(&container));
    window.set_can_target(false);
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
