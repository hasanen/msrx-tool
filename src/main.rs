use hex::FromHex;
use rusb::{Context, Device, DeviceHandle, DeviceList, UsbContext};
use rusb::{Direction, Recipient, RequestType};
use std::process;
use std::time::Duration;
use tokio::io::AsyncReadExt;
use tokio::time::sleep;

const MSRX6: [u16; 2] = [0x0801, 0x0003];

enum Command {
    GetFirmwareVersion,
    GetVoltage,
    Reset,
}
impl Command {
    fn packets(&self) -> [u8; 2] {
        match self {
            Command::Reset => [0x1b, 0x61],
            Command::GetFirmwareVersion => [0x1b, 0x76],
        }
    }
}

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

    println!("Reset device");
    sendControl(&mut device, &Command::Reset.packets().to_vec());
    sendControl(&mut device, &Command::GetFirmwareVersion.packets().to_vec());
    let firmware = read_data(&mut device);
    println!("Firmware: {}", firmware);
    sendControl(&mut device, &Command::GetVoltage.packets().to_vec());
    let voltage = read_data(&mut device);
    println!("Voltage: {}", voltage);

    // enable_read(&mut device);

    // // wait 2 seconds
    // sleep(Duration::from_secs(2)).await;

    // let data = read_data(&mut device);
    // print!("{}", data);

    // disable_read(&mut device);

    if (device.release_interface(iface).is_err()) {
        println!("Error claiming interface");
        process::exit(1)
    }
    if kernel_detached {
        device.attach_kernel_driver(iface).unwrap();
    }
}

// fn enable_read(device: &mut DeviceHandle<Context>) {
//     let request_type = rusb::request_type(Direction::Out, RequestType::Standard, Recipient::Device);
//     let buf = [
//         0xc2, 0x1b, 0x6d, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
//         0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
//         0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
//         0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
//         0x00, 0x00, 0x00, 0x00,
//     ];

//     let result = device
//         .write_control(0x21, 9, 0x0300, 0, &buf, Duration::from_secs(1))
//         .unwrap();
// }

// fn disable_read(device: &mut DeviceHandle<Context>) {
//     let request_type = rusb::request_type(Direction::Out, RequestType::Standard, Recipient::Device);
//     let buf = [
//         0xc2, 0x1b, 0x61, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
//         0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
//         0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
//         0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
//         0x00, 0x00, 0x00, 0x00,
//     ];

//     let result = device
//         .write_control(0x21, 9, 0x0300, 0, &buf, Duration::from_secs(1))
//         .unwrap();
// }

fn read_data(device: &mut DeviceHandle<Context>) -> String {
    let hex_data = read_interrupt(device);

    // Convert the [u8; 64] array to a hexadecimal string
    let hex_string: String = hex::encode(&hex_data[1..]);

    // Convert the hexadecimal string to a byte vector
    let bytes = Vec::from_hex(&hex_string).expect("Failed to decode hex string");

    // Convert the byte vector to an ASCII string
    let ascii_string = String::from_utf8_lossy(&bytes);

    println!("hex_data: {:?}", hex_data);
    println!("Hexadecimal: {}", hex_string);
    println!("ASCII: {}", ascii_string);

    return String::from_utf8_lossy(&bytes).to_string();
}

fn read_interrupt(device: &mut DeviceHandle<Context>) -> [u8; 64] {
    let mut inbuf: [u8; 64] = [0; 64];
    device.read_interrupt(0x81, &mut inbuf, Duration::from_secs(10));
    return inbuf;
}

fn sendControl(device: &mut DeviceHandle<Context>, packets: &Vec<u8>) {
    let request_type = rusb::request_type(Direction::Out, RequestType::Standard, Recipient::Device);

    let mut written = 0;
    let incoming_packet_length = packets.len();

    while written < incoming_packet_length {
        let mut header = 128;
        let mut packet_length = 63;

        if (incoming_packet_length - written < packet_length) {
            header += 64;
            packet_length = (incoming_packet_length - written) as usize;
        }
        header += packet_length as u8;
        let chunk_length = written + packet_length;
        let chunk: Vec<u8> = std::iter::once(header)
            .chain(packets[written..chunk_length].iter().cloned())
            .collect();

        written += packet_length;
        println!("chunk: {:?}", chunk);
        send_control_chunk(device, &chunk);
    }

    // let result = device
    //     .write_control(0x21, 9, 0x0300, 0, &combined, Duration::from_secs(1))
    //     .unwrap();
}

fn send_control_chunk(device: &mut DeviceHandle<Context>, chunk: &Vec<u8>) {
    let request_type = rusb::request_type(Direction::Out, RequestType::Standard, Recipient::Device);

    let result = device
        .write_control(0x21, 9, 0x0300, 0, &chunk, Duration::from_secs(1))
        .unwrap();
}

fn read_return(device: &mut DeviceHandle<Context>) {
    let mut data = [0; 64]; // Buffer to hold the data
    let timeout = Duration::from_secs(1); // Timeout for the operation

    match device.read_interrupt(0x81, &mut data, timeout) {
        Ok(bytes_read) => {
            println!("Read {} bytes: {:?}", bytes_read, &data[..bytes_read]);
        }
        Err(e) => {
            println!("Error: {}", e);
        }
    }

    println!("data: {:?}", data)
}
