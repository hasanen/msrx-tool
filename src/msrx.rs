use crate::command::Command;
use crate::config::DeviceConfig;
use crate::data_format::DataFormat;
use crate::device_data::DeviceData;
use crate::iso_data::IsoData;
use crate::msrx_tool_error::MsrxToolError;
use crate::original_device_data::OriginalDeviceData;
use crate::to_hex::ToHex;
use crate::tracks_data::TracksData;
use rusb::{Context, DeviceHandle, Direction, Recipient, RequestType, UsbContext};
use std::time::Duration;

pub trait MSRX {
    fn reset(&mut self, endpoint: u8) -> Result<bool, MsrxToolError>;
    // fn read_tracks(&mut self, endpoint: u8) -> String;
    fn read_device_interrupt(
        &mut self,
        endpoint: u8,
        format: &DataFormat,
        timeout: u64,
    ) -> Result<DeviceData, MsrxToolError>;
    fn read_device_raw_interrupt(
        &mut self,
        endpoint: u8,
        timeout: u64,
    ) -> Result<OriginalDeviceData, MsrxToolError>;
    fn send_device_control(
        &mut self,
        endpoint: u8,
        packets: &Vec<u8>,
        timeout: &Duration,
    ) -> Result<(), MsrxToolError>;
    fn run_command(
        &mut self,
        endpoint: u8,
        command: &Command,
        timeout: &Duration,
    ) -> Result<bool, MsrxToolError>;
    fn send_control_chunk(
        &mut self,
        endpoint: u8,
        chunk: &Vec<u8>,
        timeout: &Duration,
    ) -> Result<(), MsrxToolError>;
    fn read_success(&mut self, endpoint: u8) -> Result<bool, MsrxToolError>;
}
impl MSRX for DeviceHandle<Context> {
    fn reset(&mut self, endpoint: u8) -> Result<bool, MsrxToolError> {
        self.run_command(endpoint, &Command::Reset, &Duration::from_secs(1))?;
        let result = self.read_success(endpoint)?;
        Ok(result)
    }

    fn read_device_interrupt(
        &mut self,
        endpoint: u8,
        format: &DataFormat,
        timeout: u64,
    ) -> Result<DeviceData, MsrxToolError> {
        let mut raw_data: [u8; 64] = [0; 64];
        let _ = self.read_interrupt(endpoint, &mut raw_data, Duration::from_secs(timeout))?;

        DeviceData::from_interrupt_data(raw_data, &format)
    }

    fn read_device_raw_interrupt(
        &mut self,
        endpoint: u8,
        timeout: u64,
    ) -> Result<OriginalDeviceData, MsrxToolError> {
        let mut raw_data: [u8; 64] = [0; 64];
        let _ = self.read_interrupt(endpoint, &mut raw_data, Duration::from_secs(timeout))?;

        raw_data.try_into()
    }

    fn run_command(
        &mut self,
        endpoint: u8,
        command: &Command,
        timeout: &Duration,
    ) -> Result<bool, MsrxToolError> {
        let packets = command.packets();
        self.send_device_control(endpoint, &packets, timeout)?;
        dbg!("comand sent");
        Ok(true)
    }

    fn send_device_control(
        &mut self,
        endpoint: u8,
        packets: &Vec<u8>,
        timeout: &Duration,
    ) -> Result<(), MsrxToolError> {
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

            self.send_control_chunk(endpoint, &chunk, &timeout)?;
        }
        Ok(())
    }

    fn send_control_chunk(
        &mut self,
        endpoint: u8,
        chunk: &Vec<u8>,
        timeout: &Duration,
    ) -> Result<(), MsrxToolError> {
        let _ = self.write_control(
            0x21,
            9,
            0x0300,
            endpoint as u16,
            &chunk,
            Duration::from_secs(10),
        )?;
        Ok(())
    }

    fn read_success(&mut self, endpoint: u8) -> Result<bool, MsrxToolError> {
        let raw_device_data = self.read_device_raw_interrupt(endpoint, 1)?;

        Ok(raw_device_data.successful_operation())
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

        match context.open_device_with_vid_pid(config.vendor_id, config.product_id) {
            Some(device_handle) => Ok(MsrxDevice {
                device_handle,
                config,
                interface: 0,
            }),
            None => Err(MsrxToolError::DeviceNotFound),
        }
    }

    pub fn setup_device(&mut self) -> Result<(), MsrxToolError> {
        self.device_handle.set_auto_detach_kernel_driver(true)?;

        self.claim_interface()?;

        // Device setup
        self.device_handle.reset()?;
        self.set_bit_control_parity()?;
        self.set_hico_loco_mode()?;
        self.set_bit_per_inches()?;
        self.set_leading_zeros()?;

        Ok(())
    }

    pub fn detach_kernel_driver(&mut self) -> Result<(), MsrxToolError> {
        if self.device_handle.kernel_driver_active(self.interface)? {
            self.device_handle.detach_kernel_driver(self.interface)?;
            Ok(())
        } else {
            Ok(())
        }
    }

    pub fn claim_interface(&mut self) -> Result<(), MsrxToolError> {
        self.device_handle.claim_interface(self.interface)?;
        Ok(())
    }

    pub fn release_interface(&mut self) -> Result<(), MsrxToolError> {
        let _ = self.device_handle.release_interface(self.interface)?;
        Ok(())
    }

    pub fn attach_kernel_driver(&mut self) -> Result<(), MsrxToolError> {
        let kernel_active = self.device_handle.kernel_driver_active(self.interface)?;
        if !kernel_active {
            match self.device_handle.attach_kernel_driver(self.interface) {
                Ok(_) => Ok(()),
                Err(e) => {
                    println!("Error attaching kernel driver: {:?}", e);
                    Err(MsrxToolError::Unknown)
                }
            }
        } else {
            println!("Kernel driver already active");
            Ok(())
        }
    }

    pub fn set_bit_control_parity(&mut self) -> Result<(), MsrxToolError> {
        self.device_handle.send_device_control(
            self.config.control_endpoint,
            &Command::SetBCP.with_payload(&self.config.bpc_packets()),
            &Duration::from_secs(1),
        )?;
        let result = self
            .device_handle
            .read_device_raw_interrupt(self.config.interrupt_endpoint, 1)?;

        if result.data[1] == 0x1b
            && result.data[2] == 0x30
            && result.data[3] == self.config.track1.bpc
            && result.data[4] == self.config.track2.bpc
            && result.data[5] == self.config.track3.bpc
        {
            Ok(())
        } else {
            Err(MsrxToolError::Unknown)
        }
    }

    pub fn set_hico_loco_mode(&mut self) -> Result<(), MsrxToolError> {
        if self.config.is_hi_co {
            self.device_handle.send_device_control(
                self.config.control_endpoint,
                &Command::SetHiCo.packets(),
                &Duration::from_secs(1),
            )?;
        } else {
            self.device_handle.send_device_control(
                self.config.control_endpoint,
                &Command::SetLoCo.packets(),
                &Duration::from_secs(1),
            )?;
        }
        let result = self
            .device_handle
            .read_device_raw_interrupt(self.config.interrupt_endpoint, 1)?;

        if result.data[1] == 0x1b && result.data[2] == 0x30 {
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
            self.device_handle.send_device_control(
                self.config.control_endpoint,
                &Command::SetBPI.with_payload(&packets),
                &Duration::from_secs(1),
            )?;
            let result = self
                .device_handle
                .read_device_raw_interrupt(self.config.interrupt_endpoint, 1)?;

            if result.did_failed() {
                return Err(MsrxToolError::ErrorSettingBPI(index + 1));
            }
        }

        Ok(())
    }

    pub fn set_leading_zeros(&mut self) -> Result<(), MsrxToolError> {
        self.device_handle.send_device_control(
            self.config.control_endpoint,
            &Command::SetLeadingZeros.with_payload(&self.config.leading_zero_packets()),
            &Duration::from_secs(1),
        )?;
        let result = self
            .device_handle
            .read_device_raw_interrupt(self.config.interrupt_endpoint, 1)?;
        if result.did_failed() {
            return Err(MsrxToolError::ErrorSettingLeadingZeros);
        }
        Ok(())
    }

    pub fn get_model(&mut self) -> Result<String, MsrxToolError> {
        self.device_handle.run_command(
            self.config.control_endpoint,
            &Command::GetDeviceModel,
            &Duration::from_secs(1),
        )?;
        let raw_device_data = self
            .device_handle
            .read_device_raw_interrupt(self.config.interrupt_endpoint, 1)?;
        Ok(raw_device_data.to_string())
    }

    pub fn disable_read_mode(&mut self) -> Result<bool, MsrxToolError> {
        self.device_handle.run_command(
            self.config.control_endpoint,
            &Command::SetReadModeOff,
            &Duration::from_secs(1),
        )?;
        let raw_device_data = self
            .device_handle
            .read_device_raw_interrupt(self.config.interrupt_endpoint, 1)?;
        Ok(raw_device_data.successful_operation())
    }

    pub fn read_tracks(
        &mut self,
        format: &DataFormat,
        timeout: &Duration,
    ) -> Result<TracksData, MsrxToolError> {
        let read_command = match format {
            DataFormat::Iso => Command::SetReadModeOnFormatISO,
            DataFormat::Raw => return Err(MsrxToolError::UnsupportedDataFormatForReading),
        };
        dbg!("eka");
        self.device_handle.send_device_control(
            self.config.control_endpoint,
            &read_command.packets(),
            timeout,
        )?;
        dbg!("toka");

        match self.read_interrupts(timeout) {
            Ok(raw_datas) => {
                let tracks_data = match format {
                    DataFormat::Iso => raw_datas
                        .iter()
                        .map(|rd| IsoData { raw: rd.clone() })
                        .collect::<Vec<IsoData>>()
                        .try_into()?,
                    DataFormat::Raw => return Err(MsrxToolError::UnsupportedDataFormatForReading),
                };

                Ok(tracks_data)
            }
            Err(e) => match e {
                MsrxToolError::DeviceError(rusb::Error::Timeout) => {
                    self.disable_read_mode();
                    Err(MsrxToolError::CardNotSwiped)
                }
                _ => Err(MsrxToolError::Unknown),
            },
        }
    }

    pub fn get_firmware_version(&mut self) -> Result<String, MsrxToolError> {
        self.device_handle.run_command(
            self.config.control_endpoint,
            &Command::GetFirmwareVersion,
            &Duration::from_secs(1),
        )?;
        let raw_device_data = self
            .device_handle
            .read_device_raw_interrupt(self.config.interrupt_endpoint, 1)?;
        let firmware = raw_device_data.to_string();
        Ok(firmware)
    }

    pub fn write_tracks(&mut self, data: &TracksData) -> Result<bool, MsrxToolError> {
        let payload = &Command::SetISOReadModeOn.with_payload(&data.to_data_block()?);

        self.device_handle.send_device_control(
            self.config.control_endpoint,
            &payload,
            &Duration::from_secs(1),
        )?;
        let raw_device_data = self
            .device_handle
            .read_device_raw_interrupt(self.config.interrupt_endpoint, 10)?;

        Ok(raw_device_data.successful_operation())
    }

    fn read_interrupts(
        &mut self,
        timeout: &Duration,
    ) -> Result<Vec<OriginalDeviceData>, MsrxToolError> {
        let mut raw_datas = vec![];

        let device_data: OriginalDeviceData = self
            .device_handle
            .read_device_raw_interrupt(self.config.interrupt_endpoint, timeout.as_secs())?;

        raw_datas.push(device_data.clone());
        let mut is_last_packet = device_data.is_last_packet;
        while !is_last_packet {
            let raw_data = self
                .device_handle
                .read_device_raw_interrupt(self.config.interrupt_endpoint, timeout.as_secs())?;

            raw_datas.push(raw_data.clone());
            is_last_packet = raw_data.is_last_packet;
        }

        Ok(raw_datas)
    }
}
