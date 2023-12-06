use hex::FromHex;
use rusb::{Context, Device, DeviceHandle, DeviceList, UsbContext};
use rusb::{Direction, Recipient, RequestType};
use std::process;
use std::time::Duration;
use tokio::io::AsyncReadExt;
use tokio::time::sleep;

enum Command {
    Reset,
    GetFirmwareVersion,
    GetVoltage,
    GetDeviceModel,
    SetBCP,
    SetBPI,
    SetHiCo,
    SetLoCo,
    SetLeadingZeros,
    SetReadModeOn,
    SetReadModeOff,
}
impl Command {
    fn packets(&self) -> Vec<u8> {
        match self {
            Command::Reset => vec![0x1b, 0x61],
            Command::GetFirmwareVersion => vec![0x1b, 0x76],
            Command::GetVoltage => vec![0x1b, 0xa3],
            Command::GetDeviceModel => vec![0x1b, 0x74],
            Command::SetBCP => vec![0x1b, 0x6f],
            Command::SetBPI => vec![0x1b, 0x62],
            Command::SetHiCo => vec![0x1b, 0x78],
            Command::SetLoCo => vec![0x1b, 0x79],
            Command::SetLeadingZeros => vec![0x1b, 0x7a],
            Command::SetReadModeOn => vec![0x1b, 0x6d],
            Command::SetReadModeOff => vec![0x1b, 0x61],
        }
    }

    fn with_payload(&self, payload: &Vec<u8>) -> Vec<u8> {
        let mut packets = self.packets().to_vec();
        packets.extend(payload);
        return packets;
    }
}

#[derive(Debug)]
struct Track {
    bpc: u8,
    bpi: u8,
    bpi75: u8,
    bpi210: u8,
}
impl Track {
    fn bpi_packets(&self) -> Vec<u8> {
        match self.bpi {
            75 => vec![self.bpi75].clone(),
            210 => vec![self.bpi210].clone(),
            _ => panic!("Invalid BPI"),
        }
    }
}
#[derive(Debug)]
struct DeviceConfig {
    track1: Track,
    track2: Track,
    track3: Track,
    leading_zero210: u8,
    leading_zero75: u8,
    is_hi_co: bool,
    product_id: u16,
    vendor_id: u16,
}

impl DeviceConfig {
    fn msrx6() -> DeviceConfig {
        DeviceConfig {
            track1: Track {
                bpc: 5,
                bpi: 210,
                bpi75: 0xa0,
                bpi210: 0xa1,
            },
            track2: Track {
                bpc: 5,
                bpi: 75,
                bpi75: 0xc0,
                bpi210: 0xc1,
            },
            track3: Track {
                bpc: 5,
                bpi: 210,
                bpi75: 0x4b,
                bpi210: 0xd2,
            },
            leading_zero210: 61,
            leading_zero75: 22,
            is_hi_co: true,
            product_id: 0x0003,
            vendor_id: 0x0801,
        }
    }

    fn bpc_packets(&self) -> Vec<u8> {
        [self.track1.bpc, self.track2.bpc, self.track3.bpc]
            .iter()
            .cloned()
            .collect::<Vec<u8>>()
    }

    fn leading_zero_packets(&self) -> Vec<u8> {
        [self.leading_zero210, self.leading_zero75]
            .iter()
            .cloned()
            .collect::<Vec<u8>>()
    }
}

#[tokio::main]
async fn main() {
    let msrx6 = DeviceConfig::msrx6();
    // Initialize a USB context
    let context = Context::new().expect("Failed to initialize USB context");

    let mut device = match context.open_device_with_vid_pid(msrx6.vendor_id, msrx6.product_id) {
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
    send_control(&mut device, &Command::Reset.packets());
    println!("read firmware");
    send_control(&mut device, &Command::GetFirmwareVersion.packets());
    let firmware = read_firmware(&mut device);
    println!("Firmware: {}", firmware);

    // Set Bit Control Parity (BCP)
    send_control(
        &mut device,
        &Command::SetBCP.with_payload(&msrx6.bpc_packets()),
    );
    read_success(&mut device);

    if (msrx6.is_hi_co) {
        send_control(&mut device, &Command::SetHiCo.packets());
    } else {
        send_control(&mut device, &Command::SetLoCo.packets());
    }
    read_success(&mut device);

    println!("Set BPI track1");
    send_control(
        &mut device,
        &Command::SetBPI.with_payload(&msrx6.track1.bpi_packets()),
    );
    read_success(&mut device);

    println!("Set BPI track2");
    send_control(
        &mut device,
        &Command::SetBPI.with_payload(&msrx6.track2.bpi_packets()),
    );
    read_success(&mut device);

    println!("Set BPI track3");
    send_control(
        &mut device,
        &Command::SetBPI.with_payload(&msrx6.track3.bpi_packets()),
    );
    read_success(&mut device);
    println!("Set leading zeros");
    send_control(
        &mut device,
        &Command::SetLeadingZeros.with_payload(&msrx6.leading_zero_packets()),
    );
    read_success(&mut device);

    // println!("Get voltage");
    // send_control(&mut device, &Command::GetVoltage.packets());
    // let voltage = read_voltage(&mut device);
    // println!("Voltage: {}", voltage);

    // println!("Get model");
    // send_control(&mut device, &Command::GetDeviceModel.packets());
    // let model = read_data(&mut device);
    // println!("model: {}", model);

    println!("Enable reading");
    send_control(&mut device, &Command::SetReadModeOn.packets());
    let data = read_data(&mut device);
    println!("data: {}", data);
    println!("Disable reading");
    send_control(&mut device, &Command::SetReadModeOff.packets());
    read_success(&mut device);

    // enable_read(&mut device);

    // // wait 2 seconds

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

fn read_firmware(device: &mut DeviceHandle<Context>) -> String {
    let data = read_data(device);
    return data[1..].to_string();
}

fn read_voltage(device: &mut DeviceHandle<Context>) -> f64 {
    let raw_voltage = read_interrupt(device);
    let rounded_voltage =
        ((raw_voltage[0] as f64 + (raw_voltage[1] as f64 / 255.0)) * 9.9 / 128.0 * 100.0).round()
            / 100.0;
    return rounded_voltage;
}
fn read_data(device: &mut DeviceHandle<Context>) -> String {
    let hex_data = read_interrupt(device);

    if (hex_data.len() == 0) {
        return "".to_string();
    }

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

fn read_success(device: &mut DeviceHandle<Context>) -> bool {
    let hex_data = read_interrupt(device);
    println!("hex_data: {:?}", hex_data);

    // First byte is the length of the data
    // so skipping it
    return hex_data[1] == 0x1b && hex_data[2] == 0x30;
}

fn read_interrupt(device: &mut DeviceHandle<Context>) -> Vec<u8> {
    let mut inbuf: [u8; 64] = [0; 64];
    device.read_interrupt(0x81, &mut inbuf, Duration::from_secs(10));

    println!("inbuf: {:?}", inbuf);
    let len = inbuf[0] as usize;
    let header_length = 0x80;
    let buffer_length = 0x3f;
    println!("len: {:?}", len);
    println!("header_length: {:?}", header_length);
    println!("buffer_length: {:?}", buffer_length);
    let payload_length = len - header_length - buffer_length;

    println!("before: {:?}", inbuf);
    println!("after: {:?}", inbuf[0..payload_length].to_vec());
    println!("payload_length: {:?}", payload_length);
    if payload_length == 0 {
        return vec![];
    } else {
        return inbuf[0..payload_length].to_vec();
    }
}

fn send_control(device: &mut DeviceHandle<Context>, packets: &Vec<u8>) {
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
