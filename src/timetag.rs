use crate::helpers::OscParseError;

// From OSC v1.0 Spec:
// 
// Time tags are represented by a 64 bit fixed point number. 
// The first 32 bits specify the number of seconds since midnight 
// on January 1, 1900, and the last 32 bits specify fractional 
// parts of a second to a precision of about 200 picoseconds. 
// This is the representation used by Internet NTP timestamps.
// The time tag value consisting of 63 zero bits followed by a one
// in the least signifigant bit is a special case meaning "immediately."
#[derive(Debug, PartialEq)]
pub struct OscTimeTag {
    pub seconds: u32,      // number of seconds since midnight on January 1, 1900
    pub fractional: u32,     // fractional parts of a second to a precision of about 200 picoseconds
}

impl OscTimeTag {
    pub fn to_bytes(&self) -> [u8; 8] {
        let mut bytes = [0u8; 8];
        bytes[..4].copy_from_slice(&self.seconds.to_be_bytes());
        bytes[4..].copy_from_slice(&self.fractional.to_be_bytes());
        bytes
    }

    pub fn from_i64(value: i64) -> Self {
        let uvalue = value as u64; // interpret bits as unsigned
        let seconds = (uvalue >> 32) as u32;
        let fractional = uvalue as u32; // lower 32 bits
        Self { seconds, fractional }
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, OscParseError> {
        if bytes.len() < 8 {
            return Err(OscParseError::NotEnoughData);
        }
        let seconds = u32::from_be_bytes(bytes[..4].try_into().unwrap());
        let fractional = u32::from_be_bytes(bytes[4..8].try_into().unwrap());
        Ok(Self { seconds, fractional })
    }

    pub fn is_immediately(&self) -> bool {
        self.seconds == 0 && self.fractional == 1
    }
}

//==================================================================
// Tests
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_time_tag() {
      let tag = OscTimeTag { seconds: 0x11223344, fractional: 0x55667788 };
      assert_eq!(tag.to_bytes(),[0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88]);

      let result = OscTimeTag::from_bytes(&[0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88]);
      assert!(result.is_ok());
      let tag = result.unwrap();
      assert_eq!(tag.seconds, 0x11223344);
      assert_eq!(tag.fractional, 0x55667788);

      let immediately_tag = OscTimeTag { seconds: 0, fractional: 1 };
      assert!(immediately_tag.is_immediately());
    }
}