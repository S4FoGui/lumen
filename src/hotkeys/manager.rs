use anyhow::{Context, Result};
use global_hotkey::{
    hotkey::{Code, HotKey, Modifiers},
    GlobalHotKeyEvent, GlobalHotKeyManager as GHKManager,
};
use tokio::sync::mpsc;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub enum LumenHotkey {
    ToggleRecording,
    LightningMode,
    OpenDashboard,
}

pub struct HotkeyManager {
    _manager: GHKManager,
    _toggle_id: u32,
    _lightning_id: u32,
    _dashboard_id: u32,
}

// Estrutura do evento Linux input_event (24 bytes em x86_64)
// struct input_event { timeval tv_sec; timeval tv_usec; u16 type; u16 code; i32 value; }
#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct InputEvent {
    tv_sec: i64,
    tv_usec: i64,
    ev_type: u16,
    code: u16,
    value: i32,
}

const EV_KEY: u16 = 1;
const KEY_ENTER: u16 = 28;
const KEY_RELEASED: i32 = 0;

impl HotkeyManager {
    pub fn new(
        toggle_str: &str,
        lightning_str: &str,
        dashboard_str: &str,
    ) -> Result<(Self, mpsc::UnboundedReceiver<LumenHotkey>)> {
        let manager = GHKManager::new()
            .context("Falha ao criar gerenciador de hotkeys")?;

        let lightning_hk = parse_hotkey(lightning_str)?;
        let dashboard_hk = parse_hotkey(dashboard_str)?;

        let l_id = lightning_hk.id();
        let d_id = dashboard_hk.id();

        manager.register(lightning_hk).ok();
        manager.register(dashboard_hk).ok();

        let toggle_is_enter = toggle_str.trim().to_lowercase() == "enter";
        let toggle_hk_result = if !toggle_is_enter {
            parse_hotkey(toggle_str).ok().map(|hk| {
                let id = hk.id();
                manager.register(hk).ok();
                id
            })
        } else {
            None
        };
        let t_id = toggle_hk_result.unwrap_or(0);

        let (tx, rx) = mpsc::unbounded_channel();
        let tx_evdev = tx.clone();

        // Thread evdev: lê /dev/input/event* diretamente (funciona em X11 e Wayland)
        if toggle_is_enter {
            std::thread::spawn(move || {
                if let Err(e) = run_evdev_listener(tx_evdev) {
                    tracing::error!("evdev listener falhou: {}", e);
                }
            });
        }

        // Thread global-hotkey: Ctrl+Shift+L e Ctrl+Shift+D
        std::thread::spawn(move || {
            loop {
                if let Ok(event) = GlobalHotKeyEvent::receiver().recv() {
                    if event.state() == global_hotkey::HotKeyState::Released {
                        let hk = if event.id() == t_id {
                            Some(LumenHotkey::ToggleRecording)
                        } else if event.id() == l_id {
                            Some(LumenHotkey::LightningMode)
                        } else if event.id() == d_id {
                            Some(LumenHotkey::OpenDashboard)
                        } else {
                            None
                        };
                        if let Some(h) = hk {
                            let _ = tx.send(h);
                        }
                    }
                }
            }
        });

        Ok((Self { _manager: manager, _toggle_id: t_id, _lightning_id: l_id, _dashboard_id: d_id }, rx))
    }
}

fn run_evdev_listener(tx: mpsc::UnboundedSender<LumenHotkey>) -> Result<()> {
    use std::io::Read;

    // Encontrar todos os dispositivos de teclado em /dev/input/
    let keyboard_devices = find_keyboard_devices();

    if keyboard_devices.is_empty() {
        anyhow::bail!("Nenhum dispositivo de teclado encontrado em /dev/input/");
    }

    tracing::info!("evdev: monitorando {} dispositivo(s) de teclado para Enter duplo", keyboard_devices.len());

    let double_tap_window = Duration::from_millis(400);

    // Abrir todos os dispositivos em threads separadas, compartilhando estado via mutex
    let last_enter = std::sync::Arc::new(std::sync::Mutex::new(Option::<Instant>::None));

    let mut handles = vec![];
    for dev_path in keyboard_devices {
        let tx_clone = tx.clone();
        let last_enter_clone = std::sync::Arc::clone(&last_enter);
        let path_clone = dev_path.clone();

        let handle = std::thread::spawn(move || {
            let mut file = match std::fs::File::open(&path_clone) {
                Ok(f) => f,
                Err(e) => {
                    tracing::warn!("evdev: não foi possível abrir {}: {}", path_clone, e);
                    return;
                }
            };

            let event_size = std::mem::size_of::<InputEvent>();
            let mut buf = vec![0u8; event_size];

            loop {
                match file.read_exact(&mut buf) {
                    Ok(_) => {
                        // Usar read_unaligned para evitar UB com buffer não alinhado
                        let event: InputEvent = unsafe {
                            std::ptr::read_unaligned(buf.as_ptr() as *const InputEvent)
                        };

                        if event.ev_type == EV_KEY
                            && event.code == KEY_ENTER
                            && event.value == KEY_RELEASED
                        {
                            let now = Instant::now();
                            let mut last = last_enter_clone.lock().unwrap();
                            if let Some(prev) = *last {
                                if now.duration_since(prev) < double_tap_window {
                                    *last = None;
                                    drop(last);
                                    tracing::info!("Enter duplo detectado via evdev");
                                    let _ = tx_clone.send(LumenHotkey::ToggleRecording);
                                    continue;
                                }
                            }
                            *last = Some(now);
                        }
                    }
                    Err(_) => break,
                }
            }
        });
        handles.push(handle);
    }

    for h in handles {
        let _ = h.join();
    }

    Ok(())
}

fn find_keyboard_devices() -> Vec<String> {
    let mut devices = vec![];

    if let Ok(content) = std::fs::read_to_string("/proc/bus/input/devices") {
        let mut is_keyboard = false;
        let mut event_num: Option<u32> = None;

        let flush = |is_kbd: bool, num: &mut Option<u32>, devs: &mut Vec<String>| {
            if is_kbd {
                if let Some(n) = num.take() {
                    let path = format!("/dev/input/event{}", n);
                    if std::path::Path::new(&path).exists() {
                        devs.push(path);
                    }
                }
            }
            *num = None;
        };

        for line in content.lines() {
            if line.starts_with("N: Name=") {
                flush(is_keyboard, &mut event_num, &mut devices);
                let name = line.to_lowercase();
                is_keyboard = name.contains("keyboard") || name.contains("kbd");
            } else if line.starts_with("H: Handlers=") && is_keyboard {
                for token in line.split_whitespace() {
                    if let Some(rest) = token.strip_prefix("event") {
                        if let Ok(n) = rest.parse::<u32>() {
                            event_num = Some(n);
                        }
                    }
                }
            }
        }
        // Flush último bloco
        flush(is_keyboard, &mut event_num, &mut devices);
    }

    // Fallback via /sys
    if devices.is_empty() {
        for i in 0..32 {
            let sys_name_path = format!("/sys/class/input/event{}/device/name", i);
            if let Ok(name) = std::fs::read_to_string(&sys_name_path) {
                let name_lower = name.to_lowercase();
                if name_lower.contains("keyboard") || name_lower.contains("kbd") {
                    let path = format!("/dev/input/event{}", i);
                    if std::path::Path::new(&path).exists() {
                        devices.push(path);
                    }
                }
            }
        }
    }

    devices
}

fn parse_hotkey(s: &str) -> Result<HotKey> {
    let s_lower = s.to_lowercase();
    let parts: Vec<&str> = s_lower.split('+').map(|p| p.trim()).collect();
    let mut modifiers = Modifiers::empty();
    let mut key_code = None;

    for part in parts {
        match part {
            "ctrl" | "control" => modifiers |= Modifiers::CONTROL,
            "shift" => modifiers |= Modifiers::SHIFT,
            "alt" => modifiers |= Modifiers::ALT,
            "super" | "meta" | "win" => modifiers |= Modifiers::SUPER,
            key => {
                key_code = Some(match key {
                    "space" => Code::Space,
                    "enter" | "return" => Code::Enter,
                    "tab" => Code::Tab,
                    "escape" | "esc" => Code::Escape,
                    "a" => Code::KeyA, "b" => Code::KeyB, "c" => Code::KeyC,
                    "d" => Code::KeyD, "e" => Code::KeyE, "f" => Code::KeyF,
                    "g" => Code::KeyG, "h" => Code::KeyH, "i" => Code::KeyI,
                    "j" => Code::KeyJ, "k" => Code::KeyK, "l" => Code::KeyL,
                    "m" => Code::KeyM, "n" => Code::KeyN, "o" => Code::KeyO,
                    "p" => Code::KeyP, "q" => Code::KeyQ, "r" => Code::KeyR,
                    "s" => Code::KeyS, "t" => Code::KeyT, "u" => Code::KeyU,
                    "v" => Code::KeyV, "w" => Code::KeyW, "x" => Code::KeyX,
                    "y" => Code::KeyY, "z" => Code::KeyZ,
                    _ => Code::Enter,
                });
            }
        }
    }
    let code = key_code.context("Erro no parse da hotkey")?;
    let mods = if modifiers.is_empty() { None } else { Some(modifiers) };
    Ok(HotKey::new(mods, code))
}
