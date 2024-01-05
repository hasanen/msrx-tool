use crate::track_data::TrackType;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MsrxToolError {
    #[error("device error")]
    DeviceError(#[from] rusb::Error),
    #[error("Raw data was not card data")]
    RawDataNotCardData,
    #[error("Couldn't set BPI for track {0}")]
    ErrorSettingBPI(usize),
    #[error("Couldn't set leading zeros")]
    ErrorSettingLeadingZeros,
    #[error("Bit conversion error")]
    BitConversionError,
    #[error("device not found")]
    DeviceNotFound,
    #[error("invalid bits  per character for track")]
    InvalidBitsPerCharacter,
    #[error("unsupported data format")]
    UnsupportedDataFormat,
    #[error("format is not supported yet when reading tracks")]
    UnsupportedDataFormatForReading,
    #[error("Couldn't convert track data to string")]
    InvalidUtf8DataInTrack,
    #[error("unknown conversion error")]
    Unknown,
}
