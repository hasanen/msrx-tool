use crate::char_bits_conversion::to_char::ToChar;
use crate::data_format::DataFormat;
use crate::device_data::DeviceData;
use crate::msrx_tool_error::MsrxToolError;
use crate::reverse_string::ReverseString;
use crate::to_hex::ToHex;
use std::fmt;

#[derive(Debug)]
pub enum TrackType {
    Track1IsoAlphabet,
    Track2_3IsoAlpahbet,
}

#[derive(Debug)]
pub struct TrackData {
    pub data: Vec<u8>,
    pub format: DataFormat,
}
impl TrackData {
    pub fn to_string(&self) -> Result<String, MsrxToolError> {
        Ok("missing implementation".to_string())
    }
    pub fn to_string_with_bpc(
        &self,
        alphabet_track: TrackType,
        bpc: u8,
    ) -> Result<String, MsrxToolError> {
        println!("raw: {:?}", &self);
        if self.data.len() == 0 {
            return Ok(String::new());
        }
        let mut binary = String::new();
        for byte in &self.data {
            binary.push_str(&format!("{:08b}", byte).reverse());
        }

        let chunk_size = match (&alphabet_track, bpc) {
            (TrackType::Track1IsoAlphabet, 8) => 7,
            (TrackType::Track1IsoAlphabet, 7) => 8,
            (TrackType::Track1IsoAlphabet, 6) => 8,
            (TrackType::Track2_3IsoAlpahbet, 8) => 5,
            (TrackType::Track2_3IsoAlpahbet, 5) => 8,
            (TrackType::Track2_3IsoAlpahbet, 6) => 8,
            _ => return Err(MsrxToolError::InvalidBitsPerCharacter),
        };

        dbg!(chunk_size);
        dbg!(&self.data.to_hex());

        let as_binary: Vec<&str> = binary
            .as_bytes()
            .chunks(chunk_size)
            .map(|chunk| std::str::from_utf8(chunk).unwrap())
            .filter(|chunk| chunk.len() == chunk_size)
            .collect();
        let mut ascii = String::new();

        dbg!(&as_binary);
        dbg!(&as_binary.len());
        as_binary[..as_binary.len() - 1].iter().for_each(|chunk| {
            let char = match alphabet_track {
                TrackType::Track1IsoAlphabet => (*chunk).from_track_1_bits(bpc), // Tartteekohan tässä antaa tuo bpc parameetrina? todennäkösesti,
                TrackType::Track2_3IsoAlpahbet => (*chunk).from_track_2_3_bits(bpc),
            }
            .unwrap();

            ascii.push_str(char.to_string().as_str());
        });

        Ok(ascii)
    }
}

impl std::fmt::Display for TrackData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", String::from_utf8_lossy(&self.data))
    }
}
// impl TryFrom<Vec<u8>> for TrackData {
//     type Error = MsrxToolError;

//     fn try_from(raw: Vec<u8>) -> Result<Self, Self::Error> {
//         Ok(TrackData { raw })
//     }
// }
#[cfg(test)]
mod tests {
    use super::*;

    //TODO do tests for covering cases when tracks are none (manual page 11)
    mod raw_track_data_tracks {
        use super::*;

        #[test]
        fn test_convert_raw_track_data_to_ascii_emptry_track() -> Result<(), MsrxToolError> {
            let track_data: TrackData = TrackData {
                data: vec![],
                format: DataFormat::Raw,
            };

            assert_eq!(
                track_data.to_string_with_bpc(TrackType::Track1IsoAlphabet, 7)?,
                ""
            );
            Ok(())
        }
        #[test]
        fn test_convert_raw_track_data_to_ascii_track_has_data_track1_bpc_8(
        ) -> Result<(), MsrxToolError> {
            let track_data: TrackData = TrackData {
                data: vec![0xC5, 0xB0, 0x78, 0x14, 0x95, 0x4E, 0x3E, 0x2A],
                format: DataFormat::Raw,
            };

            assert_eq!(
                track_data.to_string_with_bpc(TrackType::Track1IsoAlphabet, 8)?,
                "%ABC123?"
            );
            Ok(())
        }

        // written with the windows software, UserType (7-5-5)
        // [src/track_data.rs:43] &self.bytes().to_hex() = "51 43 23 62 45 25 64 7c 2a 00 00"
        // [src/track_data.rs:53] &as_binary = [
        //     "1000101",
        //     "1100001",
        //     "1100010",
        //     "0100011",
        //     "1010001",
        //     "1010010",
        //     "0010011",
        //     "0011111",
        //     "0101010",
        //     "00000000",
        //     "00000000",
        // ]
        #[test]
        fn test_convert_raw_track_data_to_ascii_track_has_data_track1_bpc_7_from_device(
        ) -> Result<(), MsrxToolError> {
            let track_data: TrackData = TrackData {
                data: vec![
                    0x51, 0x43, 0x23, 0x62, 0x45, 0x25, 0x64, 0x7c, 0x2a, 0x00, 0x00,
                ],
                format: DataFormat::Raw,
            };

            assert_eq!(
                track_data.to_string_with_bpc(TrackType::Track1IsoAlphabet, 7)?,
                "%ABC123?"
            );
            Ok(())
        }

        // taken from the manual
        // [src/track_data.rs:43] &self.bytes().to_hex() = "45 61 62 23 51 52 13 1f 2a"
        // [src/track_data.rs:53] &as_binary = [
        //     "1010001",
        //     "1000011",
        //     "0100011",
        //     "1100010",
        //     "1000101",
        //     "0100101",
        //     "1100100",
        //     "1111100",
        //     "0101010",
        // ]
        #[test]
        fn test_convert_raw_track_data_to_ascii_track_has_data_track1_bpc_7_manual(
        ) -> Result<(), MsrxToolError> {
            let track_data: TrackData = TrackData {
                data: vec![0x45, 0x61, 0x62, 0x23, 0x51, 0x52, 0x13, 0x1F, 0x2A],
                format: DataFormat::Raw,
            };

            assert_eq!(
                track_data.to_string_with_bpc(TrackType::Track1IsoAlphabet, 7)?,
                "%ABC123?"
            );
            Ok(())
        }
        #[test]
        fn test_convert_raw_track_data_to_ascii_track_has_data_track1_bpc_6(
        ) -> Result<(), MsrxToolError> {
            let track_data: TrackData = TrackData {
                data: vec![0x05, 0x21, 0x22, 0x23, 0x11, 0x12, 0x13, 0x1F, 0x2A],
                format: DataFormat::Raw,
            };

            assert_eq!(
                track_data.to_string_with_bpc(TrackType::Track1IsoAlphabet, 6)?,
                "%ABC123?"
            );
            Ok(())
        }

        // #[test]
        // fn test_convert_raw_track_data_to_ascii_track_has_data_track1__bpc_7(
        // ) -> Result<(), MsrxToolError> {
        //     // data is: "1", bpc is 5
        //     let track_data: TrackData = TrackData {
        //         raw: vec![
        //             0x2a, 0x51, 0x43, 0x67, 0x13, 0x32, 0x73, 0x0b, 0x2a, 0x6b, 0x1a, 0x46, 0x76,
        //             0x52, 0x26, 0x16, 0x4f, 0x57, 0x4a, 0x7a, 0x07, 0x2f, 0x0e, 0x62, 0x37, 0x23,
        //             0x3b, 0x5b, 0x45, 0x25, 0x64, 0x15, 0x54, 0x34, 0x75, 0x0d, 0x4c, 0x04, 0x7c,
        //             0x01, 0x00, 0x00, 0x00,
        //         ],
        //     };

        //     assert_eq!(
        //         track_data.to_string_with_bpc(TrackType::Track1IsoAlphabet, 7)?,
        //         "%ASDFGHJKLQWERTYUIOPZXCVBNM1234567890_"
        //     );
        //     Ok(())
        // }
        #[test]
        fn test_convert_raw_track_data_to_ascii_track_has_data_track2_bpc_8(
        ) -> Result<(), MsrxToolError> {
            let track_data: TrackData = TrackData {
                data: vec![0x2B, 0x88, 0x49, 0xEA, 0xAF],
                format: DataFormat::Raw,
            };
            assert_eq!(
                track_data.to_string_with_bpc(TrackType::Track2_3IsoAlpahbet, 8)?,
                ";12345?"
            );
            Ok(())
        }
        #[test]
        fn test_convert_raw_track_data_to_ascii_track_has_data_track2_bpc_6(
        ) -> Result<(), MsrxToolError> {
            let track_data: TrackData = TrackData {
                data: vec![0x01, 0x01, 0x02, 0x03, 0x04, 0x05, 0x1F, 0x1F],
                format: DataFormat::Raw,
            };
            // TODO: check what is the correct value for ";" in hex, as the manual shows same value for the ";" and "1"
            // assert_eq!(
            //     track_data.to_string_with_bpc(TrackType::Track2_3IsoAlpahbet, 6)?,
            //     ";12345?"
            // );
            assert_eq!(
                track_data.to_string_with_bpc(TrackType::Track2_3IsoAlpahbet, 6)?,
                "112345?"
            );
            Ok(())
        }

        #[test]
        fn test_convert_raw_track_data_to_ascii_track_has_data_track2_bpc_5_from_device(
        ) -> Result<(), MsrxToolError> {
            let track_data: TrackData = TrackData {
                data: vec![0x1a, 0x10, 0x08, 0x19, 0x04, 0x15, 0x1f, 0x15, 0x00, 0x00],
                format: DataFormat::Raw,
            };
            assert_eq!(
                track_data.to_string_with_bpc(TrackType::Track2_3IsoAlpahbet, 5)?,
                ";12345?"
            );
            Ok(())
        }

        #[test]
        fn test_convert_raw_track_data_to_ascii_track_has_data_track2_bpc_5_manual(
        ) -> Result<(), MsrxToolError> {
            let track_data: TrackData = TrackData {
                data: vec![0x0B, 0x01, 0x02, 0x13, 0x04, 0x15, 0x1F, 0x15],
                format: DataFormat::Raw,
            };
            assert_eq!(
                track_data.to_string_with_bpc(TrackType::Track2_3IsoAlpahbet, 5)?,
                ";12345?"
            );
            Ok(())
        }
        // #[test]
        // fn test_convert_raw_track_data_to_ascii_track_has_data_track2_bpc_5(
        // ) -> Result<(), MsrxToolError> {
        //     // data is: "1", bpc is 5
        //     let track_data: TrackData = TrackData {
        //         raw: vec![
        //             0x1a, 0x1a, 0x01, 0x13, 0x02, 0x1c, 0x0d, 0x15, 0x04, 0x19, 0x08, 0x10, 0x01,
        //             0x13, 0x6e, 0x02, 0x1c, 0x0d, 0x15, 0x04, 0x19, 0x08, 0x10, 0x1f, 0x04, 0x00,
        //             0x00, 0x00,
        //         ],
        //     };

        //     assert_eq!(
        //         track_data.to_string_with_bpc(TrackType::Track2_3IsoAlpahbet, 5)?,
        //         "`01234567890123456789_"
        //     );
        //     Ok(())
        // }
    }
}
