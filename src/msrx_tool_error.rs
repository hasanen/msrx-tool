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
    #[error("unknown conversion error")]
    Unknown,
}
