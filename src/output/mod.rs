use crate::msrx_tool_error::MsrxToolError;
use crate::tracks_data::TracksData;
use std::str::FromStr;

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum OutputFormat {
    Json,
    /// This format combines all tracks into one string, tracks separated by underscore
    Combined,
}

impl FromStr for OutputFormat {
    type Err = MsrxToolError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "json" => Ok(OutputFormat::Json),
            "combined" => Ok(OutputFormat::Combined),
            _ => Err(MsrxToolError::UnsupportedOutputFormat),
        }
    }
}

pub fn format(tracks_data: &TracksData, format: &OutputFormat, separator: &Option<char>) -> String {
    match format {
        OutputFormat::Json => format_json(tracks_data),
        OutputFormat::Combined => format_combined(tracks_data, separator),
    }
}

fn format_json(_tracks_data: &TracksData) -> String {
    todo!("Implement JSON output")
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
