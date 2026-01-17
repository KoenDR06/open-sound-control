use crate::helpers;
use crate::helpers::OscParseError;
use crate::timetag::OscTimeTag;



// Represents a colour sent by OSC
#[derive(Debug, PartialEq)]
pub struct OscColour {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
    pub alpha: u8
}

/// Represents an OSC argument with a given type
/// 17 types are recognised: 
/// - 4 "main" argument types
/// - 13 "nonstandard" types (often not used by other OSC implementations)
#[derive(Debug, PartialEq)]
pub enum OscArgument {

    /** MAIN TYPES **/
    Int32(i32), /* 32-bit integer */
    Float32(f32), /* 32-bit floating point number */
    String(String), /* String */
    Blob(Vec<u8>), /* A number of 8-bit bytes of arbitrary binary data */

    /** NONSTANDARD TYPES - MANY OSC IMPLEMENTATIONS DON'T IMPLEMENT THESE (BUT WE DO, OF COURSE) **/
    Int64(i64), /* 64 bit big-endian two’s complement integer */
    TimeTag(OscTimeTag), /* OSC-timetag */
    Float64(f64), /* 64 bit (“double”) IEEE 754 floating point number */
    AlternateType(String), /* Alternate type represented as an OSC-string (for example, for systems that differentiate “symbols” from “strings”) */
    AsciiCharacter(u8), /* an ascii character, sent as 32 bits */
    Colour(OscColour) /* 32 bit RGBA color */,
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
            OscArgument::Colour(_) => 'r',
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
            bytes.push(0);
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
        OscArgument::TimeTag(t) => t.to_bytes().to_vec(),
        OscArgument::Float64(f) => f.to_be_bytes().to_vec(),
        OscArgument::AlternateType(s) => {
            let mut bytes = s.as_bytes().to_vec();
            bytes.push(0);
            helpers::pad_to_multiple_of_4_bytes(&mut bytes);
            bytes
        },
        OscArgument::AsciiCharacter(c) => (*c as u32).to_be_bytes().to_vec(),
        OscArgument::Colour(c) => vec![c.red, c.green, c.blue, c.alpha],
        OscArgument::MidiMessage(port_id, status_byte, data1, data2) => vec![*port_id, *status_byte, *data1, *data2],
        _ => Vec::new()
      }
    }

    // Returns an OscArgument given a sequence of bytes, an index and a typetag
    pub fn from_bytes(bytes: &[u8], index: &mut usize, typetag: char) -> Result<Self, OscParseError> {
        match typetag {
            // Int32
            'i' => { 
                Ok(OscArgument::Int32(Self::read_be_bytes::<4, _, _>(bytes, index, i32::from_be_bytes)?))
            },
            // Float32
            'f' => { 
                Ok(OscArgument::Float32(Self::read_be_bytes::<4, _, _>(bytes, index, f32::from_be_bytes)?))
            },
            // String 
            's' => { 
                Ok(OscArgument::String(Self::read_osc_string(bytes, index)?))
            },
            // Blob
            'b' => { 
                Self::ensure_bytes_available(bytes, *index, 4)?;
                let blob_size = Self::read_be_bytes::<4, _, _>(bytes, index, i32::from_be_bytes)? as usize;
                Self::ensure_bytes_available(bytes, *index, blob_size)?;
                let data = bytes[*index..*index + blob_size].to_vec();
                *index += blob_size;
                *index += (4 - (blob_size % 4)) % 4;
                Ok(OscArgument::Blob(data))
            },
            // Int64
            'h' => { 
                Ok(OscArgument::Int64(Self::read_be_bytes::<8, _, _>(bytes, index, i64::from_be_bytes)?))
            },
            // TimeTag
            't' => {
                let value = Self::read_be_bytes::<8, _, _>(bytes, index, i64::from_be_bytes)?;
                Ok(OscArgument::TimeTag(OscTimeTag::from_i64 (value)))
            },
            // Float64
            'd' => {
                Ok(OscArgument::Float64(Self::read_be_bytes::<8, _, _>(bytes, index, f64::from_be_bytes)?))
            },
            // Alternate 
            'S' => { 
                Ok(OscArgument::AlternateType(Self::read_osc_string(bytes, index)?))
            },
            // AsciiCharacter
            'c' => {
                Self::ensure_bytes_available(bytes, *index, 4)?;
                Ok(OscArgument::AsciiCharacter(Self::read_be_bytes::<4, _, _>(bytes, index, i32::from_be_bytes)? as u8))
            },
            // Colour
            'r' => {
                Self::ensure_bytes_available(bytes, *index, 4)?;
                let colour = OscColour { red: bytes[*index], green: bytes[*index + 1], blue: bytes[*index + 2], alpha: bytes[*index + 3]};
                *index += 4;
                Ok(OscArgument::Colour(colour))
            },
            // Midi Message
            'm' => {
                Self::ensure_bytes_available(bytes, *index, 4)?;
                let i = *index;
                *index += 4;
                Ok(OscArgument::MidiMessage(bytes[i], bytes[i + 1], bytes[i + 2], bytes[i + 3]))
            },
            'T' => {
                Ok(OscArgument::True)
            },
            'F' => {
                Ok(OscArgument::False)
            },
            'N' => {
                Ok(OscArgument::Nil)
            },
            'I' => {
                Ok(OscArgument::Infinitum)
            },
            '[' => {
                Ok(OscArgument::ArrayBegin)
            },
            ']' => {
                Ok(OscArgument::ArrayEnd)
            },
            _ => {
                Err(OscParseError::UnknownTypeTag)
            }
        }
    }

    fn ensure_bytes_available(bytes: &[u8], index: usize, needed: usize) -> Result<(), OscParseError> {
        if index + needed > bytes.len() {
            Err(OscParseError::NotEnoughData)
        } else {
            Ok(())
        }
    }

    fn read_be_bytes<const N: usize, T, F>( bytes: &[u8], index: &mut usize, f: F,) -> Result<T, OscParseError> where F: Fn([u8; N]) -> T,
    {
        Self::ensure_bytes_available(bytes, *index, N)?;
        let raw: [u8; N] = bytes[*index..*index + N].try_into().unwrap();
        *index += N;
        Ok(f(raw))
    }

    fn read_osc_string(bytes: &[u8], index: &mut usize) -> Result<String, OscParseError> {
        let start = *index;
        let end = bytes[start..]
            .iter()
            .position(|&b| b == 0)
            .ok_or(OscParseError::InvalidFormat)?
            + start;
        let s = String::from_utf8(bytes[start..end].to_vec())
            .map_err(|_| OscParseError::InvalidString)?;
        *index = (end + 1 + 3) & !3; // Advance past null and padding
        Ok(s)
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
        assert_eq!(OscArgument::TimeTag(OscTimeTag::from_i64(0)).type_tag(), 't');
        assert_eq!(OscArgument::Float64(0.0).type_tag(), 'd');
        assert_eq!(OscArgument::AlternateType("".to_string()).type_tag(), 'S');
        assert_eq!(OscArgument::AsciiCharacter('a' as u8).type_tag(), 'c');
        assert_eq!(OscArgument::Colour(OscColour { red: 255, green: 0, blue: 255, alpha: 0 }).type_tag(), 'r');
        assert_eq!(OscArgument::MidiMessage(0, 0, 0, 0).type_tag(), 'm');
        assert_eq!(OscArgument::True.type_tag(), 'T');
        assert_eq!(OscArgument::False.type_tag(), 'F');
        assert_eq!(OscArgument::Nil.type_tag(), 'N');
        assert_eq!(OscArgument::Infinitum.type_tag(), 'I');
        assert_eq!(OscArgument::ArrayBegin.type_tag(), '[');
        assert_eq!(OscArgument::ArrayEnd.type_tag(), ']');
    }

    #[test]
    fn test_arguments_to_bytes() {
        assert_eq!(OscArgument::Int32(123).to_bytes(), 123_i32.to_be_bytes());
        assert_eq!(OscArgument::Float32(123.456).to_bytes(), 123.456_f32.to_be_bytes());
        assert_eq!(OscArgument::String("hello".to_string()).to_bytes(), helpers::osc_string_as_bytes("hello"));
        assert_eq!(OscArgument::Blob(vec![0x01, 0x02, 0x03]).to_bytes(), vec![0x00, 0x00, 0x00, 0x03, 0x01, 0x02, 0x03, 0x00]);
        assert_eq!(OscArgument::Int64(0x0123_4567_89AB_CDEF).to_bytes(), (0x0123_4567_89AB_CDEF as i64).to_be_bytes());
        assert_eq!(OscArgument::TimeTag(OscTimeTag { seconds: 0x12345678, fractional: 0x9ABCDEF0 }).to_bytes(), vec![0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC, 0xDE, 0xF0]);
        assert_eq!(OscArgument::Float64(123456789.1234).to_bytes(), (123456789.1234 as f64).to_be_bytes());
        assert_eq!(OscArgument::AlternateType("alternate".to_string()).to_bytes(), helpers::osc_string_as_bytes("alternate"));
        assert_eq!(OscArgument::AsciiCharacter('A' as u8).to_bytes(), vec![0x00, 0x00, 0x00, 0x41]);
        assert_eq!(OscArgument::Colour(OscColour { red: 255, green: 0, blue: 255, alpha: 0 }).to_bytes(), vec![255, 0, 255, 0]);
        assert_eq!(OscArgument::MidiMessage(0, 10, 20, 30).to_bytes(), vec![0, 10, 20, 30]);
        assert_eq!(OscArgument::True.to_bytes(), vec![]);
        assert_eq!(OscArgument::False.to_bytes(), vec![]);
        assert_eq!(OscArgument::Nil.to_bytes(), vec![]);
        assert_eq!(OscArgument::Infinitum.to_bytes(), vec![]);
        assert_eq!(OscArgument::ArrayBegin.to_bytes(), vec![]);
        assert_eq!(OscArgument::ArrayEnd.to_bytes(), vec![]);
    }

    #[test]
    fn test_arguments_from_bytes() {
        assert_eq!(OscArgument::from_bytes(&123_i32.to_be_bytes(), &mut 0usize.clone(), 'i'), Ok(OscArgument::Int32(123)));
        assert_eq!(OscArgument::from_bytes(&567.3_f32.to_be_bytes(), &mut 0usize.clone(), 'f'), Ok(OscArgument::Float32(567.3)));
        assert_eq!(OscArgument::from_bytes(&helpers::osc_string_as_bytes("hello"), &mut 0usize.clone(), 's'), Ok(OscArgument::String("hello".to_string())));
        assert_eq!(OscArgument::from_bytes(&vec![0x00, 0x00, 0x00, 0x03, 0x01, 0x02, 0x03, 0x00], &mut 0usize.clone(), 'b'), Ok(OscArgument::Blob(vec![0x01, 0x02, 0x03])));
        assert_eq!(OscArgument::from_bytes(&(0x0123_4567_89AB_CDEF as i64).to_be_bytes(), &mut 0usize.clone(), 'h'), Ok(OscArgument::Int64(0x0123_4567_89AB_CDEF)));
        assert_eq!(OscArgument::from_bytes(&vec![0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC, 0xDE, 0xF0], &mut 0usize.clone(), 't'), Ok(OscArgument::TimeTag(OscTimeTag { seconds: 0x12345678, fractional: 0x9ABCDEF0 })));
        assert_eq!(OscArgument::from_bytes(&(123456789.1234 as f64).to_be_bytes(), &mut 0usize.clone(), 'd'), Ok(OscArgument::Float64(123456789.1234)));
        assert_eq!(OscArgument::from_bytes(&helpers::osc_string_as_bytes("alternate"), &mut 0usize.clone(), 'S'), Ok(OscArgument::AlternateType("alternate".to_string())));
        assert_eq!(OscArgument::from_bytes(&vec![0x00, 0x00, 0x00, 0x41], &mut 0usize.clone(), 'c'), Ok(OscArgument::AsciiCharacter('A' as u8)));
        assert_eq!(OscArgument::from_bytes(&vec![255, 87, 123, 255], &mut 0usize.clone(), 'r'), Ok(OscArgument::Colour(OscColour { red: 255, green: 87, blue: 123, alpha: 255 })));
        assert_eq!(OscArgument::from_bytes(&vec![0, 10, 20, 30], &mut 0usize.clone(), 'm'), Ok(OscArgument::MidiMessage(0, 10, 20, 30)));
        assert_eq!(OscArgument::from_bytes(&vec![8, 9, 10, 11], &mut 0usize.clone(), 'T'), Ok(OscArgument::True));
        assert_eq!(OscArgument::from_bytes(&vec![8, 9, 10, 11], &mut 0usize.clone(), 'F'), Ok(OscArgument::False));
        assert_eq!(OscArgument::from_bytes(&vec![8, 9, 10, 11], &mut 0usize.clone(), 'N'), Ok(OscArgument::Nil));
        assert_eq!(OscArgument::from_bytes(&vec![8, 9, 10, 11], &mut 0usize.clone(), 'I'), Ok(OscArgument::Infinitum));
        assert_eq!(OscArgument::from_bytes(&vec![8, 9, 10, 11], &mut 0usize.clone(), '['), Ok(OscArgument::ArrayBegin));
        assert_eq!(OscArgument::from_bytes(&vec![8, 9, 10, 11], &mut 0usize.clone(), ']'), Ok(OscArgument::ArrayEnd));
    }
}