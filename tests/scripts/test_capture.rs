use cpal::traits::{DeviceTrait, HostTrait};

fn main() {
    let host = cpal::default_host();
    let devices = host.input_devices().unwrap();
    for d in devices {
        if let Ok(name) = d.name() {
            println!("Testing {}", name);
            if let Ok(config) = d.default_input_config() {
                println!("  Supported config: {:?}", config);
            } else {
                println!("  Failed config!");
            }
        }
    }
}
