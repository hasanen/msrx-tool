use crate::msrx_tool_error::MsrxToolError;
use std::str::FromStr;

#[derive(Debug, Clone, Copy)]
pub enum DataFormat {
    Iso,
    Raw,
}

impl FromStr for DataFormat {
    type Err = MsrxToolError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "iso" => Ok(DataFormat::Iso),
            _ => Err(MsrxToolError::UnsupportedDataFormat),
        }
    }
}
