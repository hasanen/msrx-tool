use crate::command::Command;
use crate::config::DeviceConfig;
use crate::msrx_tool_error::MsrxToolError;
use crate::raw_device_data::RawDeviceData;
use crate::raw_tracks_data::RawTracksData;
use crate::to_hex::ToHex;
use rusb::{Context, DeviceHandle, UsbContext};
use std::time::Duration;

pub trait MSRX {
    fn reset(&mut self) -> Result<bool, MsrxToolError>;
    fn get_firmware_version(&mut self) -> Result<String, MsrxToolError>;
    fn read_tracks(&mut self) -> String;
    fn read_device_interrupt(&mut self, timeout: u64) -> Result<RawDeviceData, MsrxToolError>;
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
        let raw_device_data = self.read_device_interrupt(1)?;
        let firmware = raw_device_data.to_string();
        Ok(firmware)
    }
    fn read_tracks(&mut self) -> String {
        let raw_data = self.read_device_interrupt(1).unwrap();
        let raw_track_data: RawTracksData = raw_data.try_into().unwrap();
        dbg!(raw_track_data);
        //
        return "".to_string();
    }
    fn read_device_interrupt(&mut self, timeout: u64) -> Result<RawDeviceData, MsrxToolError> {
        let mut raw_data: [u8; 64] = [0; 64];
        let _ = self.read_interrupt(0x81, &mut raw_data, Duration::from_secs(timeout))?;

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
        let raw_device_data = self.read_device_interrupt(1)?;

        Ok(raw_device_data.successful_read())
    }
}

#[derive(Debug)]
pub struct MsrxDevice {
    pub device_handle: DeviceHandle<Context>,
    pub config: DeviceConfig,
    interface: u8,
}

impl MsrxDevice {
    pub fn init_msrx6() -> Result<MsrxDevice, MsrxToolError> {
        let config = DeviceConfig::msrx6();
        // Initialize a USB context
        let context = Context::new().expect("Failed to initialize USB context");

        let device_handle = context
            .open_device_with_vid_pid(config.vendor_id, config.product_id)
            .unwrap();

        Ok(MsrxDevice {
            device_handle,
            config,
            interface: 0,
        })
    }

    pub fn detach_kernel_driver(&mut self) -> Result<(), MsrxToolError> {
        if self.device_handle.kernel_driver_active(self.interface)? {
            self.device_handle.detach_kernel_driver(self.interface)?;
            Ok(())
        } else {
            println!("Kernel driver not active");
            Ok(())
        }
    }

    pub fn claim_interface(&mut self) -> Result<(), MsrxToolError> {
        self.device_handle.claim_interface(self.interface)?;
        Ok(())
    }

    pub fn release_interface(&mut self) -> Result<(), MsrxToolError> {
        let res = self.device_handle.release_interface(self.interface)?;
        dbg!(res);
        Ok(())
    }

    pub fn attach_kernel_driver(&mut self) -> Result<(), MsrxToolError> {
        let kernel_active = self.device_handle.kernel_driver_active(self.interface)?;
        if !kernel_active {
            match self.device_handle.attach_kernel_driver(self.interface) {
                Ok(_) => {
                    println!("Attached kernel driver");
                    Ok(())
                }
                Err(e) => {
                    println!("Error attaching kernel driver: {:?}", e);
                    Err(MsrxToolError::Unknown)
                }
            }
        } else {
            Ok(())
        }
    }

    pub fn set_bit_control_parity(&mut self) -> Result<(), MsrxToolError> {
        self.device_handle
            .send_device_control(&Command::SetBCP.with_payload(&self.config.bpc_packets()))?;
        let result = self.device_handle.read_device_interrupt(1)?;

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

    pub fn set_hico_loco_mode(&mut self) -> Result<(), MsrxToolError> {
        if self.config.is_hi_co {
            self.device_handle
                .send_device_control(&Command::SetHiCo.packets())?;
        } else {
            self.device_handle
                .send_device_control(&Command::SetLoCo.packets())?;
        }
        let result = self.device_handle.read_device_interrupt(1)?;

        if result.raw_data[1] == 0x1b && result.raw_data[2] == 0x30 {
            Ok(())
        } else {
            Err(MsrxToolError::Unknown)
        }
    }

    pub fn set_bit_per_inches(&mut self) -> Result<(), MsrxToolError> {
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
            let result = self.device_handle.read_device_interrupt(1)?;

            if result.did_failed() {
                return Err(MsrxToolError::ErrorSettingBPI(index + 1));
            }
        }

        Ok(())
    }

    pub fn set_leading_zeros(&mut self) -> Result<(), MsrxToolError> {
        self.device_handle.send_device_control(
            &Command::SetLeadingZeros.with_payload(&self.config.leading_zero_packets()),
        )?;
        let result = self.device_handle.read_device_interrupt(1)?;
        if result.did_failed() {
            return Err(MsrxToolError::ErrorSettingLeadingZeros);
        }
        Ok(())
    }

    pub fn get_model(&mut self) -> Result<String, MsrxToolError> {
        self.device_handle.run_command(&Command::GetDeviceModel)?;
        let raw_device_data = self.device_handle.read_device_interrupt(1)?;
        Ok(raw_device_data.to_string())
    }

    pub fn read_tracks(&mut self) -> Result<RawTracksData, MsrxToolError> {
        self.device_handle
            .send_device_control(&Command::SetReadModeOn.packets())?;
        let raw_data = self.device_handle.read_device_interrupt(10)?;
        dbg!(&raw_data.raw_data.to_hex());
        let mut is_last_packet = raw_data.is_last_packet;
        while !is_last_packet {
            let raw_data = self.device_handle.read_device_interrupt(10)?;
            dbg!(&raw_data.raw_data.to_hex());
            is_last_packet = raw_data.is_last_packet;
        }
        let raw_track_data: RawTracksData = raw_data.try_into().unwrap();

        Ok(raw_track_data)
    }
}
