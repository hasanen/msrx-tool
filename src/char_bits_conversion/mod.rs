/// Char bits conversion
/// Module offers traits to convert chars to bits and vice versa depending on which track is being used
/// Track 1 suppors wider range of characters than track 2 and track 3
pub mod from_char;
pub mod to_char;
use crate::msrx_tool_error::MsrxToolError;

const TRACK1_SUPPORTED_ASCII: &str =
    " !\"#$%&'()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_";
const TRACK2_3_SUPPORTED_ASCII: &str = "0123456789:;<=>?";

fn parity(bits: &str) -> Result<String, MsrxToolError> {
    if bits.matches('1').count() % 2 != 0 {
        Ok("0".to_string())
    } else {
        Ok("1".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parity_odd_ones() -> Result<(), MsrxToolError> {
        assert_eq!(parity(&"00100".to_string())?, "0".to_string());

        Ok(())
    }

    #[test]
    fn test_parity_even_ones() -> Result<(), MsrxToolError> {
        assert_eq!(parity(&"00110".to_string())?, "1".to_string());

        Ok(())
    }
}
