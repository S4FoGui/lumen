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
        self.inject_with_refocus(text, None).await
    }

    /// Injeta texto na janela alvo, opcionalmente refocando-a antes de colar.
    /// `target_window_id`: ID da janela X11/XWayland que deve receber o texto.
    /// Se fornecido, o injetor refoca essa janela antes de simular Ctrl+V.
    pub async fn inject_with_refocus(&self, text: &str, target_window_id: Option<&str>) -> Result<()> {
        if text.is_empty() {
            return Ok(());
        }

        tracing::info!("📝 Injetando texto ({} caracteres): \"{}\"", text.len(), text);
        
        // ✅ FIX: Sempre usar o método configurado (clipboard paste).
        // A "digitação direta" via xdotool/wtype NÃO funciona em janelas Wayland-nativas
        // (ex: Brave, Firefox, terminais nativos). xdotool retorna exit 0 mas não digita nada.
        // Clipboard paste (wl-copy + Ctrl+V) é o único método confiável no Wayland.
        match self.method {
            InjectionMethod::ClipboardPaste => {
                // Copiar para clipboard ANTES de refocá-la (evita atraso)
                self.copy_to_clipboard(text)?;
                
                // ✅ Refocá-la ANTES de colar — o overlay pode ter roubado o foco
                self.refocus_target_window(target_window_id);
                
                // Delay para o foco e clipboard estabilizarem
                std::thread::sleep(Duration::from_millis(250));
                self.simulate_paste()?;
            }
            _ => {
                // Tentar digitar diretamente; fallback para clipboard
                self.refocus_target_window(target_window_id);
                std::thread::sleep(Duration::from_millis(150));
                
                if self.simulate_type_text(text).is_err() {
                    tracing::warn!("Digitação direta falhou, usando clipboard");
                    self.copy_to_clipboard(text)?;
                    std::thread::sleep(Duration::from_millis(200));
                    self.simulate_paste()?;
                }
            }
        }

        tracing::info!("✅ Texto injetado com sucesso");
        Ok(())
    }

    /// Refoca a janela alvo do usuário antes de colar texto.
    /// Necessário porque o overlay GTK pode roubar o foco durante a gravação.
    fn refocus_target_window(&self, target_window_id: Option<&str>) {
        if let Some(win_id) = target_window_id {
            tracing::info!("🎯 Refocando janela alvo: {}", win_id);
            
            // xdotool windowactivate — funciona em X11 e XWayland
            if let Ok(status) = Command::new("xdotool")
                .args(["windowactivate", "--sync", win_id])
                .status()
            {
                if status.success() {
                    tracing::info!("✅ Janela alvo refocada com sucesso");
                    // Delay extra para o WM processar a mudança de foco
                    std::thread::sleep(Duration::from_millis(150));
                    return;
                }
            }
            tracing::warn!("⚠️ Falha ao refocá-la janela alvo {}", win_id);
        } else {
            tracing::debug!("Nenhuma janela alvo salva — tentando colar na janela atual");
        }
    }

    fn copy_to_clipboard(&self, text: &str) -> Result<()> {
        tracing::debug!("📋 Copiando texto para clipboard...");
        
        // 1) arboard (Rust nativo — melhor persistência de clipboard no Wayland KDE)
        if let Ok(mut clipboard) = arboard::Clipboard::new() {
            if clipboard.set_text(text.to_string()).is_ok() {
                std::thread::sleep(Duration::from_millis(100));
                tracing::info!("📋 Texto copiado via arboard nativo");
                return Ok(());
            } else {
                tracing::warn!("arboard falhou ao definir texto");
            }
        } else {
            tracing::warn!("Falha ao inicializar o arboard");
        }

        // 2) wl-copy (Wayland shell fallback)
        if let Ok(mut child) = Command::new("wl-copy")
            .stdin(std::process::Stdio::piped())
            .spawn()
        {
            use std::io::Write;
            if let Some(mut stdin) = child.stdin.take() {
                let _ = stdin.write_all(text.as_bytes());
            }
            if child.wait().map(|s| s.success()).unwrap_or(false) {
                tracing::info!("📋 Texto copiado via wl-copy");
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
            tracing::info!("📋 Texto copiado via xclip");
        }

        Ok(())
    }

    fn simulate_paste(&self) -> Result<()> {
        let is_wayland = std::env::var("XDG_SESSION_TYPE").unwrap_or_default() == "wayland";
        
        tracing::info!("⌨️ Simulando paste (Ctrl+V)...");

        // 1. ydotool (Wayland-native, funciona sem virtual keyboard protocol)
        if which::which("ydotool").is_ok() {
            let socket = std::env::var("YDOTOOL_SOCKET").unwrap_or_else(|_| "/tmp/ydotoolsock".to_string());
            if let Ok(mut child) = Command::new("ydotool")
                .env("YDOTOOL_SOCKET", &socket)
                .args(["key", "29:1", "47:1", "47:0", "29:0"])
                .spawn()
            {
                if child.wait().map(|s| s.success()).unwrap_or(false) {
                    tracing::info!("✅ Paste via ydotool");
                    return Ok(());
                }
            }
            tracing::debug!("ydotool falhou, tentando próximo método...");
        }

        // 2. Lumen Native Injector (uinput — funciona em qualquer sessão)
        let script_paths = [".system/tools/injector.py", "tools/injector.py"];
        for injector_script in script_paths {
            if std::path::Path::new(injector_script).exists() {
                if let Ok(mut child) = Command::new("python3")
                    .args([injector_script, "paste"])
                    .spawn()
                {
                    if child.wait().map(|s| s.success()).unwrap_or(false) {
                        tracing::info!("✅ Paste via Lumen Native Injector (uinput) no caminho {}", injector_script);
                        return Ok(());
                    }
                }
                tracing::debug!("Lumen Native Injector falhou, tentando próximo método...");
            }
        }

        // 3. wtype (Wayland — funciona apenas se o compositor suporta virtual keyboard)
        if is_wayland && which::which("wtype").is_ok() {
            if let Ok(mut child) = Command::new("wtype")
                .args(["-M", "ctrl", "-k", "v", "-m", "ctrl"])
                .spawn()
            {
                if child.wait().map(|s| s.success()).unwrap_or(false) {
                    tracing::info!("✅ Paste via wtype (Wayland)");
                    return Ok(());
                }
            }
            tracing::debug!("wtype falhou, tentando próximo método...");
        }

        // 4. xdotool (X11 / XWayland — funciona na maioria das apps mesmo no Wayland)
        if which::which("xdotool").is_ok() {
            if let Ok(status) = Command::new("xdotool")
                .args(["key", "--clearmodifiers", "ctrl+v"])
                .status()
            {
                if status.success() {
                    tracing::info!("✅ Paste via xdotool Ctrl+V");
                    return Ok(());
                }
            }
            tracing::debug!("xdotool falhou");
        }

        tracing::warn!("⚠️ Nenhum método de paste funcionou; texto está no clipboard — cole manualmente com Ctrl+V");
        Ok(())
    }

    fn simulate_type_text(&self, text: &str) -> Result<()> {
        let is_wayland = std::env::var("XDG_SESSION_TYPE").unwrap_or_default() == "wayland";

        // wtype (Wayland nativo)
        if is_wayland && which::which("wtype").is_ok() {
            // ✅ Proteção: Tenta detectar se o protocolo é suportado
            // Se o wtype falhar com 'Compositor does not support', ele retorna 1.
            if let Ok(status) = Command::new("wtype").arg(text).status() {
                if status.success() {
                    return Ok(());
                } else {
                    tracing::warn!("wtype falhou (vkeyboard protocol missing na sua sessão), tentando fallback...");
                }
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

        // 1. Lumen Native Injector (uinput)
        let script_paths = [".system/tools/injector.py", "tools/injector.py"];
        for injector_script in script_paths {
            if std::path::Path::new(injector_script).exists() {
                if let Ok(output) = Command::new("python3")
                    .args([injector_script, "enter"])
                    .output()
                {
                    if output.status.success() {
                        tracing::info!("⏎ Enter via Lumen Native Injector (uinput) no caminho {}", injector_script);
                        return Ok(());
                    } else {
                        let stderr = String::from_utf8_lossy(&output.stderr);
                        tracing::warn!("Lumen Native Injector falhou para enter. stderr: {}", stderr);
                    }
                }
            }
        }

        // 2. ydotool
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

        // 3. wtype
        if is_wayland && which::which("wtype").is_ok() {
            if let Ok(status) = Command::new("wtype").args(["-k", "Return"]).status() {
                if status.success() {
                    tracing::debug!("Enter via wtype");
                    return Ok(());
                }
            }
        }

        // 4. xdotool
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

#[cfg(test)]
mod tests {
    use super::*;

    /// Test 2: Text Injector Method Detection
    /// Validates that probe_injection_method returns a valid method
    /// and that inject("") is a no-op.
    #[tokio::test]
    async fn test_probe_injection_method_returns_valid() {
        let method = TextInjector::probe_injection_method().await;
        // Must return one of the known variants
        assert!(
            matches!(
                method,
                InjectionMethod::X11
                    | InjectionMethod::Wayland
                    | InjectionMethod::KdeFakeInput
                    | InjectionMethod::Uinput
                    | InjectionMethod::Ydotool
                    | InjectionMethod::ClipboardPaste
            ),
            "probe_injection_method returned unexpected variant: {:?}",
            method
        );
    }

    #[tokio::test]
    async fn test_inject_empty_text_is_noop() {
        let injector = TextInjector::new(Some("clipboard"), 50).await;
        // Injecting empty text must succeed immediately without side effects
        let result = injector.inject("").await;
        assert!(result.is_ok(), "inject('') should be a no-op but returned error");
    }

    #[tokio::test]
    async fn test_new_with_explicit_method() {
        let injector = TextInjector::new(Some("x11"), 100).await;
        assert_eq!(injector.method, InjectionMethod::X11);

        let injector = TextInjector::new(Some("clipboard_paste"), 100).await;
        assert_eq!(injector.method, InjectionMethod::ClipboardPaste);

        let injector = TextInjector::new(Some("ydotool"), 100).await;
        assert_eq!(injector.method, InjectionMethod::Ydotool);
    }
}
