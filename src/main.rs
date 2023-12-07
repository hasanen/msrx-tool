use hex::FromHex;
use rusb::{Context, Device, DeviceHandle, DeviceList, UsbContext};
use rusb::{Direction, Recipient, RequestType};
use std::process;
use std::time::Duration;
use thiserror::Error;
use tokio::io::AsyncReadExt;
use tokio::time::sleep;

#[derive(Error, Debug)]
pub enum MsrxToolError {
    #[error("device error")]
    DeviceError(#[from] rusb::Error),
    #[error("Raw data was not card data")]
    RawDataNotCardData,
    #[error("unknown conversion error")]
    Unknown,
}
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
    TurnLedAllOn,
    TurnLedRedOn,
    TurnLedGreenOn,
    TurnLedYellowOn,
    TurnLedAllOff,
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
            Command::TurnLedAllOn => vec![0x1b, 0x82],
            Command::TurnLedRedOn => vec![0x1b, 0x85],
            Command::TurnLedGreenOn => vec![0x1b, 0x83],
            Command::TurnLedYellowOn => vec![0x1b, 0x84],
            Command::TurnLedAllOff => vec![0x1b, 0x81],
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

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
enum Status {
    Ok,
    WriteOrReadError,
    CommandFormatError,
    InvalidCommand,
    InvalidCardSwipeOnWrite,
    Unknown,
}

impl From<u8> for Status {
    fn from(value: u8) -> Self {
        match value {
            0x30 => Status::Ok,
            0x31 => Status::WriteOrReadError,
            0x32 => Status::CommandFormatError,
            0x34 => Status::InvalidCommand,
            0x39 => Status::InvalidCardSwipeOnWrite,
            _ => Status::Unknown,
        }
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
    if device.claim_interface(iface).is_err() {
        println!("Error claiming interface");
        process::exit(1)
    }

    println!("Reset device");
    send_control(&mut device, &Command::Reset.packets());
    // println!("read firmware");
    // send_control(&mut device, &Command::GetFirmwareVersion.packets());
    // let firmware = read_firmware(&mut device);
    // println!("Firmware: {}", firmware);

    // // Set Bit Control Parity (BCP)
    // send_control(
    //     &mut device,
    //     &Command::SetBCP.with_payload(&msrx6.bpc_packets()),
    // );
    // read_success(&mut device);

    // if (msrx6.is_hi_co) {
    //     send_control(&mut device, &Command::SetHiCo.packets());
    // } else {
    //     send_control(&mut device, &Command::SetLoCo.packets());
    // }
    // read_success(&mut device);

    // println!("Set BPI track1");
    // send_control(
    //     &mut device,
    //     &Command::SetBPI.with_payload(&msrx6.track1.bpi_packets()),
    // );
    // read_success(&mut device);

    // println!("Set BPI track2");
    // send_control(
    //     &mut device,
    //     &Command::SetBPI.with_payload(&msrx6.track2.bpi_packets()),
    // );
    // read_success(&mut device);

    // println!("Set BPI track3");
    // send_control(
    //     &mut device,
    //     &Command::SetBPI.with_payload(&msrx6.track3.bpi_packets()),
    // );
    // read_success(&mut device);
    // println!("Set leading zeros");
    // send_control(
    //     &mut device,
    //     &Command::SetLeadingZeros.with_payload(&msrx6.leading_zero_packets()),
    // );
    // read_success(&mut device);

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
    read_tracks(&mut device);
    println!("Disable reading");
    send_control(&mut device, &Command::SetReadModeOff.packets());
    let _ = read_success(&mut device);

    // enable_read(&mut device);

    // // wait 2 seconds

    // let data = read_data(&mut device);
    // print!("{}", data);

    // disable_read(&mut device);

    if device.release_interface(iface).is_err() {
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
    let raw_voltage = read_interrupt(device).unwrap();
    let rounded_voltage =
        ((raw_voltage[0] as f64 + (raw_voltage[1] as f64 / 255.0)) * 9.9 / 128.0 * 100.0).round()
            / 100.0;
    return rounded_voltage;
}
fn read_tracks(device: &mut DeviceHandle<Context>) -> String {
    let raw_track_data = read_interrupt(device).unwrap();
    println!("raw_track_data: {:?}", raw_track_data.to_hex());
    println!("raw_track_data.len: {:?}", raw_track_data.len());
    let rawtrackdata: RawTrackData = raw_track_data.try_into().unwrap();
    dbg!(rawtrackdata);

    if raw_track_data[1] != 0x1b || raw_track_data[2] != 0x73 {
        println!("Invalid data");
        return "".to_string();
    }
    // println!("hex_data[0]: {:?}", hex_data[0]);

    let len = raw_track_data[0] as usize;
    println!("len: {:?}", len);
    let mut read_index = 3;
    for i in 1..=3 {
        println!("TRACK: {:?}", i);
        if raw_track_data[read_index] != 0x1b || raw_track_data[read_index + 1] != i {
            println!("Invalid data");
            println!(
                "raw_track_data[read_index]: {:?}",
                raw_track_data[read_index]
            );
            println!(
                "raw_track_data[read_index + 1]: {:?}",
                raw_track_data[read_index + 1]
            );
            println!("i: {:?}", i);
            return "".to_string();
        }
        read_index += 2;
        let track_len = raw_track_data[read_index] as usize;
        println!("track_len: {:?}", track_len);
        read_index += 1;
        let track_data = &raw_track_data[read_index..read_index + track_len];
        println!("track_data: {:?}", track_data);
        read_index += track_len;
    }
    //
    return "".to_string();
}
fn read_data(device: &mut DeviceHandle<Context>) -> String {
    let hex_data = read_interrupt(device).unwrap();

    if hex_data.len() == 0 {
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

fn read_success(device: &mut DeviceHandle<Context>) -> Result<bool, MsrxToolError> {
    let hex_data = read_interrupt(device)?;
    println!("hex_data: {:?}", hex_data);

    // First byte is the length of the data
    // so skipping it
    Ok(hex_data[1] == 0x1b && hex_data[2] == 0x30)
}

fn read_interrupt(device: &mut DeviceHandle<Context>) -> Result<[u8; 64], MsrxToolError> {
    let mut inbuf: [u8; 64] = [0; 64];
    let result = device.read_interrupt(0x81, &mut inbuf, Duration::from_secs(10))?;
    Ok(inbuf)
}
fn read_bulk(device: &mut DeviceHandle<Context>) -> [u8; 200] {
    let mut inbuf: [u8; 200] = [0; 200];
    device.read_bulk(0x81, &mut inbuf, Duration::from_secs(10));
    return inbuf;
}

fn send_control(device: &mut DeviceHandle<Context>, packets: &Vec<u8>) {
    let mut written = 0;
    let incoming_packet_length = packets.len();

    while written < incoming_packet_length {
        let mut header = 128;
        let mut packet_length = 63;

        if incoming_packet_length - written < packet_length {
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

    let result = device.write_control(0x21, 9, 0x0300, 0, &chunk, Duration::from_secs(10));
    dbg!(result);
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
trait ToHex {
    fn to_hex(&self) -> String;
}

impl<T> ToHex for T
where
    T: AsRef<[u8]>,
{
    fn to_hex(&self) -> String {
        self.as_ref()
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect::<Vec<_>>()
            .join(" ")
    }
}

#[derive(Debug)]
struct RawTrackData {
    is_header: bool,
    is_last_packet: bool,
    raw_data: Vec<u8>,
    track1: Vec<u8>,
    track2: Vec<u8>,
    track3: Vec<u8>,
    status: Status,
}
impl TryFrom<[u8; 64]> for RawTrackData {
    type Error = MsrxToolError;

    fn try_from(value: [u8; 64]) -> Result<Self, Self::Error> {
        if value[1] != 0x1b || value[2] != 0x73 {
            return Err(MsrxToolError::RawDataNotCardData);
        }
        let mut data_length = value[0];
        let is_header = data_length & 0x80 != 0;
        let is_last_packet = data_length & 0x40 != 0;
        if is_header && is_last_packet {
            data_length &= !(0x80 | 0x40);
        }
        let raw_data = value[1..data_length as usize + 1].to_vec();

        dbg!(&raw_data);

        let mut read_index = 2;
        let mut tracks: Vec<Vec<u8>> = vec![];
        for i in 1..=3 {
            if raw_data[read_index] != 0x1b || raw_data[read_index + 1] != i {
                return Err(MsrxToolError::RawDataNotCardData);
            }
            read_index += 2;
            let track_len = raw_data[read_index] as usize;
            read_index += 1;
            let track_data = raw_data[read_index..read_index + track_len].to_vec();
            tracks.push(track_data);
            read_index += track_len;
        }
        dbg!(read_index);
        dbg!(tracks);
        // Confirm the ending sequence 3F 1C 1B
        if raw_data[read_index] != 0x3f
            || raw_data[read_index + 1] != 0x1c
            || raw_data[read_index + 2] != 0x1b
        {
            return Err(MsrxToolError::RawDataNotCardData);
        }
        read_index += 3;
        let status = Status::from(raw_data[read_index]);

        Ok(RawTrackData {
            raw_data,
            is_header,
            is_last_packet,
            track1: vec![],
            track2: vec![],
            track3: vec![],
            status,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod raw_track_data_statuses {
        use super::*;

        #[test]
        fn test_convert_raw_data_to_raw_track_data_status_ok() -> Result<(), MsrxToolError> {
            // Track 1 and Track 2 doesn't contain ant data, Track  3 data is: "1"
            let raw_data = b"\xd3\x1b\x73\x1b\x01\x00\x1b\x02\x00\x1b\x03\x04\xaf\xc2\xb0\x00\x3f\x1c\x1b\x30\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00";

            let raw_track_data: RawTrackData = (*raw_data).try_into()?;

            assert_eq!(raw_track_data.status, Status::Ok);
            Ok(())
        }

        #[test]
        fn test_convert_raw_data_to_raw_track_data_status_write_or_read_error(
        ) -> Result<(), MsrxToolError> {
            // Track 1 and Track 2 doesn't contain ant data, Track  3 data is: "1"
            let raw_data =
                b"\xd3\x1b\x73\x1b\x01\x00\x1b\x02\x00\x1b\x03\x04\xaf\xc2\xb0\x00\x3f\x1c\x1b\x31\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00";

            let raw_track_data: RawTrackData = (*raw_data).try_into()?;

            assert_eq!(raw_track_data.status, Status::WriteOrReadError);
            Ok(())
        }

        #[test]
        fn test_convert_raw_data_to_raw_track_data_status_command_format_error(
        ) -> Result<(), MsrxToolError> {
            // Track 1 and Track 2 doesn't contain ant data, Track  3 data is: "1"
            let raw_data =
                b"\xd3\x1b\x73\x1b\x01\x00\x1b\x02\x00\x1b\x03\x04\xaf\xc2\xb0\x00\x3f\x1c\x1b\x32\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00";

            let raw_track_data: RawTrackData = (*raw_data).try_into()?;

            assert_eq!(raw_track_data.status, Status::CommandFormatError);
            Ok(())
        }

        #[test]
        fn test_convert_raw_data_to_raw_track_data_status_invalid_command(
        ) -> Result<(), MsrxToolError> {
            // Track 1 and Track 2 doesn't contain ant data, Track  3 data is: "1"
            let raw_data =
                b"\xd3\x1b\x73\x1b\x01\x00\x1b\x02\x00\x1b\x03\x04\xaf\xc2\xb0\x00\x3f\x1c\x1b\x34\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00";

            let raw_track_data: RawTrackData = (*raw_data).try_into()?;

            assert_eq!(raw_track_data.status, Status::InvalidCommand);
            Ok(())
        }

        #[test]
        fn test_convert_raw_data_to_raw_track_data_status_invalid_card_swipe(
        ) -> Result<(), MsrxToolError> {
            // Track 1 and Track 2 doesn't contain ant data, Track  3 data is: "1"
            let raw_data =
                b"\xd3\x1b\x73\x1b\x01\x00\x1b\x02\x00\x1b\x03\x04\xaf\xc2\xb0\x00\x3f\x1c\x1b\x39\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00";

            let raw_track_data: RawTrackData = (*raw_data).try_into()?;

            assert_eq!(raw_track_data.status, Status::InvalidCardSwipeOnWrite);
            Ok(())
        }
    }

    //TODO do tests for covering cases when tracks are none (manual page 11)
    // mod raw_track_data_tracks {
    //     fn test_convert_raw_data_to_raw_track_data_all_tracks_empty() -> Result<(), MsrxToolError> {
    //         // Track 1 and Track 2 doesn't contain ant data, Track  3 data is: "1"
    //         let raw_data = b"\xd3\x1b\x73\x1b\x01\x1b\x02\x1b\x03\x04\xaf\xc2\xb0\x00\x3f\x1c\x1b\x30\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00";

    //         let raw_track_data: RawTrackData = (*raw_data).try_into()?;

    //         assert_eq!(raw_track_data.track1, vec![]);
    //         assert_eq!(raw_track_data.track2, vec![]);
    //         assert_eq!(raw_track_data.track3, vec!["1"]);
    //         Ok(())
    //     }
    // }
}
