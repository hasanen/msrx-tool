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
    #[error("unknown conversion error")]
    Unknown,
}
