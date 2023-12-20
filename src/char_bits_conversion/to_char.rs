use crate::msrx_tool_error::MsrxToolError;
use crate::reverse_string::ReverseString;
use std::char;

// Define the trait

pub trait ToChar {
    type Error;

    fn from_track_1_bits(&self, bits_per_character: u8) -> Result<char, MsrxToolError>;
    fn from_track_2_3_bits(&self, bits_per_character: u8) -> Result<char, MsrxToolError>;
}

// Implement the trait for str
impl<T: AsRef<str>> ToChar for T {
    type Error = MsrxToolError;
    fn from_track_1_bits(&self, bits_per_character: u8) -> Result<char, MsrxToolError> {
        dbg!("--------------------");
        dbg!(&self.as_ref());
        let mut as_num = if bits_per_character > 6 {
            let start_index = (self.as_ref().len() - 6) as usize;
            usize::from_str_radix(&self.as_ref().reverse()[start_index..], 2).unwrap()
        } else {
            usize::from_str_radix(&self.as_ref().reverse(), 2).unwrap()
        };

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
    fn from_track_2_3_bits(&self, bits_per_character: u8) -> Result<char, MsrxToolError> {
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
    fn test_from_track_1_bits_per_character_8_to_char() -> Result<(), MsrxToolError> {
        assert_eq!("1010001".to_string().from_track_1_bits(8)?, '%');
        assert_eq!("1000011".to_string().from_track_1_bits(8)?, 'A');
        assert_eq!("0100011".to_string().from_track_1_bits(8)?, 'B');
        assert_eq!("1100010".to_string().from_track_1_bits(8)?, 'C');
        assert_eq!("1000101".to_string().from_track_1_bits(8)?, '1');
        assert_eq!("0100101".to_string().from_track_1_bits(8)?, '2');
        assert_eq!("1100100".to_string().from_track_1_bits(8)?, '3');
        assert_eq!("1111100".to_string().from_track_1_bits(8)?, '?');
        assert_eq!("0000010".to_string().from_track_1_bits(8)?, '@');
        assert_eq!("1101011".to_string().from_track_1_bits(8)?, 'K');

        Ok(())
    }

    #[test]
    fn test_from_track_1_bits_per_character_7_to_char() -> Result<(), MsrxToolError> {
        assert_eq!("10100010".to_string().from_track_1_bits(7)?, '%');
        assert_eq!("1010001".to_string().from_track_1_bits(7)?, '%');
        assert_eq!("1000011".to_string().from_track_1_bits(7)?, 'A');
        assert_eq!("0100011".to_string().from_track_1_bits(7)?, 'B');
        assert_eq!("1100010".to_string().from_track_1_bits(7)?, 'C');
        assert_eq!("1000101".to_string().from_track_1_bits(7)?, '1');
        assert_eq!("0100101".to_string().from_track_1_bits(7)?, '2');
        assert_eq!("1100100".to_string().from_track_1_bits(7)?, '3');
        assert_eq!("1111100".to_string().from_track_1_bits(7)?, '?');

        Ok(())
    }

    #[test]
    fn test_from_track_1_bits_per_character_6_to_char() -> Result<(), MsrxToolError> {
        assert_eq!("101000".to_string().from_track_1_bits(6)?, '%');
        assert_eq!("100001".to_string().from_track_1_bits(6)?, 'A');
        assert_eq!("010001".to_string().from_track_1_bits(6)?, 'B');
        assert_eq!("110001".to_string().from_track_1_bits(6)?, 'C');
        assert_eq!("100010".to_string().from_track_1_bits(6)?, '1');
        assert_eq!("010010".to_string().from_track_1_bits(6)?, '2');
        assert_eq!("110010".to_string().from_track_1_bits(6)?, '3');
        assert_eq!("111110".to_string().from_track_1_bits(6)?, '?');

        Ok(())
    }

    #[test]
    fn test_from_track_2_3_bits_per_character_8_to_char() -> Result<(), MsrxToolError> {
        assert_eq!("11010".to_string().from_track_2_3_bits(8)?, ';');
        assert_eq!("10000".to_string().from_track_2_3_bits(8)?, '1');
        assert_eq!("01000".to_string().from_track_2_3_bits(8)?, '2');
        assert_eq!("11001".to_string().from_track_2_3_bits(8)?, '3');
        assert_eq!("00100".to_string().from_track_2_3_bits(8)?, '4');
        assert_eq!("10101".to_string().from_track_2_3_bits(8)?, '5');
        assert_eq!("11111".to_string().from_track_2_3_bits(8)?, '?');

        Ok(())
    }

    #[test]
    fn test_from_track_2_3_bits_per_character_6_to_char() -> Result<(), MsrxToolError> {
        assert_eq!("100000".to_string().from_track_2_3_bits(6)?, '1');
        assert_eq!("010000".to_string().from_track_2_3_bits(6)?, '2');
        assert_eq!("110000".to_string().from_track_2_3_bits(6)?, '3');
        assert_eq!("001000".to_string().from_track_2_3_bits(6)?, '4');
        assert_eq!("101000".to_string().from_track_2_3_bits(6)?, '5');
        assert_eq!("111110".to_string().from_track_2_3_bits(6)?, '?');

        Ok(())
    }

    #[test]
    fn test_from_track_2_3_bits_per_character_5_to_char() -> Result<(), MsrxToolError> {
        assert_eq!("11010".to_string().from_track_2_3_bits(5)?, ';');
        assert_eq!("10000".to_string().from_track_2_3_bits(5)?, '1');
        assert_eq!("01000".to_string().from_track_2_3_bits(5)?, '2');
        assert_eq!("11001".to_string().from_track_2_3_bits(5)?, '3');
        assert_eq!("00100".to_string().from_track_2_3_bits(5)?, '4');
        assert_eq!("10101".to_string().from_track_2_3_bits(5)?, '5');
        assert_eq!("11111".to_string().from_track_2_3_bits(5)?, '?');

        Ok(())
    }
}
