use anyhow::Result;
use std::sync::mpsc as std_mpsc;

/// Ícone de bandeja do sistema (system tray) do Lumen.
/// Mostra o estado atual e oferece controles rápidos.
pub struct TrayIcon {
    _handle: std::thread::JoinHandle<()>,
}

/// Eventos do tray menu
#[derive(Debug, Clone)]
pub enum TrayEvent {
    ToggleRecording,
    OpenDashboard,
    Quit,
}

impl TrayIcon {
    /// Cria e mostra o ícone na bandeja do sistema.
    /// Retorna um receiver para eventos do menu.
    pub fn new() -> Result<(Self, std_mpsc::Receiver<TrayEvent>)> {
        let (tx, rx) = std_mpsc::channel();

        let handle = std::thread::spawn(move || {
            match Self::run_tray(tx) {
                Ok(()) => tracing::info!("System tray encerrado"),
                Err(e) => tracing::error!("Erro no system tray: {}", e),
            }
        });

        tracing::info!("🔔 System tray iniciado");

        Ok((Self { _handle: handle }, rx))
    }

    fn run_tray(tx: std_mpsc::Sender<TrayEvent>) -> Result<()> {
        let mut tray = tray_item::TrayItem::new("Lumen", tray_item::IconSource::Resource("lumen"))
            .or_else(|_| tray_item::TrayItem::new("Lumen", tray_item::IconSource::Resource("audio-input-microphone")))
            .unwrap_or_else(|_| {
                // Fallback without icon
                tray_item::TrayItem::new("Lumen", tray_item::IconSource::Resource("dialog-information"))
                    .expect("Falha ao criar tray item")
            });

        // Menu items
        let tx_toggle = tx.clone();
        tray.add_menu_item("⏺ Gravar / Parar", move || {
            let _ = tx_toggle.send(TrayEvent::ToggleRecording);
        }).ok();

        let tx_dash = tx.clone();
        tray.add_menu_item("📊 Dashboard", move || {
            let _ = tx_dash.send(TrayEvent::OpenDashboard);
        }).ok();

        // tray.add_separator().ok(); // Remoção de separator não suportada na config atual

        let tx_quit = tx;
        tray.add_menu_item("❌ Sair", move || {
            let _ = tx_quit.send(TrayEvent::Quit);
        }).ok();

        // Keep the tray alive — this blocks
        loop {
            std::thread::sleep(std::time::Duration::from_secs(1));
        }
    }
}
