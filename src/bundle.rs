use crate::message::OscMessage;
use crate::timetag::OscTimeTag;
use crate::helpers::{self, OscParseError, osc_string_as_bytes};

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
        let message_size = u32::from_be_bytes(size_bytes.try_into().unwrap()) as usize;
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

    //------------------------------------------------------------------
    #[test]
    fn test_bundle() {
    }
}