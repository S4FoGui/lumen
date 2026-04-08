use cpal::traits::{DeviceTrait, HostTrait};

fn main() {
    let host = cpal::default_host();
    let devices = host.input_devices().unwrap();
    // we use a loop to avoid using deprecated .name() without checking if wait, what are the available methods?
    for (i, d) in devices.enumerate() {
        println!("{}:", i);
        println!("  name: {:?}", d.name());
    }
}
