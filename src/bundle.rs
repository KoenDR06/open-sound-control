use crate::message::OscMessage;
use crate::timetag::OscTimeTag;
use crate::helpers::{self, OscParseError, osc_string_as_bytes};

/// Represents an OSC Bundle
pub struct OscBundle {
    pub messages: Vec<OscMessage>,
    pub time_tag: OscTimeTag
}

impl OscBundle {

  /// Construct an OscBundle from a sequence of bytes
  pub fn from_bytes(bytes: &[u8]) -> Result<Self, OscParseError> {

    if ! helpers::is_bundle(bytes) {
      return Err(OscParseError::InvalidFormat)
    }

    let time_tag = OscTimeTag::from_bytes(&bytes[8..16])?;

    let mut messages = vec![];
    let mut offset = 16;

    while offset < bytes.len() {
        let size_bytes = bytes.get(offset..offset + 4).ok_or(OscParseError::NotEnoughData)?;
        let message_size = u32::from_be_bytes(size_bytes.try_into().map_err(|it| OscParseError::InvalidString)?) as usize;
        offset += 4;

        // check we have enough bytes for the message
        let message_bytes = bytes.get(offset..offset + message_size).ok_or(OscParseError::NotEnoughData)?;
        messages.push(OscMessage::from_bytes(message_bytes)?);

        offset += message_size;
    }

    let bundle = OscBundle {
      messages,
      time_tag
    };

    Ok(bundle)
  }

  /// Turn an OscBundle into a sequence of bytes
  pub fn to_bytes(&self) -> Vec<u8> {
    let mut bytes = osc_string_as_bytes("#bundle");

    bytes.extend (self.time_tag.to_bytes());

    for message in &self.messages {
      let message_bytes = message.to_bytes();
      let message_size = message_bytes.len() as u32;
      bytes.extend (&message_size.to_be_bytes());
      bytes.extend (message_bytes);
    }

    return bytes
  }
}


//==================================================================
// Tests
#[cfg(test)]
mod tests {
    use super::*;
    use crate::argument::OscArgument;

    //------------------------------------------------------------------
    #[test]
    fn test_bundle_to_bytes() {
      let time_tag = OscTimeTag::now();

      let m1 = OscMessage {
        address: String::from("/oscillator/4/frequency"),
        arguments: vec![
          OscArgument::Float32(440.0)
        ]
      };

      let m2 = OscMessage {
        address: String::from("/foo"),
        arguments: vec![
          OscArgument::Int32(1000),
          OscArgument::Int32(-1),
          OscArgument::String("hello".to_string()),
          OscArgument::Float32(1.234),
          OscArgument::Float32(5.678)
        ]
      };

      let mut expected_bytes = osc_string_as_bytes("#bundle");
      expected_bytes.extend(time_tag.to_bytes().to_vec());
      expected_bytes.extend (32_u32.to_be_bytes());
      expected_bytes.extend (vec![
          0x2f, 0x6f, 0x73, 0x63,
          0x69, 0x6c, 0x6c, 0x61,
          0x74, 0x6f, 0x72, 0x2f,
          0x34, 0x2f, 0x66, 0x72,
          0x65, 0x71, 0x75, 0x65,
          0x6e, 0x63, 0x79, 0x00,
          0x2c, 0x66, 0x00, 0x00,
          0x43, 0xdc, 0x00, 0x00,
      ]);
      expected_bytes.extend (40_u32.to_be_bytes());
      expected_bytes.extend (vec![
          0x2f, 0x66, 0x6f, 0x6f,
          0x00, 0x00, 0x00, 0x00,
          0x2c, 0x69, 0x69, 0x73,
          0x66, 0x66, 0x00, 0x00,
          0x00, 0x00, 0x03, 0xe8,
          0xff, 0xff, 0xff, 0xff,
          0x68, 0x65, 0x6c, 0x6c,
          0x6f, 0x00, 0x00, 0x00,
          0x3f, 0x9d, 0xf3, 0xb6,
          0x40, 0xb5, 0xb2, 0x2d,
      ]);

      let bundle = OscBundle {
        time_tag,
        messages: vec![
          m1, m2
        ]
      };

      let bytes = bundle.to_bytes();
      assert_eq!(bytes, expected_bytes);

    }

    //------------------------------------------------------------------
    #[test]
    fn test_bundle_from_bytes() {
      let time_tag = OscTimeTag::now();

      let mut bundle_bytes = osc_string_as_bytes("#bundle");
      bundle_bytes.extend(time_tag.to_bytes().to_vec());
      bundle_bytes.extend (32_u32.to_be_bytes());
      bundle_bytes.extend (vec![
          0x2f, 0x6f, 0x73, 0x63,
          0x69, 0x6c, 0x6c, 0x61,
          0x74, 0x6f, 0x72, 0x2f,
          0x34, 0x2f, 0x66, 0x72,
          0x65, 0x71, 0x75, 0x65,
          0x6e, 0x63, 0x79, 0x00,
          0x2c, 0x66, 0x00, 0x00,
          0x43, 0xdc, 0x00, 0x00,
      ]);
      bundle_bytes.extend (40_u32.to_be_bytes());
      bundle_bytes.extend (vec![
          0x2f, 0x66, 0x6f, 0x6f,
          0x00, 0x00, 0x00, 0x00,
          0x2c, 0x69, 0x69, 0x73,
          0x66, 0x66, 0x00, 0x00,
          0x00, 0x00, 0x03, 0xe8,
          0xff, 0xff, 0xff, 0xff,
          0x68, 0x65, 0x6c, 0x6c,
          0x6f, 0x00, 0x00, 0x00,
          0x3f, 0x9d, 0xf3, 0xb6,
          0x40, 0xb5, 0xb2, 0x2d,
      ]);

      let result = OscBundle::from_bytes(&bundle_bytes);

      assert!(result.is_ok());

      let bundle = result.unwrap();

      assert_eq!(bundle.messages.len(), 2);

      let r1 = bundle.messages.get(0);
      let r2 = bundle.messages.get(1);

      assert!(r1.is_some());
      assert!(r2.is_some());

      let m1 = r1.unwrap();
      let m2 = r2.unwrap();

      assert_eq!(m1.address, "/oscillator/4/frequency");
      assert_eq!(m1.arguments.len(), 1);
      assert_eq!(m1.arguments, vec![OscArgument::Float32(440.0)]);

      assert_eq!(m2.address, "/foo");
      assert_eq!(m2.arguments.len(), 5);
      assert_eq!(m2.arguments, vec![
          OscArgument::Int32(1000),
          OscArgument::Int32(-1),
          OscArgument::String("hello".to_string()),
          OscArgument::Float32(1.234),
          OscArgument::Float32(5.678)
        ]);

      //assert_eq!(m1.)
    }
}
