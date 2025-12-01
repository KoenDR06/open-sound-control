use crate::helpers;

/// Represents an OSC argument with a given type
/// 17 types are recognised: 
/// - 4 "main" argument types
/// - 13 "nonstandard" types
#[derive(Debug, PartialEq)]
pub enum OscArgument {

    /** MAIN TYPES **/
    Int32(i32), /* 32-bit integer */
    Float32(f32), /* 32-bit floating point number */
    String(String), /* String */
    Blob(Vec<u8>), /* A number of 8-bit bytes of arbitrary binary data */

    /** NONSTANDARD TYPES **/
    Int64(i64), /* 64 bit big-endian two’s complement integer */
    TimeTag(i64), /* OSC-timetag */
    Float64(f64), /* 64 bit (“double”) IEEE 754 floating point number */
    AlternateType(String), /* Alternate type represented as an OSC-string (for example, for systems that differentiate “symbols” from “strings”) */
    AsciiCharacter(i32), /* an ascii character, sent as 32 bits */
    Colour(u8, u8, u8, u8) /* 32 bit RGBA color */,
    MidiMessage(u8, u8, u8, u8), /* 4 byte MIDI message. Bytes from MSB to LSB are: port id, status byte, data1, data2*/
    True, /* True. No bytes are allocated in the argument data. */
    False, /* False. No bytes are allocated in the argument data. */
    Nil, /* Nil. No bytes are allocated in the argument data. */
    Infinitum, /* Infinitum. No bytes are allocated in the argument data.*/
    ArrayBegin, /* Indicates the beginning of an array. The tags following are for data in the Array until a close brace tag is reached. */
    ArrayEnd /* Indicates the end of an array */
}

impl OscArgument {
    /// Returns the type tag character for the OscArgument
    pub fn type_tag(&self) -> char {
        match self {
            OscArgument::Int32(_) => 'i',
            OscArgument::Float32(_) => 'f',
            OscArgument::String(_) => 's',
            OscArgument::Blob(_) => 'b',
            OscArgument::Int64(_) => 'h',
            OscArgument::TimeTag(_) => 't',
            OscArgument::Float64(_) => 'd',
            OscArgument::AlternateType(_) => 'S',
            OscArgument::AsciiCharacter(_) => 'c',
            OscArgument::Colour(_, _, _, _) => 'r',
            OscArgument::MidiMessage(_, _, _, _) => 'm',
            OscArgument::True => 'T',
            OscArgument::False => 'F',
            OscArgument::Nil => 'N',
            OscArgument::Infinitum => 'I',
            OscArgument::ArrayBegin => '[',
            OscArgument::ArrayEnd => ']'
        }
    }

    /// Returns the argument as a vector of bytes
    pub fn to_bytes(&self) -> Vec<u8> {
      match self {
        OscArgument::Int32(x) => x.to_be_bytes().to_vec(),
        OscArgument::Float32(f) => f.to_be_bytes().to_vec(),
        OscArgument::String(s) => {
            let mut bytes = s.as_bytes().to_vec();
            helpers::pad_to_multiple_of_4_bytes(&mut bytes);
            bytes
        },
        OscArgument::Blob(data) => {
            let mut bytes = Vec::new();

            // add length as 32-bit integer
            let len = data.len() as u32;
            bytes.extend(&len.to_be_bytes());

            // add data bytes
            bytes.extend(data);

            helpers::pad_to_multiple_of_4_bytes(&mut bytes); // pad
            bytes
        },
        OscArgument::Int64(i) => i.to_be_bytes().to_vec(),
        OscArgument::TimeTag(t) => t.to_be_bytes().to_vec(),
        OscArgument::Float64(f) => f.to_be_bytes().to_vec(),
        OscArgument::AlternateType(s) => {
            let mut bytes = s.as_bytes().to_vec();
            helpers::pad_to_multiple_of_4_bytes(&mut bytes);
            bytes
        },
        OscArgument::AsciiCharacter(c) => c.to_be_bytes().to_vec(),
        OscArgument::Colour(r, g, b, a) => vec![*r, *g, *b, *a],
        OscArgument::MidiMessage(port_id, status_byte, data1, data2) => vec![*port_id, *status_byte, *data1, *data2],
        _ => [].to_vec()
      }
    }
}

//==================================================================
// Tests
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_tag() {
        assert_eq!(OscArgument::Int32(0).type_tag(), 'i');
        assert_eq!(OscArgument::Float32(0.0).type_tag(), 'f');
        assert_eq!(OscArgument::String("".to_string()).type_tag(), 's');
        assert_eq!(OscArgument::Blob(vec![]).type_tag(), 'b');
        assert_eq!(OscArgument::Int64(0).type_tag(), 'h');
        assert_eq!(OscArgument::TimeTag(0).type_tag(), 't');
        assert_eq!(OscArgument::Float64(0.0).type_tag(), 'd');
        assert_eq!(OscArgument::AlternateType("".to_string()).type_tag(), 'S');
        assert_eq!(OscArgument::AsciiCharacter('a' as i32).type_tag(), 'c');
        assert_eq!(OscArgument::Colour(255, 0, 255, 0).type_tag(), 'r');
        assert_eq!(OscArgument::MidiMessage(0, 0, 0, 0).type_tag(), 'm');
        assert_eq!(OscArgument::True.type_tag(), 'T');
        assert_eq!(OscArgument::False.type_tag(), 'F');
        assert_eq!(OscArgument::Nil.type_tag(), 'N');
        assert_eq!(OscArgument::Infinitum.type_tag(), 'I');
        assert_eq!(OscArgument::ArrayBegin.type_tag(), '[');
        assert_eq!(OscArgument::ArrayEnd.type_tag(), ']');
    }
}