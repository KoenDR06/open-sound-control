use crate::argument::OscArgument;
use crate::helpers;
use crate::helpers::OscParseError;

/// Represents an OSC message
pub struct OscMessage {
    pub address: String,
    pub arguments: Vec<OscArgument>
}

impl OscMessage {

  /// Constructs a new OscMessage with a given address and zero arguments
  pub fn new(address: String) -> Self {
        Self { 
          address,
          arguments: Vec::new()
        }
  }

  /// Construct an OscMessage from a sequence of bytes
  pub fn from_bytes(bytes: &[u8]) -> Result<Self, OscParseError> {

    if bytes.is_empty() {
      return Err(OscParseError::NotEnoughData);
    }

    // Extract Address Pattern
    let address_pattern_null_pos = bytes.iter().position(|&b| b == 0).ok_or(OscParseError::InvalidFormat)?;
    let address_pattern_bytes = &bytes[0..address_pattern_null_pos];
    let address_pattern = String::from_utf8(address_pattern_bytes.to_vec()).map_err(|_| OscParseError::InvalidString)?;
    
    // check address pattern starts with '/'
    if ! address_pattern.starts_with("/") {
      return Err(OscParseError::InvalidString);
    }

    // Extract Type Tag String
    let comma_pos = address_pattern_null_pos + bytes[address_pattern_null_pos..].iter().position(|&b| b == 0x2c).ok_or(OscParseError::InvalidFormat)?;
    let type_tag_null_pos = comma_pos + bytes[comma_pos..].iter().position(|&b| b == 0).ok_or(OscParseError::InvalidFormat)?;
    let type_tag_bytes = &bytes[comma_pos..type_tag_null_pos];
    let type_tag_string = String::from_utf8(type_tag_bytes.to_vec()).map_err(|_| OscParseError::InvalidString)?;

    let mut arguments = Vec::<OscArgument>::new();

    // Process arguments
    let mut arguments_index: usize = (type_tag_null_pos + 1 + 3) & !3;

    for tag in type_tag_string[1..].chars() {
      let result = OscArgument::from_bytes(bytes, &mut arguments_index, tag);
      if result.is_err() {
        return Err(OscParseError::CouldNotParseArguments);
      }
      let argument = result.unwrap();
      arguments.push (argument);
    }

    Ok(OscMessage { address: address_pattern, arguments })
  }

  /// Convert an OscMessage to a vector of bytes
  pub fn to_bytes(&self) -> Vec<u8> {
    [
      self.address_pattern_in_bytes(),
      self.type_tag_string_in_bytes(),
      self.arguments_in_bytes()
    ].concat()
  }

  fn address_pattern_in_bytes(&self) -> Vec<u8> {
    let mut bytes: Vec<u8> = self.address.as_bytes().to_vec();
    bytes.push(b'\0');
    helpers::pad_to_multiple_of_4_bytes(&mut bytes);
    bytes
  }

  fn type_tag_string_in_bytes(&self) -> Vec<u8> {
    let mut bytes: Vec<u8> = vec![b','];
    bytes.extend(self.arguments.iter().map(|a| a.type_tag() as u8));
    helpers::pad_to_multiple_of_4_bytes(&mut bytes);
    bytes
  }

  fn arguments_in_bytes(&self) -> Vec<u8> {
    return self.arguments.iter().flat_map(|a| a.to_bytes()).collect();
  }
}

//==================================================================
// Tests
#[cfg(test)]
mod tests {
    use super::*;

    //------------------------------------------------------------------
    #[test]
    fn test_construct_message1() {
      let m1 = OscMessage {
        address: String::from("/new/message"),
        arguments: vec![
          OscArgument::Int32(3),
          OscArgument::Float32(0.4)
        ]
      };

      assert_eq!(m1.address, "/new/message");
      assert_eq!(m1.arguments.len(), 2);
      assert_eq!(m1.arguments.get(0), Some(&OscArgument::Int32(3)));
      assert_eq!(m1.arguments.get(1), Some(&OscArgument::Float32(0.4)));
      assert_eq!(m1.arguments.get(2), None);
    }

    //------------------------------------------------------------------
    #[test]
    fn test_construct_message2() {
      let m2 = OscMessage::new("/nice".to_string());

      assert_eq!(m2.address, "/nice");
      assert_eq!(m2.arguments.len(), 0);
    }

    //------------------------------------------------------------------
    #[test]
    fn test_convert_message_to_bytes1() {
      let m1 = OscMessage {
        address: String::from("/oscillator/4/frequency"),
        arguments: vec![
          OscArgument::Float32(440.0)
        ]
      };

      let mut bytes = m1.to_bytes();

      let expected_bytes: Vec<u8> = vec![
          0x2f, 0x6f, 0x73, 0x63,
          0x69, 0x6c, 0x6c, 0x61,
          0x74, 0x6f, 0x72, 0x2f,
          0x34, 0x2f, 0x66, 0x72,
          0x65, 0x71, 0x75, 0x65,
          0x6e, 0x63, 0x79, 0x00,
          0x2c, 0x66, 0x00, 0x00,
          0x43, 0xdc, 0x00, 0x00,
      ];

      assert_eq!(bytes, expected_bytes);

      let result = OscMessage::from_bytes(&mut bytes);
      assert!(result.is_ok(), "Expected OscMessage ok, got: {:?}", result.err());

      let m2 = result.unwrap();
      assert_eq!(m2.address, "/oscillator/4/frequency");
      assert_eq!(m2.arguments.len(), 1);
      assert_eq!(m2.arguments.get(0), Some(&OscArgument::Float32(440.0)));
    }

    //------------------------------------------------------------------
    #[test]
    fn test_convert_message_to_bytes2() {
      let m1 = OscMessage {
        address: String::from("/foo"),
        arguments: vec![
          OscArgument::Int32(1000),
          OscArgument::Int32(-1),
          OscArgument::String("hello".to_string()),
          OscArgument::Float32(1.234),
          OscArgument::Float32(5.678)
        ]
      };

      let mut bytes = m1.to_bytes();

      let expected_bytes: Vec<u8> = vec![
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
      ];

      assert_eq!(bytes, expected_bytes);

      let result = OscMessage::from_bytes(&mut bytes);
      assert!(result.is_ok(), "Expected OscMessage ok, got: {:?}", result.err());

      let m2 = result.unwrap();
      assert_eq!(m2.address, "/foo");
      assert_eq!(m2.arguments.len(), 5);
      assert_eq!(m2.arguments.get(0), Some(&OscArgument::Int32(1000)));
      assert_eq!(m2.arguments.get(1), Some(&OscArgument::Int32(-1)));
      assert_eq!(m2.arguments.get(2), Some(&OscArgument::String("hello".to_string())));
      assert_eq!(m2.arguments.get(3), Some(&OscArgument::Float32(1.234)));
      assert_eq!(m2.arguments.get(4), Some(&OscArgument::Float32(5.678)));
    }
}