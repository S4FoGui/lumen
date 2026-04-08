use global_hotkey::GlobalHotKeyEvent;
fn main() {
    let e = GlobalHotKeyEvent::receiver().recv().unwrap();
    println!("{:?}", e);
}
