use crate::char_bits_conversion::parity;
use crate::msrx_tool_error::MsrxToolError;
use crate::reverse_string::ReverseString;
use std::char;

pub trait ReversedSignificantBitsFromChar {
    type Error;

    fn to_track_1_bits(&self) -> Result<String, MsrxToolError>;
    fn to_track_2_3_bits(&self) -> Result<String, MsrxToolError>;
}

impl ReversedSignificantBitsFromChar for char {
    type Error = MsrxToolError;

    fn to_track_1_bits(&self) -> Result<String, MsrxToolError> {
        let mut bits = format!("{:b}", *self as u8).reverse()[0..5].to_string();
        let sixth_bit = if (*self as u8) < 0x40 { "0" } else { "1" };
        bits = format!("{}{}", bits, sixth_bit);
        Ok(format!("{}{}", bits, parity(&bits)?))
    }

    fn to_track_2_3_bits(&self) -> Result<String, MsrxToolError> {
        let bits = format!("{:b}", *self as u8).reverse()[0..4].to_string();

        // add parity
        Ok(format!("{}{}", bits, parity(&bits)?))
    }
}
// Tests
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_track_1_bits() -> Result<(), MsrxToolError> {
        assert_eq!('3'.to_track_1_bits()?, "1100100".to_string());
        assert_eq!('K'.to_track_1_bits()?, "1101011".to_string());
        assert_eq!('?'.to_track_1_bits()?, "1111100".to_string());
        assert_eq!('@'.to_track_1_bits()?, "0000010".to_string());

        Ok(())
    }

    #[test]
    fn test_to_track_2_3_bits() -> Result<(), MsrxToolError> {
        assert_eq!('3'.to_track_2_3_bits()?, "11001".to_string());
        assert_eq!('7'.to_track_2_3_bits()?, "11100".to_string());

        Ok(())
    }
}
