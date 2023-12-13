use crate::msrx_tool_error::MsrxToolError;
use crate::reverse_string::ReverseString;
use lazy_static::lazy_static;
use std::collections::HashMap;

lazy_static! {
    static ref TRACK1_BIN_TO_ASCII: HashMap<&'static str, &'static str> = {
        let mut m = HashMap::new();
        m.insert("0000001", " ");
        m.insert("1000000", "!");
        m.insert("0100000", "\"");
        m.insert("1100001", "#");
        m.insert("0010000", "$");
        m.insert("1010001", "%");
        m.insert("0110001", "&");
        m.insert("1110000", "'");
        m.insert("0001000", "(");
        m.insert("1001001", ")");
        m.insert("0101001", "*");
        m.insert("1101000", "+");
        m.insert("0011001", ",");
        m.insert("1011000", "-");
        m.insert("0111000", ".");
        m.insert("1001001", "/");
        m.insert("0000100", "0");
        m.insert("1000101", "1");
        m.insert("0100101", "2");
        m.insert("1100100", "3");
        m.insert("0010101", "4");
        m.insert("1010100", "5");
        m.insert("0110100", "6");
        m.insert("1110101", "7");
        m.insert("0001101", "8");
        m.insert("1001100", "9");
        m.insert("0101100", ":");
        m.insert("1101101", ";");
        m.insert("0011100", "<");
        m.insert("1011101", "=");
        m.insert("0111101", ">");
        m.insert("1111100", "?");
        m.insert("0000010", "@");
        m.insert("1000011", "A");
        m.insert("0100011", "B");
        m.insert("1100010", "C");
        m.insert("0010011", "D");
        m.insert("1010010", "E");
        m.insert("0110010", "F");
        m.insert("1110011", "G");
        m.insert("0001011", "H");
        m.insert("1001010", "I");
        m.insert("0101010", "J");
        m.insert("1101011", "K");
        m.insert("0011010", "L");
        m.insert("1011011", "M");
        m.insert("0111011", "N");
        m.insert("1111010", "O");
        m.insert("0000111", "P");
        m.insert("1000110", "Q");
        m.insert("0100110", "R");
        m.insert("1100111", "S");
        m.insert("0010110", "T");
        m.insert("1010111", "U");
        m.insert("0110111", "V");
        m.insert("1110110", "W");
        m.insert("0001110", "X");
        m.insert("1001111", "Y");
        m.insert("0101111", "Z");
        m.insert("1101110", "[");
        m.insert("0011111", "\\");
        m.insert("1011110", "]");
        m.insert("0111110", "^");
        m.insert("1111111", "_");
        m
    };
    static ref TRACK2_3_BIN_TO_ASCII: HashMap<&'static str, &'static str> = {
        let mut m = HashMap::new();
        m.insert("00001", "0");
        m.insert("10000", "1");
        m.insert("01000", "2");
        m.insert("11001", "3");
        m.insert("00100", "4");
        m.insert("10101", "5");
        m.insert("01101", "6");
        m.insert("11100", "7");
        m.insert("00010", "8");
        m.insert("10011", "9");
        m.insert("01011", ":");
        m.insert("11010", ";");
        m.insert("00111", "<");
        m.insert("10110", "=");
        m.insert("01110", ">");
        m.insert("11111", "?");

        m
    };
}

enum TrackType {
    Track1IsoAlphabet,
    Track2_3IsoAlpahbet,
}

#[derive(Debug)]
pub struct TrackData {
    pub raw: Vec<u8>,
}
impl TrackData {
    pub fn to_string_with_bpi(&self, alphabet_track: TrackType, bpi: usize) -> String {
        if self.raw.len() == 0 {
            return String::new();
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
        as_binary[..as_binary.len() - 1]
            .iter()
            .for_each(|chunk| match alphabet_track {
                TrackType::Track1IsoAlphabet => {
                    ascii.push_str(TRACK1_BIN_TO_ASCII.get(chunk).unwrap());
                }
                TrackType::Track2_3IsoAlpahbet => todo!(),
            });

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
            let track_data: TrackData = TrackData { raw: vec![] };

            assert_eq!(
                track_data.to_string_with_bpi(TrackType::Track1IsoAlphabet, 7),
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
                track_data.to_string_with_bpi(TrackType::Track1IsoAlphabet, 7),
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
                track_data.to_string_with_bpi(TrackType::Track2_3IsoAlpahbet, 5),
                ";12345?"
            );
            Ok(())
        }
    }
}
