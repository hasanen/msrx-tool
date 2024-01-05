use crate::msrx_tool_error::MsrxToolError;
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

impl Output {
    pub fn format(&self, format: OutputFormat, separator: Option<char>) -> String {
        match format {
            OutputFormat::Json => self.format_json(),
            OutputFormat::Combined => self.format_combined(separator),
        }
    }

    fn format_json(&self) -> String {
        todo!("Implement JSON output")
    }

    fn format_combined(&self, separator: Option<char>) -> String {
        let separator = separator.unwrap_or('_');
        let strings = vec![];
        if 
    }
}
