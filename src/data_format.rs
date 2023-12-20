use crate::msrx_tool_error::MsrxToolError;
use std::str::FromStr;

#[derive(Debug, Clone, Copy)]
pub enum DataFormat {
    ISO,
    RAW,
}

impl FromStr for DataFormat {
    type Err = MsrxToolError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "iso" => Ok(DataFormat::ISO),
            "raw" => Ok(DataFormat::RAW),
            _ => Err(MsrxToolError::UnsupportedDataFormat),
        }
    }
}
