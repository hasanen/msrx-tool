use rusb::{Context, Device, DeviceHandle, DeviceList, UsbContext};
use std::process;

const MSRX6: [u16; 2] = [0x0801, 0x0003];

fn main() {
    // Initialize a USB context
    let context = Context::new().expect("Failed to initialize USB context");

    let device = match context.open_device_with_vid_pid(MSRX6[0], MSRX6[1]) {
        Some(device) => device,
        None => {
            println!("Device not found");
            process::exit(1)
        }
    };

    dbg!(device);
}
