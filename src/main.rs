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

    let request_type = rusb::request_type(Direction::Out, RequestType::Standard, Recipient::Device);
    let buf = [
        0xc2, 0x1b, 0xa3, 0x60, 0xf0, 0x19, 0x00, 0xbf, 0x64, 0x9d, 0x75, 0x03, 0x00, 0x00, 0x00,
        0x99, 0xd8, 0x4a, 0x1c, 0x40, 0x16, 0xef, 0x00, 0xc8, 0xf0, 0x19, 0x00, 0x70, 0x41, 0xa5,
        0x75, 0x32, 0x07, 0x01, 0x05, 0x40, 0x16, 0xef, 0x00, 0x05, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0xff, 0xff, 0xff, 0xff, 0xa0, 0x14, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xb0,
        0xf0, 0x19, 0x00, 0xb4,
    ];

    let result = device
        .write_control(0x21, 9, 0x0300, 0, &buf, Duration::from_secs(1))
        .unwrap();

    dbg!(result);
    let mut inbuf: [u8; 64] = [0; 64];

    device.read_interrupt(0x81, &mut inbuf, Duration::from_secs(1));

    let request_type2 =
        rusb::request_type(Direction::Out, RequestType::Standard, Recipient::Device);

    let buf2 = [
        0xc2, 0x1b, 0x6d, 0x10, 0x59, 0x70, 0x75, 0xdb, 0x36, 0x92, 0xcd, 0xfe, 0xff, 0xff, 0xff,
        0x1c, 0xf1, 0x19, 0x00, 0x6a, 0x36, 0x6f, 0x75, 0xc0, 0xa7, 0x4b, 0x75, 0x00, 0x00, 0x00,
        0x00, 0x46, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x84, 0xf1, 0x19, 0x00, 0x00, 0xd0,
        0x3a, 0x00, 0x01, 0x00, 0x00, 0x00, 0x2f, 0x2d, 0xfd, 0xb8, 0x24, 0xf2, 0x19, 0x00, 0xe0,
        0x87, 0xa1, 0x77, 0xc4,
    ];

    let result2 = device
        .write_control(0x21, 9, 0x0300, 0, &buf2, Duration::from_secs(1))
        .unwrap();

    // wait for data

    for _ in 0..5 {
        let mut databuf: [u8; 64] = [0; 64];
        let size = device
            .read_interrupt(0x81, &mut databuf, Duration::from_secs(1))
            .unwrap();
        dbg!(size);
        if size > 0 {
            println!("Received {} bytes: {:?}", size, &databuf[..size]);
            // Process the received data as needed

            break;
        }

        // Optionally, add a delay to avoid busy-waiting
        sleep(Duration::from_millis(100)).await;
    }

    if (device.release_interface(iface).is_err()) {
        println!("Error claiming interface");
        process::exit(1)
    }
    if kernel_detached {
        device.attach_kernel_driver(iface).unwrap();
    }
}
