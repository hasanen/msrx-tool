use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
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
    #[error("unsupported data format")]
    UnsupportedDataFormat,
    #[error("unsupported output format")]
    UnsupportedOutputFormat,
    #[error("format is not supported yet when reading tracks")]
    UnsupportedDataFormatForReading,
    #[error("Couldn't convert track data to string")]
    InvalidUtf8DataInTrack,
    #[error("Card was not swiped")]
    CardNotSwiped,
    #[error("Data for track {0} is too long: {1}. Max length is {2}")]
    DataForTrackIsTooLong(usize, usize, usize),
    #[error("Invalid data for track {0}. Allowed chars: {1}")]
    InvalidTrackData(usize, String),
    #[error("Invalid start sentinel for track {0}, expected {1}")]
    InvalidStartSentinel(usize, char),
    #[error("Invalid end sentinel for track {0}, expected {1}")]
    InvalidEndSentinel(usize, char),
    #[error("unknown conversion error")]
    Unknown,
}
