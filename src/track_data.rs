use crate::msrx_tool_error::MsrxToolError;

#[derive(Debug)]
pub struct TrackData {
    pub raw: Vec<u8>,
}
impl TrackData {
    pub fn to_string(&self) -> String {
        let mut ascii = String::new();
        for byte in &self.raw {
            ascii.push(*byte as char);
        }
        ascii
    }
}
impl TryFrom<Vec<u8>> for TrackData {
    type Error = MsrxToolError;

    fn try_from(raw: Vec<u8>) -> Result<Self, Self::Error> {
        Ok(TrackData { raw })
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    //TODO do tests for covering cases when tracks are none (manual page 11)
    mod raw_track_data_tracks {
        use super::*;

        #[test]
        fn test_convert_raw_track_data_to_ascii_emptry_track() -> Result<(), MsrxToolError> {
            let track_data: TrackData = TrackData::try_from(vec![])?;

            assert_eq!(track_data.to_string(), "");
            Ok(())
        }
        #[test]
        fn test_convert_raw_track_data_to_ascii_track_has_data() -> Result<(), MsrxToolError> {
            // data is: "1", bpi is 5
            let track_data: TrackData = TrackData::try_from(vec![0xaf, 0xc2, 0xb0, 0x00])?;

            assert_eq!(track_data.to_string(), "1");
            Ok(())
        }
    }
}
