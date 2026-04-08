use anyhow::{Context, Result};
use std::process::Command;
use std::time::Duration;

/// Módulo de envio automático — pressiona Enter após injetar texto.
///
/// Detecta automaticamente se estamos em X11 ou Wayland e usa o
/// método apropriado para simular o pressionamento da tecla Enter.
#[derive(Clone)]
pub struct AutoSender {
    /// Delay entre a injeção do texto e o pressionamento de Enter (ms).
    /// Permite que o texto termine de ser digitado antes de enviar.
    delay_after_text_ms: u64,
}

impl AutoSender {
    /// Cria um novo AutoSender.
    ///
    /// # Argumentos
    /// - `delay_after_text_ms`: delay em ms antes de pressionar Enter (recomendado: 100-300)
    pub fn new(delay_after_text_ms: u64) -> Self {
        Self { delay_after_text_ms }
    }

    /// Pressiona Enter para enviar a mensagem.
    ///
    /// Detecta o ambiente gráfico (X11/Wayland) e usa xdotool ou wtype.
    pub fn send_enter(&self) -> Result<()> {
        // Aguardar o texto terminar de ser injetado
        if self.delay_after_text_ms > 0 {
            std::thread::sleep(Duration::from_millis(self.delay_after_text_ms));
        }

        let session_type = std::env::var("XDG_SESSION_TYPE")
            .unwrap_or_default()
            .to_lowercase();

        match session_type.as_str() {
            "wayland" => self.send_enter_wayland(),
            _ => self.send_enter_x11(),
        }
    }

    /// Pressiona Enter via wtype (Wayland), com fallback para xdotool e ydotool
    fn send_enter_wayland(&self) -> Result<()> {
        // Tentativa 1: wtype
        if let Ok(status) = Command::new("wtype").args(["-k", "Return"]).status() {
            if status.success() {
                tracing::debug!("Enter enviado via wtype (Wayland)");
                return Ok(());
            }
        }

        // Tentativa 2: xdotool via XWayland (funciona no Brave e apps Electron/XWayland)
        if let Ok(status) = Command::new("xdotool").args(["key", "--clearmodifiers", "Return"]).status() {
            if status.success() {
                tracing::debug!("Enter enviado via xdotool (XWayland)");
                return Ok(());
            }
        }

        // Tentativa 3: ydotool via uinput
        if let Ok(status) = Command::new("ydotool").args(["key", "28:1", "28:0"]).status() {
            if status.success() {
                tracing::debug!("Enter enviado via ydotool");
                return Ok(());
            }
        }

        anyhow::bail!("Nenhum método disponível para pressionar Enter (wtype, xdotool e ydotool falharam)");
    }

    /// Pressiona Enter via xdotool (X11)
    fn send_enter_x11(&self) -> Result<()> {
        let status = Command::new("xdotool")
            .args(["key", "--clearmodifiers", "Return"])
            .status()
            .context("Falha ao executar xdotool para Enter. Instale: sudo pacman -S xdotool")?;

        if !status.success() {
            anyhow::bail!("xdotool falhou ao pressionar Enter");
        }

        tracing::debug!("Enter enviado via X11");
        Ok(())
    }
}
