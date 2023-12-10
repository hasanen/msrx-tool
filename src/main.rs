use hex::FromHex;
use rusb::{Context, DeviceHandle, UsbContext};
use std::process;
use std::time::Duration;

mod config;
mod msrx_tool_error;
mod raw_device_data;
mod raw_tracks_data;
mod track_data;
mod track_status;
use config::DeviceConfig;
use msrx_tool_error::MsrxToolError;
use raw_device_data::RawDeviceData;
use raw_tracks_data::RawTracksData;

enum Command {
    Reset,
    GetFirmwareVersion,
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
struct MsrxDevice {
    device_handle: DeviceHandle<Context>,
    config: DeviceConfig,
    kernel_detached: bool,
    interface: u8,
}

impl MsrxDevice {
    fn init_msrx6() -> MsrxDevice {
        let config = DeviceConfig::msrx6();
        // Initialize a USB context
        let context = Context::new().expect("Failed to initialize USB context");

        let device_handle =
            match context.open_device_with_vid_pid(config.vendor_id, config.product_id) {
                Some(device) => device,
                None => {
                    println!("Device not found");
                    process::exit(1)
                }
            };

        MsrxDevice {
            device_handle,
            config,
            kernel_detached: false,
            interface: 0,
        }
    }

    fn detach_kernel_driver(&mut self) -> Result<(), MsrxToolError> {
        if self.device_handle.kernel_driver_active(self.interface)? {
            println!("Kernel driver active");
            self.kernel_detached = true;
            self.device_handle.detach_kernel_driver(self.interface)?;
            Ok(())
        } else {
            println!("Kernel driver not active");
            Ok(())
        }
    }

    fn claim_interface(&mut self) -> Result<(), MsrxToolError> {
        self.device_handle.claim_interface(self.interface)?;
        Ok(())
    }

    fn release_interface(&mut self) -> Result<(), MsrxToolError> {
        self.device_handle.release_interface(self.interface)?;
        Ok(())
    }

    fn attach_kernel_driver(&mut self) -> Result<(), MsrxToolError> {
        dbg!(self.kernel_detached);
        if self.kernel_detached {
            self.device_handle
                .attach_kernel_driver(self.interface)
                .unwrap();
        }
        Ok(())
    }

    fn set_bit_control_parity(&mut self) -> Result<(), MsrxToolError> {
        self.device_handle
            .send_device_control(&Command::SetBCP.with_payload(&self.config.bpc_packets()))?;
        let result = self.device_handle.read_device_interrupt()?;

        if result.raw_data[1] == 0x1b
            && result.raw_data[2] == 0x30
            && result.raw_data[3] == self.config.track1.bpc
            && result.raw_data[4] == self.config.track2.bpc
            && result.raw_data[5] == self.config.track3.bpc
        {
            Ok(())
        } else {
            Err(MsrxToolError::Unknown)
        }
    }

    fn set_hico_loco_mode(&mut self) -> Result<(), MsrxToolError> {
        if self.config.is_hi_co {
            self.device_handle
                .send_device_control(&Command::SetHiCo.packets())?;
        } else {
            self.device_handle
                .send_device_control(&Command::SetLoCo.packets())?;
        }
        let result = self.device_handle.read_device_interrupt()?;

        if result.raw_data[1] == 0x1b && result.raw_data[2] == 0x30 {
            Ok(())
        } else {
            Err(MsrxToolError::Unknown)
        }
    }

    fn set_bit_per_inches(&mut self) -> Result<(), MsrxToolError> {
        for (index, packets) in [
            &self.config.track1.bpi_packets(),
            &self.config.track2.bpi_packets(),
            &self.config.track3.bpi_packets(),
        ]
        .iter()
        .enumerate()
        {
            self.device_handle
                .send_device_control(&Command::SetBPI.with_payload(&packets))?;
            let result = self.device_handle.read_device_interrupt()?;

            if result.did_failed() {
                return Err(MsrxToolError::ErrorSettingBPI(index + 1));
            }
        }

        Ok(())
    }

    fn set_leading_zeros(&mut self) -> Result<(), MsrxToolError> {
        self.device_handle.send_device_control(
            &Command::SetLeadingZeros.with_payload(&self.config.leading_zero_packets()),
        )?;
        let result = self.device_handle.read_device_interrupt()?;
        if result.did_failed() {
            return Err(MsrxToolError::ErrorSettingLeadingZeros);
        }
        Ok(())
    }

    fn get_model(&mut self) -> Result<String, MsrxToolError> {
        self.device_handle.run_command(&Command::GetDeviceModel)?;
        let raw_device_data = self.device_handle.read_device_interrupt()?;
        Ok(raw_device_data.to_string())
    }
}

#[tokio::main]
async fn main() {
    let mut msrx_device = MsrxDevice::init_msrx6();

    dbg!(&msrx_device);

    let _ = msrx_device.detach_kernel_driver();
    let _ = msrx_device.claim_interface();

    println!("Reset device");
    msrx_device.device_handle.reset().unwrap();

    println!("read firmware");
    let firmware = msrx_device.device_handle.get_firmware_version().unwrap();
    println!("Firmware: {}", firmware);

    println!("Set BPC");
    let _ = msrx_device.set_bit_control_parity();

    println!("Set HiCo/LoCo mode");
    let _ = msrx_device.set_hico_loco_mode();

    println!("Set BPI");
    let _ = msrx_device.set_bit_per_inches();

    println!("Set leading zeros");
    let _ = msrx_device.set_leading_zeros();

    println!("Get model");
    let model = msrx_device.get_model().unwrap();
    println!("Firmware: {}", model);

    // println!("Enable reading");
    // send_control(&mut device, &Command::SetReadModeOn.packets());
    // let _ = device.read_tracks();
    // println!("Disable reading");
    // send_control(&mut device, &Command::SetReadModeOff.packets());
    // let _ = read_success(&mut device);

    // enable_read(&mut device);

    // // wait 2 seconds

    // let data = read_data(&mut device);
    // print!("{}", data);

    // disable_read(&mut device);

    let _ = msrx_device.release_interface();
    let _ = msrx_device.attach_kernel_driver();
}
trait MSRX {
    fn reset(&mut self) -> Result<bool, MsrxToolError>;
    fn get_firmware_version(&mut self) -> Result<String, MsrxToolError>;
    fn read_tracks(&mut self) -> String;
    fn read_device_interrupt(&mut self) -> Result<RawDeviceData, MsrxToolError>;
    fn send_device_control(&mut self, packets: &Vec<u8>) -> Result<(), MsrxToolError>;
    fn run_command(&mut self, command: &Command) -> Result<bool, MsrxToolError>;
    fn send_control_chunk(&mut self, chunk: &Vec<u8>) -> Result<(), MsrxToolError>;
    fn read_success(&mut self) -> Result<bool, MsrxToolError>;
}
impl MSRX for DeviceHandle<Context> {
    fn reset(&mut self) -> Result<bool, MsrxToolError> {
        self.run_command(&Command::Reset)?;
        let result = self.read_success()?;
        Ok(result)
    }
    fn get_firmware_version(&mut self) -> Result<String, MsrxToolError> {
        self.run_command(&Command::GetFirmwareVersion)?;
        let raw_device_data = self.read_device_interrupt()?;
        let firmware = raw_device_data.to_string();
        Ok(firmware)
    }
    fn read_tracks(&mut self) -> String {
        let raw_data = self.read_device_interrupt().unwrap();
        let raw_track_data: RawTracksData = raw_data.try_into().unwrap();
        dbg!(raw_track_data);
        //
        return "".to_string();
    }
    fn read_device_interrupt(&mut self) -> Result<RawDeviceData, MsrxToolError> {
        let mut raw_data: [u8; 64] = [0; 64];
        let _ = self.read_interrupt(0x81, &mut raw_data, Duration::from_secs(10))?;

        raw_data.try_into()
    }

    fn run_command(&mut self, command: &Command) -> Result<bool, MsrxToolError> {
        let packets = command.packets();
        self.send_device_control(&packets)?;
        Ok(true)
    }

    fn send_device_control(&mut self, packets: &Vec<u8>) -> Result<(), MsrxToolError> {
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
            self.send_control_chunk(&chunk)?;
        }
        Ok(())
    }

    fn send_control_chunk(&mut self, chunk: &Vec<u8>) -> Result<(), MsrxToolError> {
        let _ = self.write_control(0x21, 9, 0x0300, 0, &chunk, Duration::from_secs(10))?;
        Ok(())
    }

    fn read_success(&mut self) -> Result<bool, MsrxToolError> {
        let hex_data = read_interrupt(self)?;
        println!("hex_data: {:?}", hex_data);

        // First byte is the length of the data
        // so skipping it
        Ok(hex_data[1] == 0x1b && hex_data[2] == 0x30)
    }
}

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
