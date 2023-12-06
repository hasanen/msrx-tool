use rusb::{Context, Device, DeviceHandle, DeviceList, UsbContext};
use rusb::{Direction, Recipient, RequestType};
use std::process;
use std::time::Duration;
use tokio::time::sleep;

const MSRX6: [u16; 2] = [0x0801, 0x0003];

#[tokio::main]
async fn main() {
    // Initialize a USB context
    let context = Context::new().expect("Failed to initialize USB context");

    let mut device = match context.open_device_with_vid_pid(MSRX6[0], MSRX6[1]) {
        Some(device) => device,
        None => {
            println!("Device not found");
            process::exit(1)
        }
    };

    dbg!(&device);
    let iface = 0;
    let mut kernel_detached = false;
    if device.kernel_driver_active(iface).unwrap_or(false) {
        println!("Kernel driver active");
        kernel_detached = true;
        match &device.detach_kernel_driver(iface) {
            Ok(_) => {
                println!("Kernel driver detached");
                kernel_detached = true;
            }
            Err(e) => {
                println!("Error detaching kernel driver: {}", e);
                process::exit(1)
            }
        }
    }
    if (device.claim_interface(iface).is_err()) {
        println!("Error claiming interface");
        process::exit(1)
    }

    enable_read(&mut device);

    // wait 2 seconds
    sleep(Duration::from_secs(2)).await;

    let mut inbuf: [u8; 64] = [0; 64];
    device.read_interrupt(0x81, &mut inbuf, Duration::from_secs(1));

    disable_read(&mut device);

    let mut inbuf2: [u8; 64] = [0; 64];
    device.read_interrupt(0x81, &mut inbuf2, Duration::from_secs(1));

    if (device.release_interface(iface).is_err()) {
        println!("Error claiming interface");
        process::exit(1)
    }
    if kernel_detached {
        device.attach_kernel_driver(iface).unwrap();
    }
}

fn enable_read(device: &mut DeviceHandle<Context>) {
    let request_type = rusb::request_type(Direction::Out, RequestType::Standard, Recipient::Device);
    let buf = [
        0xc2, 0x1b, 0x6d, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00,
    ];

    let result = device
        .write_control(0x21, 9, 0x0300, 0, &buf, Duration::from_secs(1))
        .unwrap();
}

fn disable_read(device: &mut DeviceHandle<Context>) {
    let request_type = rusb::request_type(Direction::Out, RequestType::Standard, Recipient::Device);
    let buf = [
        0xc2, 0x1b, 0x61, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00,
    ];

    let result = device
        .write_control(0x21, 9, 0x0300, 0, &buf, Duration::from_secs(1))
        .unwrap();
}
