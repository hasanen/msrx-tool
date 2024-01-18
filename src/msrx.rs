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
    fn send_device_control(&mut self, endpoint: u8, packets: &Vec<u8>)
        -> Result<(), MsrxToolError>;
    fn run_command(&mut self, endpoint: u8, command: &Command) -> Result<bool, MsrxToolError>;
    fn send_control_chunk(&mut self, endpoint: u8, chunk: &Vec<u8>) -> Result<(), MsrxToolError>;
    fn read_success(&mut self, endpoint: u8) -> Result<bool, MsrxToolError>;
}
impl MSRX for DeviceHandle<Context> {
    fn reset(&mut self, endpoint: u8) -> Result<bool, MsrxToolError> {
        self.run_command(endpoint, &Command::Reset)?;
        let result = self.read_success(endpoint)?;
        Ok(result)
    }
    // fn read_tracks(&mut self, endpoint: u8) -> String {
    //     let raw_data = self.read_device_interrupt(endpoint, 1).unwrap();
    //     let raw_track_data: TracksData = raw_data.try_into().unwrap();
    //     dbg!(raw_track_data);
    //     //
    //     return "".to_string();
    // }
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

    fn run_command(&mut self, endpoint: u8, command: &Command) -> Result<bool, MsrxToolError> {
        let packets = command.packets();
        self.send_device_control(endpoint, &packets)?;
        Ok(true)
    }

    fn send_device_control(
        &mut self,
        endpoint: u8,
        packets: &Vec<u8>,
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
            self.send_control_chunk(endpoint, &chunk)?;
        }
        Ok(())
    }

    fn send_control_chunk(&mut self, endpoint: u8, chunk: &Vec<u8>) -> Result<(), MsrxToolError> {
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
            self.device_handle
                .send_device_control(self.config.control_endpoint, &Command::SetHiCo.packets())?;
        } else {
            self.device_handle
                .send_device_control(self.config.control_endpoint, &Command::SetLoCo.packets())?;
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
        self.device_handle
            .run_command(self.config.control_endpoint, &Command::GetDeviceModel)?;
        let raw_device_data = self
            .device_handle
            .read_device_raw_interrupt(self.config.interrupt_endpoint, 1)?;
        Ok(raw_device_data.to_string())
    }

    pub fn read_tracks(&mut self, format: &DataFormat) -> Result<TracksData, MsrxToolError> {
        let read_command = match format {
            DataFormat::Iso => Command::SetReadModeOnFormatISO,
            DataFormat::Raw => return Err(MsrxToolError::UnsupportedDataFormatForReading),
        };

        self.device_handle
            .send_device_control(self.config.control_endpoint, &read_command.packets())?;

        let raw_datas = self.read_interrupts()?;

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

    pub fn get_firmware_version(&mut self) -> Result<String, MsrxToolError> {
        self.device_handle
            .run_command(self.config.control_endpoint, &Command::GetFirmwareVersion)?;
        let raw_device_data = self
            .device_handle
            .read_device_raw_interrupt(self.config.interrupt_endpoint, 1)?;
        let firmware = raw_device_data.to_string();
        Ok(firmware)
    }

    pub fn write_tracks(&mut self, data: &TracksData) -> Result<bool, MsrxToolError> {
        dbg!("moi");

        with_payload should probably return vector of packets? or some other method? would make sense that the command nows what to return
        let packet_chunks = &Command::SetISOReadModeOn.with_payload(&self.config.to_data_block()),


        let packets = self.to_packets(data, &Command)?;
        for packet in packets {
            dbg!(&packet);
            self.device_handle
                .send_device_control(self.config.control_endpoint, &packet)?;
            let raw_device_data = self
                .device_handle
                .read_device_raw_interrupt(self.config.interrupt_endpoint, 1)?;

            dbg!(raw_device_data);
        }

        Ok(true)
    }

    pub fn to_packets(&self, data: &TracksData) -> Result<Vec<Vec<u8>>, MsrxToolError> {
        let card_data = TRACK_1_START_FIELD
            .to_vec()
            .into_iter()
            .chain(self.track1.data.clone())
            .chain(TRACK_2_START_FIELD.to_vec())
            .chain(self.track2.data.clone())
            .chain(TRACK_3_START_FIELD.to_vec())
            .chain(self.track3.data.clone())
            .collect::<Vec<u8>>();

        let data_block = WRITE_BLOCK_START_FIELD
            .to_vec()
            .into_iter()
            .chain(card_data.clone())
            .chain(WRITE_BLOCK_END_FIELD.to_vec())
            .collect::<Vec<u8>>();

        let packet_datas: Vec<Vec<u8>> =
            data_block.chunks(63).map(|chunk| chunk.to_vec()).collect();

        let mut packets = vec![];

        for packet_data in packet_datas {
            let header_bit = 0x80;
            let mut packet_length = 0x3f;

            if packet_data.len() < 63 {
                packet_length = packet_data.len() as u8;
            }
            let first_packet = header_bit | packet_length;
            packets.push(
                std::iter::once(first_packet)
                    .chain(packet_data.iter().cloned())
                    .collect::<Vec<u8>>(),
            );
        }

        Ok(packets)
    }

    fn read_interrupts(&mut self) -> Result<Vec<OriginalDeviceData>, MsrxToolError> {
        let mut raw_datas = vec![];

        let device_data: OriginalDeviceData = self
            .device_handle
            .read_device_raw_interrupt(self.config.interrupt_endpoint, 10)?;

        raw_datas.push(device_data.clone());
        let mut is_last_packet = device_data.is_last_packet;
        while !is_last_packet {
            let raw_data = self
                .device_handle
                .read_device_raw_interrupt(self.config.interrupt_endpoint, 10)?;

            raw_datas.push(raw_data.clone());
            is_last_packet = raw_data.is_last_packet;
        }

        Ok(raw_datas)
    }
}
