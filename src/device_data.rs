use crate::data_format::DataFormat;
use crate::msrx_tool_error::MsrxToolError;
use crate::original_device_data::OriginalDeviceData;

#[derive(Debug, Copy, Clone)]
pub struct DeviceData {
    pub raw: OriginalDeviceData,
    pub format: DataFormat,
}

impl DeviceData {
    pub fn from_interrupt_data(
        data: [u8; 64],
        data_format: &DataFormat,
    ) -> Result<Self, MsrxToolError> {
        let is_header = data[0] & 0x80 != 0;
        let is_last_packet = data[0] & 0x40 != 0;

        Ok(DeviceData {
            raw: OriginalDeviceData {
                is_header,
                is_last_packet,
                data,
            },
            format: data_format.clone(),
        })
    }
}
