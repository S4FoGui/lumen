use global_hotkey::{
    hotkey::{Code, HotKey, Modifiers},
    GlobalHotKeyEvent, GlobalHotKeyManager,
    HotKeyState,
};

fn main() {
    let manager = GlobalHotKeyManager::new().unwrap();
    let hk = HotKey::new(Some(Modifiers::CONTROL | Modifiers::SHIFT), Code::Space);
    manager.register(hk).unwrap();
    
    let rx = GlobalHotKeyEvent::receiver();
    if let Ok(event) = rx.recv() {
        println!("Event: {:?}", event);
        println!("State: {:?}", event.state);
    }
}
