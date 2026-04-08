use crate::error::LumenResult as Result;
use std::process::Command;
use std::time::Duration;

/// Método de injeção de texto
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InjectionMethod {
    X11,
    Wayland,
    KdeFakeInput,
    Uinput,
    Ydotool,
    ClipboardPaste,
}

/// Injetor de texto — insere texto em qualquer aplicação ativa.
///
/// # Melhorias v1.0.2 (KDE/Plasma/Debian fix)
/// - Prioriza clipboard+paste no Wayland (mais confiável no KDE)
/// - Delay aumentado para 200ms (KDE demora para focar janela)
/// - ydotool como fallback antes de wtype
/// - Detecção aprimorada de ambiente KDE
pub struct TextInjector {
    method: InjectionMethod,
    delay_ms: u64,
}

impl TextInjector {
    pub async fn new(method: Option<&str>, delay_ms: u64) -> Self {
        let method = if let Some(m) = method {
            match m {
                "x11" => InjectionMethod::X11,
                "wayland" => InjectionMethod::Wayland,
                "kdefakeinput" => InjectionMethod::KdeFakeInput,
                "uinput" => InjectionMethod::Uinput,
                "ydotool" => InjectionMethod::Ydotool,
                "clipboard" | "clipboard_paste" => InjectionMethod::ClipboardPaste,
                _ => Self::probe_injection_method().await,
            }
        } else {
            Self::probe_injection_method().await
        };

        tracing::info!("🚀 Injetor inicializado com método: {:?}", method);
        Self { method, delay_ms }
    }

    /// Detecta automaticamente o melhor método para o ambiente atual.
    ///
    /// Prioridade no Wayland/KDE:
    /// 1. ClipboardPaste (wl-copy + xdotool/ydotool Ctrl+V) — mais confiável
    /// 2. ydotool — funciona com uinput
    /// 3. wtype — nem sempre disponível
    pub async fn probe_injection_method() -> InjectionMethod {
        let session_type = std::env::var("XDG_SESSION_TYPE")
            .unwrap_or_default()
            .to_lowercase();
        let desktop = std::env::var("XDG_CURRENT_DESKTOP")
            .unwrap_or_default()
            .to_lowercase();
        let kde = desktop.contains("kde") || desktop.contains("plasma");

        tracing::info!(
            "Detectando método de injeção: session={}, desktop={}",
            session_type, desktop
        );

        if session_type == "wayland" || kde {
            // ✅ No KDE Wayland, clipboard é o mais confiável
            // wl-copy + Ctrl+V via ydotool/xdotool
            let has_wl_copy = which::which("wl-copy").is_ok();
            let has_ydotool = which::which("ydotool").is_ok();
            let has_xdotool = which::which("xdotool").is_ok();

            tracing::info!(
                "Wayland/KDE tools: wl-copy={}, ydotool={}, xdotool={}",
                has_wl_copy, has_ydotool, has_xdotool
            );

            // Clipboard é preferido no KDE porque funciona em apps nativos e Electron
            if has_wl_copy && (has_ydotool || has_xdotool) {
                return InjectionMethod::ClipboardPaste;
            }

            // Fallback para Clipboard mesmo se ferramentas de paste automático faltarem
            // (Melhor que wtype que falha silenciosamente ou quebra no KDE 6)
            return InjectionMethod::ClipboardPaste;
        }

        // X11 puro
        if which::which("xdotool").is_ok() {
            return InjectionMethod::X11;
        }

        InjectionMethod::ClipboardPaste
    }

    pub async fn inject(&self, text: &str) -> Result<()> {
        if text.is_empty() {
            return Ok(());
        }

        tracing::info!("📝 Injetando texto ({} caracteres): \"{}\"", text.len(), text);
        // ✅ Delay antes de injetar (deixa tempo para o foco voltar à janela alvo)
        std::thread::sleep(Duration::from_millis(self.delay_ms));

        match self.method {
            InjectionMethod::ClipboardPaste => {
                // Copiar para clipboard e colar
                self.copy_to_clipboard(text)?;
                // Delay extra para o clipboard estar pronto e a janela focar (KDE 6 é lento)
                std::thread::sleep(Duration::from_millis(400));
                self.simulate_paste()?;
            }
            _ => {
                // Tentar digitar diretamente; fallback para clipboard
                if self.simulate_type_text(text).is_err() {
                    tracing::warn!("Digitação direta falhou, usando clipboard");
                    self.copy_to_clipboard(text)?;
                    std::thread::sleep(Duration::from_millis(100));
                    self.simulate_paste()?;
                }
            }
        }

        tracing::info!("✅ Texto injetado com sucesso");
        Ok(())
    }

    fn copy_to_clipboard(&self, text: &str) -> Result<()> {
        tracing::debug!("📋 Copiando texto para clipboard...");
        // 1) wl-copy (Wayland — mais confiável no KDE)
        if let Ok(mut child) = Command::new("wl-copy")
            .stdin(std::process::Stdio::piped())
            .spawn()
        {
            use std::io::Write;
            if let Some(mut stdin) = child.stdin.take() {
                let _ = stdin.write_all(text.as_bytes());
            }
            if child.wait().map(|s| s.success()).unwrap_or(false) {
                tracing::debug!("Texto copiado via wl-copy");
                return Ok(());
            }
        }

        // 2) arboard (Rust nativo — funciona X11 e Wayland via XWayland)
        if let Ok(mut clipboard) = arboard::Clipboard::new() {
            if clipboard.set_text(text.to_string()).is_ok() {
                std::thread::sleep(Duration::from_millis(100));
                tracing::debug!("Texto copiado via arboard");
                return Ok(());
            }
        }

        // 3) xclip (X11 fallback)
        if let Ok(mut child) = Command::new("xclip")
            .args(["-selection", "clipboard"])
            .stdin(std::process::Stdio::piped())
            .spawn()
        {
            use std::io::Write;
            if let Some(mut stdin) = child.stdin.take() {
                let _ = stdin.write_all(text.as_bytes());
            }
            let _ = child.wait();
            tracing::debug!("Texto copiado via xclip");
        }

        Ok(())
    }

    fn simulate_paste(&self) -> Result<()> {
        let is_wayland = std::env::var("XDG_SESSION_TYPE").unwrap_or_default() == "wayland";
        
        tracing::debug!("⌨️ Simulando paste (Ctrl+V)...");

        // 1. ydotool (Melhor para Wayland nativo)
        if which::which("ydotool").is_ok() {
            // Tenta usar o socket do ydotool se o daemon estiver rodando
            let socket = std::env::var("YDOTOOL_SOCKET").unwrap_or_else(|_| "/tmp/ydotoolsock".to_string());
            if let Ok(mut child) = Command::new("ydotool")
                .env("YDOTOOL_SOCKET", &socket)
                .args(["key", "29:1", "47:1", "47:0", "29:0"])
                .spawn()
            {
                if child.wait().map(|s| s.success()).unwrap_or(false) {
                    tracing::debug!("Paste via ydotool");
                    return Ok(());
                }
            }
        }

        // 2. wtype (Wayland nativo — foca no protocolo do compositor)
        if is_wayland && which::which("wtype").is_ok() {
            if let Ok(mut child) = Command::new("wtype")
                .args(["-M", "ctrl", "-k", "v", "-m", "ctrl"])
                .spawn()
            {
                if child.wait().map(|s| s.success()).unwrap_or(false) {
                    tracing::debug!("Paste via wtype");
                    return Ok(());
                }
            }
        }

        // 3. xdotool (X11 / Fallback XWayland)
        // ✅ IMPORTANTE: No Wayland puro, xdotool reporta sucesso mas falha silenciosamente.
        // Só usamos se NÃO for Wayland ou se as outras falharem miseravelmente.
        if (!is_wayland || which::which("ydotool").is_err()) && which::which("xdotool").is_ok() {
            if let Ok(status) = Command::new("xdotool")
                .args(["key", "--clearmodifiers", "ctrl+v"])
                .status()
            {
                if status.success() {
                    tracing::debug!("Paste via xdotool Ctrl+V");
                    return Ok(());
                }
            }
        }

        tracing::warn!("Nenhum método de paste disponível ou funcional; texto está no clipboard (Ctrl+V manual)");
        Ok(())
    }

    fn simulate_type_text(&self, text: &str) -> Result<()> {
        // wtype (Wayland nativo)
        if let Ok(status) = Command::new("wtype").arg(text).status() {
            if status.success() {
                return Ok(());
            }
        }

        // xdotool type (X11/XWayland)
        if let Ok(status) = Command::new("xdotool")
            .args(["type", "--clearmodifiers", "--delay", "5", text])
            .status()
        {
            if status.success() {
                return Ok(());
            }
        }

        // ydotool type
        if let Ok(status) = Command::new("ydotool").args(["type", text]).status() {
            if status.success() {
                return Ok(());
            }
        }

        Err(crate::error::LumenError::Internal(
            "Falha ao injetar texto diretamente (tente instalar xdotool, wtype ou ydotool)".into()
        ))
    }

    pub async fn send_enter(&self) -> Result<()> {
        let is_wayland = std::env::var("XDG_SESSION_TYPE").unwrap_or_default() == "wayland";
        std::thread::sleep(Duration::from_millis(300));

        // 1. ydotool
        if which::which("ydotool").is_ok() {
            let socket = std::env::var("YDOTOOL_SOCKET").unwrap_or_else(|_| "/tmp/ydotoolsock".to_string());
            if let Ok(mut child) = Command::new("ydotool")
                .env("YDOTOOL_SOCKET", &socket)
                .args(["key", "28:1", "28:0"])
                .spawn()
            {
                if child.wait().map(|s| s.success()).unwrap_or(false) {
                    tracing::debug!("Enter via ydotool");
                    return Ok(());
                }
            }
        }

        // 2. wtype
        if is_wayland && which::which("wtype").is_ok() {
            if let Ok(status) = Command::new("wtype").args(["-k", "Return"]).status() {
                if status.success() {
                    tracing::debug!("Enter via wtype");
                    return Ok(());
                }
            }
        }

        // 3. xdotool
        if (!is_wayland || which::which("ydotool").is_err()) && which::which("xdotool").is_ok() {
            if let Ok(status) = Command::new("xdotool")
                .args(["key", "--clearmodifiers", "Return"])
                .status()
            {
                if status.success() {
                    tracing::debug!("Enter via xdotool");
                    return Ok(());
                }
            }
        }

        Ok(())
    }
}
