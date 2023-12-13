use crate::msrx_tool_error::MsrxToolError;
use crate::reverse_string::ReverseString;
use std::char;

// Define the trait

pub trait ToChar {
    type Error;

    fn from_track_1_bits(&self) -> Result<char, MsrxToolError>;
    fn from_track_2_3_bits(&self) -> Result<char, MsrxToolError>;
}

// Implement the trait for str
impl<T: AsRef<str>> ToChar for T {
    type Error = MsrxToolError;
    fn from_track_1_bits(&self) -> Result<char, MsrxToolError> {
        let mut as_num = usize::from_str_radix(&self.as_ref().reverse()[1..], 2).unwrap();
        if as_num & (1 << 5) == 0 {
            as_num |= 0x20;
        } else {
            as_num &= !(1 << 5); // Reset the 5th bit
            as_num |= 1 << 6; // Set the 6th bit
        }

        match char::from_u32(as_num as u32) {
            Some(c) => Ok(c),
            None => Err(MsrxToolError::BitConversionError),
        }
    }
    fn from_track_2_3_bits(&self) -> Result<char, MsrxToolError> {
        let mut as_num = usize::from_str_radix(&self.as_ref().reverse(), 2).unwrap();
        let mask = usize::from_str_radix("00110000", 2).unwrap();
        as_num |= mask;

        match char::from_u32(as_num as u32) {
            Some(c) => Ok(c),
            None => Err(MsrxToolError::BitConversionError),
        }
    }
}

// Tests
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_from_track_2_3_bits_to_char() -> Result<(), MsrxToolError> {
        assert_eq!("11001".to_string().from_track_2_3_bits()?, '3');
        assert_eq!("11100".to_string().from_track_2_3_bits()?, '7');

        Ok(())
    }

    #[test]
    fn test_from_track_1_bits_to_char() -> Result<(), MsrxToolError> {
        assert_eq!("1100100".to_string().from_track_1_bits()?, '3');
        assert_eq!("1111100".to_string().from_track_1_bits()?, '?');
        assert_eq!("0000010".to_string().from_track_1_bits()?, '@');
        assert_eq!("1101011".to_string().from_track_1_bits()?, 'K');

        Ok(())
    }
}
