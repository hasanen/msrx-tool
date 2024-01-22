use crate::data_format::DataFormat;
use crate::msrx_tool_error::MsrxToolError;

#[derive(Debug)]
pub struct TrackData {
    pub data: Vec<u8>,
    pub format: DataFormat,
}
impl TrackData {
    pub fn as_packets(&self) -> Vec<u8> {
        if self.data.is_empty() {
            return vec![0x00, 0x00];
        } else {
            return self.data.clone();
        }
    }
    pub fn to_string(&self) -> Result<String, MsrxToolError> {
        match self.format {
            DataFormat::Iso => self.to_string_iso(),
            DataFormat::Raw => return Err(MsrxToolError::UnsupportedDataFormat),
        }
    }

    fn to_string_iso(&self) -> Result<String, MsrxToolError> {
        match String::from_utf8(self.data.clone()) {
            Ok(ascii_string) => Ok(ascii_string),
            Err(_) => Err(MsrxToolError::InvalidUtf8DataInTrack),
        }
    }
}

impl std::fmt::Display for TrackData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", String::from_utf8_lossy(&self.data))
    }
}
