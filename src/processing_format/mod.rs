use crate::msrx_tool_error::MsrxToolError;
use crate::tracks_data::TracksData;

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum ProcessingFormat {
    /// This format reads all tracks as one string, tracks separated by separator-character
    Text,
}

impl ProcessingFormat {
    fn parse(&self, data: &str, separator: &Option<char>) -> Result<TracksData, MsrxToolError> {
        let separator = separator.unwrap_or('_');

        match self {
            ProcessingFormat::Text => TracksData::from_str(data, &separator),
            _ => Err(MsrxToolError::UnsupportedInputFormat),
        }
    }
}
