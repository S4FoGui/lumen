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
pub struct TextInjector {
    method: InjectionMethod,
    delay_ms: u64,
    typed_fallback_delay_ms: u64,
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
        Self { method, delay_ms, typed_fallback_delay_ms: 220 }
    }

    pub async fn probe_injection_method() -> InjectionMethod {
        let session_type = std::env::var("XDG_SESSION_TYPE").unwrap_or_default().to_lowercase();

        // Wayland: prioriza digitação direta robusta (wtype/ydotool), não clipboard.
        if session_type == "wayland" {
            if which::which("wtype").is_ok() {
                return InjectionMethod::Wayland;
            }
            if which::which("ydotool").is_ok() {
                return InjectionMethod::Ydotool;
            }
            return InjectionMethod::ClipboardPaste;
        }

        if session_type == "x11" && which::which("xdotool").is_ok() {
            return InjectionMethod::X11;
        }

        InjectionMethod::ClipboardPaste
    }

    pub async fn inject(&self, text: &str) -> Result<()> {
        if text.is_empty() { return Ok(()); }

        // Digita diretamente na janela com foco — não usa clipboard para não
        // interferir com o que o usuário copiou e evitar colar no lugar errado.
        if self.simulate_type_text(text).is_ok() {
            return Ok(());
        }

        // Fallback: clipboard + paste (menos confiável — só se digitação falhou)
        self.copy_to_clipboard(text)?;
        std::thread::sleep(Duration::from_millis(self.delay_ms));
        self.simulate_paste()?;

        Ok(())
    }

    fn copy_to_clipboard(&self, text: &str) -> Result<()> {
        // Wayland (wl-copy)
        if let Ok(mut child) = Command::new("wl-copy").stdin(std::process::Stdio::piped()).spawn() {
            use std::io::Write;
            if let Some(mut stdin) = child.stdin.take() {
                let _ = stdin.write_all(text.as_bytes());
            }
            let _ = child.wait();
            return Ok(());
        }

        // Fallback: arboard (Rust)
        if let Ok(mut clipboard) = arboard::Clipboard::new() {
            let _ = clipboard.set_text(text.to_string());
            std::thread::sleep(Duration::from_millis(150));
            return Ok(());
        }
        
        // Fallback: xclip (X11) — via stdin para evitar command injection
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
        }

        Ok(())
    }

    fn simulate_paste(&self) -> Result<()> {
        // Tentativa 1: wtype (Wayland)
        if let Ok(mut child) = Command::new("wtype").arg("-M").arg("ctrl").arg("-k").arg("v").arg("-m").arg("ctrl").spawn() {
            let _ = child.wait();
            return Ok(());
        }

        // Tentativa 2: xdotool (X11/XWayland)
        if let Ok(mut child) = Command::new("xdotool").arg("key").arg("ctrl+v").spawn() {
            let _ = child.wait();
            return Ok(());
        }

        // Tentativa 3: ydotool
        if let Ok(mut child) = Command::new("ydotool").arg("key").arg("29:1").arg("47:1").arg("47:0").arg("29:0").spawn() {
            let _ = child.wait();
            return Ok(());
        }

        Ok(())
    }

    fn simulate_type_text(&self, text: &str) -> Result<()> {
        // 1) Wayland direto
        if let Ok(mut child) = Command::new("wtype").arg(text).spawn() {
            let _ = child.wait();
            return Ok(());
        }

        // 2) X11/XWayland
        if let Ok(mut child) = Command::new("xdotool").arg("type").arg("--delay").arg("1").arg(text).spawn() {
            let _ = child.wait();
            return Ok(());
        }

        // 3) ydotool (best effort)
        if let Ok(mut child) = Command::new("ydotool").arg("type").arg(text).spawn() {
            let _ = child.wait();
            return Ok(());
        }

        Err(crate::error::LumenError::Internal("Falha ao injetar texto diretamente no campo ativo".into()))
    }

    pub async fn send_enter(&self) -> Result<()> {
        std::thread::sleep(Duration::from_millis(600));
        
        // Tentativa 1: wtype
        if let Ok(mut child) = Command::new("wtype").arg("-k").arg("Return").spawn() {
            if child.wait().is_ok() { return Ok(()); }
        }
        
        // Tentativa 2: xdotool
        if let Ok(mut child) = Command::new("xdotool").arg("key").arg("Return").spawn() {
            if child.wait().is_ok() { return Ok(()); }
        }
        
        // Tentativa 3: ydotool
        let _ = Command::new("ydotool").arg("key").arg("28:1").arg("28:0").spawn();

        Ok(())
    }
}
