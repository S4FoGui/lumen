use cpal::traits::{DeviceTrait, HostTrait};

fn main() {
    let host = cpal::default_host();
    let devices = host.input_devices().unwrap();
    for d in devices {
        if let Ok(name) = d.name() {
            if name.contains("Device") || name.contains("Web") || name.contains("Generic") {
                match d.default_input_config() {
                    Ok(config) => println!("{} -> OK: {:?}", name, config),
                    Err(e) => println!("{} -> ERROR: {:?}", name, e),
                }
            }
        }
    }
}
