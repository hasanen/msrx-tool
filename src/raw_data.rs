use crate::msrx_tool_error::MsrxToolError;

#[derive(Debug, Copy, Clone)]
pub struct RawData {
    pub is_header: bool,
    pub is_last_packet: bool,
    pub raw_data: [u8; 64],
}

impl RawData {
    pub fn stripped_data(&self) -> Vec<u8> {
        let length = self.raw_data[0] & !(0x80 | 0x40);
        self.raw_data[2..1 + length as usize].to_vec()
    }

    pub fn did_failed(&self) -> bool {
        self.raw_data[1] == 0x1b && self.raw_data[2] == 0x31
    }

    pub fn successful_read(&self) -> bool {
        // First byte is the length of the data
        // so skipping it
        self.raw_data[1] == 0x1b && self.raw_data[2] == 0x30
    }
}

impl TryFrom<[u8; 64]> for RawData {
    type Error = MsrxToolError;

    fn try_from(raw_data: [u8; 64]) -> Result<Self, Self::Error> {
        let is_header = raw_data[0] & 0x80 != 0;
        let is_last_packet = raw_data[0] & 0x40 != 0;

        Ok(RawData {
            is_header,
            is_last_packet,
            raw_data,
        })
    }
}
impl std::fmt::Display for RawData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", String::from_utf8_lossy(&self.stripped_data()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    mod raw_device_data_tests {
        use super::*;
        #[test]
        fn test_convert_raw_data_to_firmware_version() -> Result<(), MsrxToolError> {
            // content should be: REVT3.12
            let data = *b"\xc9\x1b\x52\x45\x56\x54\x33\x2e\x31\x32\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00";
            let raw_data: RawData = data.try_into()?;

            assert_eq!(raw_data.to_string(), "REVT3.12");
            Ok(())
        }
    }
}
