
// Parse Errors
#[derive(Debug, PartialEq)]
pub enum OscParseError {
  NotEnoughData,
  InvalidFormat,
  InvalidString,
  CouldNotParseArguments,
  UnknownTypeTag
}

/// Pads a Vec<u8> to the next multiple of 4 bytes by appending zeros
pub fn pad_to_multiple_of_4_bytes(bytes: &mut Vec<u8>) {
    let pad_length = (4 - (bytes.len() % 4)) % 4;
    if pad_length > 0 {
        bytes.extend(std::iter::repeat(0).take(pad_length));
    }
}

#[allow(dead_code)]
pub fn osc_string_as_bytes(s: &str) -> Vec<u8> {
    let mut v = s.as_bytes().to_vec();
    v.push(0);
    while v.len() % 4 != 0 { v.push(0); }
    v
}

pub fn is_bundle(bytes: &[u8]) -> bool {
    
    // Minimum 20 bytes: 8 bytes for bundle, 8 bytes for timetag, 4 bytes for first message size
    if bytes.len() < 20 {
        return false
    }

    // Check first 8 bytes match "#bundle" with a null terminator
    if &bytes[0..7] != b"#bundle" || bytes[7] != 0 {
        return false;
    }

    true
}