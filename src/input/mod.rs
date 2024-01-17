use crate::msrx_tool_error::MsrxToolError;
use crate::tracks_data::TracksData;
use std::str::FromStr;

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum InputFormat {
    /// This format reads all tracks as one string, tracks separated by separator-character
    Combined,
}

impl FromStr for InputFormat {
    type Err = MsrxToolError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "combined" => Ok(InputFormat::Combined),
            _ => Err(MsrxToolError::UnsupportedInputFormat),
        }
    }
}

pub fn format(tracks_data: &TracksData, format: &OutputFormat, separator: &Option<char>) -> String {
    match format {
        InputFormat::Combined => format_combined(tracks_data, separator),
    }
}

fn format_combined(tracks_data: &TracksData, separator: &Option<char>) -> String {
    let separator = separator.unwrap_or('_');
    let strings: Vec<String> = vec![
        tracks_data.track1.to_string().unwrap(),
        tracks_data.track2.to_string().unwrap(),
        tracks_data.track3.to_string().unwrap(),
    ];

    strings.join(&separator.to_string())
}
