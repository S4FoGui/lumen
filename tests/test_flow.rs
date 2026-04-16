use std::process::Command;
use std::time::Duration;

fn main() {
    let script = ".system/tools/injector.py";
    
    println!("Testing paste...");
    let mut paste = Command::new("python3").args([script, "paste"]).spawn().unwrap();
    let res = paste.wait().unwrap();
    println!("Paste done: {}", res.success());

    std::thread::sleep(Duration::from_millis(300));

    println!("Testing enter...");
    let mut enter = Command::new("python3").args([script, "enter"]).spawn().unwrap();
    let res2 = enter.wait().unwrap();
    println!("Enter done: {}", res2.success());
}
