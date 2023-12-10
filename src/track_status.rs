#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum TrackStatus {
    Ok,
    WriteOrReadError,
    CommandFormatError,
    InvalidCommand,
    InvalidCardSwipeOnWrite,
    Unknown,
}

impl From<u8> for TrackStatus {
    fn from(value: u8) -> Self {
        match value {
            0x30 => TrackStatus::Ok,
            0x31 => TrackStatus::WriteOrReadError,
            0x32 => TrackStatus::CommandFormatError,
            0x34 => TrackStatus::InvalidCommand,
            0x39 => TrackStatus::InvalidCardSwipeOnWrite,
            _ => TrackStatus::Unknown,
        }
    }
}
