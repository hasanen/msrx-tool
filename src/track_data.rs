use crate::char_bits_conversion::to_char::ToChar;
use crate::msrx_tool_error::MsrxToolError;
use crate::reverse_string::ReverseString;
enum TrackType {
    Track1IsoAlphabet,
    Track2_3IsoAlpahbet,
}

#[derive(Debug)]
pub struct TrackData {
    pub raw: Vec<u8>,
}
impl TrackData {
    pub fn to_string_with_bpi(
        &self,
        alphabet_track: TrackType,
        bpi: usize,
    ) -> Result<String, MsrxToolError> {
        if self.raw.len() == 0 {
            return Ok(String::new());
        }
        let mut binary = String::new();
        for byte in &self.raw {
            binary.push_str(&format!("{:08b}", byte).reverse());
        }

        let as_binary: Vec<&str> = binary
            .as_bytes()
            .chunks(bpi)
            .map(|chunk| std::str::from_utf8(chunk).unwrap())
            .filter(|chunk| chunk.len() == bpi)
            .collect();
        let mut ascii = String::new();
        dbg!(&as_binary);
        as_binary[..as_binary.len() - 1].iter().for_each(|chunk| {
            let char = match alphabet_track {
                TrackType::Track1IsoAlphabet => (*chunk).from_track_1_bits(),
                TrackType::Track2_3IsoAlpahbet => (*chunk).from_track_2_3_bits(),
            }
            .unwrap();

            ascii.push_str(char.to_string().as_str());
        });

        Ok(ascii)
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
            let track_data: TrackData = TrackData { raw: vec![] };

            assert_eq!(
                track_data.to_string_with_bpi(TrackType::Track1IsoAlphabet, 7)?,
                ""
            );
            Ok(())
        }
        #[test]
        fn test_convert_raw_track_data_to_ascii_track_has_data_track1() -> Result<(), MsrxToolError>
        {
            // data is: "1", bpi is 5
            let track_data: TrackData = TrackData {
                raw: vec![0xC5, 0xB0, 0x78, 0x14, 0x95, 0x4E, 0x3E, 0x2A],
            };

            assert_eq!(
                track_data.to_string_with_bpi(TrackType::Track1IsoAlphabet, 7)?,
                "%ABC123?"
            );
            Ok(())
        }
        #[test]
        fn test_convert_raw_track_data_to_ascii_track_has_data_track2() -> Result<(), MsrxToolError>
        {
            // data is: "1", bpi is 5
            let track_data: TrackData = TrackData {
                raw: vec![0x2B, 0x88, 0x49, 0xEA, 0xAF],
            };

            assert_eq!(
                track_data.to_string_with_bpi(TrackType::Track2_3IsoAlpahbet, 5)?,
                ";12345?"
            );
            Ok(())
        }
    }
}
