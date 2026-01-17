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

    /// Returns a human-readable string representation of the argument
    pub fn to_string(&self) -> String {
        match self {
            OscArgument::Int32(x) => x.to_string(),
            OscArgument::Float32(f) => f.to_string(),
            OscArgument::String(s) => format!("\"{}\"", s),
            OscArgument::Blob(data) => format!("Blob({} bytes)", data.len()),
            OscArgument::Int64(i) => i.to_string(),
            OscArgument::TimeTag(t) => format!("TimeTag({}:{})", t.seconds, t.fractional),
            OscArgument::Float64(f) => f.to_string(),
            OscArgument::AlternateType(s) => format!("Alt(\"{}\")", s),
            OscArgument::AsciiCharacter(c) => {
                if c.is_ascii_graphic() || *c == b' ' {
                    format!("'{}'", *c as char)
                } else {
                    format!("'\\x{:02x}'", c)
                }
            },
            OscArgument::Colour(c) => format!("rgba({}, {}, {}, {})", c.red, c.green, c.blue, c.alpha),
            OscArgument::MidiMessage(port, status, data1, data2) => {
                format!("MIDI(port:{}, status:0x{:02x}, data1:{}, data2:{})", port, status, data1, data2)
            },
            OscArgument::True => "true".to_string(),
            OscArgument::False => "false".to_string(),
            OscArgument::Nil => "nil".to_string(),
            OscArgument::Infinitum => "inf".to_string(),
            OscArgument::ArrayBegin => "[".to_string(),
            OscArgument::ArrayEnd => "]".to_string(),
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

        // Calculate length including null terminator
        let string_length = end - start + 1; // +1 for null
        // Pad to next multiple of 4
        let padded_length = (string_length + 3) & !3;
        *index = start + padded_length;

        Ok(s)
    }
}

//==================================================================
// Tests
#[cfg(test)]
mod tests {
    use std::i64;

    use super::*;

    //----------------------------------------------------------------
    // Test type tags
    #[test]
    fn test_type_tag() {
        let test_cases = vec![
            (OscArgument::Int32(0), 'i'),
            (OscArgument::Float32(0.0), 'f'),
            (OscArgument::String("".to_string()), 's'),
            (OscArgument::Blob(vec![]), 'b'),
            (OscArgument::Int64(0), 'h'),
            (OscArgument::TimeTag(OscTimeTag::from_i64(0)), 't'),
            (OscArgument::Float64(0.0), 'd'),
            (OscArgument::AlternateType("".to_string()), 'S'),
            (OscArgument::AsciiCharacter(b'a'), 'c'),
            (OscArgument::Colour(OscColour { red: 255, green: 0, blue: 255, alpha: 0 }), 'r'),
            (OscArgument::MidiMessage(0, 0, 0, 0), 'm'),
            (OscArgument::True, 'T'),
            (OscArgument::False, 'F'),
            (OscArgument::Nil, 'N'),
            (OscArgument::Infinitum, 'I'),
            (OscArgument::ArrayBegin, '['),
            (OscArgument::ArrayEnd, ']'),
        ];

        for (arg, expected_tag) in test_cases {
            assert_eq!(arg.type_tag(), expected_tag, "Failed for {:?}", arg);
        }
    }

    //----------------------------------------------------------------
    // Test argument to bytes conversion
    #[test]
    fn test_arguments_to_bytes_int32() {
        assert_eq!(OscArgument::Int32(123).to_bytes(), 123_i32.to_be_bytes());
        assert_eq!(OscArgument::Int32(0).to_bytes(), 0_i32.to_be_bytes());
        assert_eq!(OscArgument::Int32(-123).to_bytes(), (-123_i32).to_be_bytes());
        assert_eq!(OscArgument::Int32(i32::MAX).to_bytes(), i32::MAX.to_be_bytes());
        assert_eq!(OscArgument::Int32(i32::MIN).to_bytes(), i32::MIN.to_be_bytes());
    }

    #[test]
    fn test_arguments_to_bytes_float32() {
        assert_eq!(OscArgument::Float32(123.456).to_bytes(), 123.456_f32.to_be_bytes());
        assert_eq!(OscArgument::Float32(0.0).to_bytes(), 0.0_f32.to_be_bytes());
        assert_eq!(OscArgument::Float32(-123.456).to_bytes(), (-123.456_f32).to_be_bytes());
        assert_eq!(OscArgument::Float32(f32::MAX).to_bytes(), f32::MAX.to_be_bytes());
        assert_eq!(OscArgument::Float32(f32::MIN).to_bytes(), f32::MIN.to_be_bytes());
        assert_eq!(OscArgument::Float32(f32::INFINITY).to_bytes(), f32::INFINITY.to_be_bytes());
        assert_eq!(OscArgument::Float32(f32::NEG_INFINITY).to_bytes(), f32::NEG_INFINITY.to_be_bytes());
        assert_eq!(OscArgument::Float32(-0.0).to_bytes(), (-0.0_f32).to_be_bytes());
    }

    #[test]
    fn test_arguments_to_bytes_string() {
        assert_eq!(OscArgument::String("".to_string()).to_bytes(), vec![0x00, 0x00, 0x00, 0x00]); // empty string = null + 3 pad
        assert_eq!(OscArgument::String("x".to_string()).to_bytes(), vec![b'x', 0x00, 0x00, 0x00]); // 1 char + null + 2 pad
        assert_eq!(OscArgument::String("hi".to_string()).to_bytes(), vec![b'h', b'i', 0x00, 0x00]); // 2 chars + null + 1 pad
        assert_eq!(OscArgument::String("abc".to_string()).to_bytes(), vec![b'a', b'b', b'c', 0x00]); // 3 chars + null = 4 bytes
        assert_eq!(OscArgument::String("word".to_string()).to_bytes(), vec![b'w', b'o', b'r', b'd', 0x00, 0x00, 0x00, 0x00]); // 4 chars + null + 3 pad = 8 bytes
        assert_eq!(OscArgument::String("hello".to_string()).to_bytes(), vec![b'h', b'e', b'l', b'l', b'o', 0x00, 0x00, 0x00]); // 5 chars + null + 2 pad = 8 bytes
    }

    #[test]
    fn test_arguments_to_bytes_blob() {
        // Empty blob
        assert_eq!(OscArgument::Blob(vec![]).to_bytes(), vec![0x00, 0x00, 0x00, 0x00]);

        // 1 byte + 3 pad
        assert_eq!(OscArgument::Blob(vec![0xFF]).to_bytes(), vec![0x00, 0x00, 0x00, 0x01, 0xFF, 0x00, 0x00, 0x00]);

        // 2 bytes + 2 pad
        assert_eq!(OscArgument::Blob(vec![0xAA, 0xBB]).to_bytes(), vec![0x00, 0x00, 0x00, 0x02, 0xAA, 0xBB, 0x00, 0x00]);

        // 3 bytes + 1 pad
        assert_eq!(OscArgument::Blob(vec![0x01, 0x02, 0x03]).to_bytes(), vec![0x00, 0x00, 0x00, 0x03, 0x01, 0x02, 0x03, 0x00]);
        
        // Blob that's exactly 4 bytes (no padding needed)
        assert_eq!(OscArgument::Blob(vec![0x01, 0x02, 0x03, 0x04]).to_bytes(), vec![0x00, 0x00, 0x00, 0x04, 0x01, 0x02, 0x03, 0x04]);
    }

    #[test]
    fn test_arguments_to_bytes_int64() {
        assert_eq!(OscArgument::Int64(0x0123_4567_89AB_CDEF).to_bytes(), (0x0123_4567_89AB_CDEF as i64).to_be_bytes());
        assert_eq!(OscArgument::Int64(0).to_bytes(), 0_i64.to_be_bytes());
        assert_eq!(OscArgument::Int64(-10000).to_bytes(), (-10000_i64).to_be_bytes());
        assert_eq!(OscArgument::Int64(i64::MAX).to_bytes(), i64::MAX.to_be_bytes());
        assert_eq!(OscArgument::Int64(i64::MIN).to_bytes(), i64::MIN.to_be_bytes());
    }

    #[test]
    fn test_arguments_to_bytes_timetag() {
        assert_eq!(OscArgument::TimeTag(OscTimeTag { seconds: 0x12345678, fractional: 0x9ABCDEF0 }).to_bytes(), vec![0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC, 0xDE, 0xF0]);
    }

    #[test]
    fn test_arguments_to_bytes_float64() {
        assert_eq!(OscArgument::Float64(123456789.1234).to_bytes(), (123456789.1234 as f64).to_be_bytes());
        assert_eq!(OscArgument::Float64(0.0).to_bytes(), (0.0_f64).to_be_bytes());
        assert_eq!(OscArgument::Float64(-10000.0).to_bytes(), (-10000.0_f64).to_be_bytes());
        assert_eq!(OscArgument::Float64(f64::MAX).to_bytes(), f64::MAX.to_be_bytes());
        assert_eq!(OscArgument::Float64(f64::MIN).to_bytes(), f64::MIN.to_be_bytes());
        assert_eq!(OscArgument::Float64(f64::INFINITY).to_bytes(), f64::INFINITY.to_be_bytes());
        assert_eq!(OscArgument::Float64(f64::NEG_INFINITY).to_bytes(), f64::NEG_INFINITY.to_be_bytes());
        assert_eq!(OscArgument::Float64(-0.0).to_bytes(), (-0.0_f64).to_be_bytes());
    }

    #[test]
    fn test_arguments_to_bytes_alternate_type() {
        assert_eq!(OscArgument::AlternateType("alternate".to_string()).to_bytes(), helpers::osc_string_as_bytes("alternate"));
        assert_eq!(OscArgument::AlternateType("".to_string()).to_bytes(), vec![0x00, 0x00, 0x00, 0x00]);
        assert_eq!(OscArgument::AlternateType("yo".to_string()).to_bytes(), vec![b'y', b'o', 0x00, 0x00]); // 2 chars + null + 1 pad
        assert_eq!(OscArgument::AlternateType("xyz".to_string()).to_bytes(), vec![b'x', b'y', b'z', 0x00]); // 3 chars + null = 4 bytes
    }

    #[test]
    fn test_arguments_to_bytes_ascii() {
        // Test printable ASCII
        assert_eq!(OscArgument::AsciiCharacter('A' as u8).to_bytes(), vec![0x00, 0x00, 0x00, 0x41]);
        assert_eq!(OscArgument::AsciiCharacter('B' as u8).to_bytes(), vec![0x00, 0x00, 0x00, 0x42]);
        assert_eq!(OscArgument::AsciiCharacter('C' as u8).to_bytes(), vec![0x00, 0x00, 0x00, 0x43]);
        assert_eq!(OscArgument::AsciiCharacter('x' as u8).to_bytes(), vec![0x00, 0x00, 0x00, 0x78]);
        assert_eq!(OscArgument::AsciiCharacter('y' as u8).to_bytes(), vec![0x00, 0x00, 0x00, 0x79]);
        assert_eq!(OscArgument::AsciiCharacter('z' as u8).to_bytes(), vec![0x00, 0x00, 0x00, 0x7A]);

        // Test non-printable ASCII
        assert_eq!(OscArgument::AsciiCharacter(0).to_bytes(), vec![0x00, 0x00, 0x00, 0x00]);
        assert_eq!(OscArgument::AsciiCharacter(127).to_bytes(), vec![0x00, 0x00, 0x00, 0x7F]);

        // Test extended ASCII (might want to validate this is intentional)
        assert_eq!(OscArgument::AsciiCharacter(255).to_bytes(), vec![0x00, 0x00, 0x00, 0xFF]);
    }

    #[test]
    fn test_arguments_to_bytes_colour() {
        // Test all combinations of rgba 
        for r in [0, 128, 255] {
            for g in [0, 128, 255] {
                for b in [0, 128, 255] {
                    for a in [0, 128, 255] {
                        let colour = OscColour { red: r, green: g, blue: b, alpha: a };
                        assert_eq!(OscArgument::Colour(colour).to_bytes(), vec![r, g, b, a]);
                    }
                }
            }
        }
    }

    #[test]
    fn test_arguments_to_bytes_midi() {
        assert_eq!(OscArgument::MidiMessage(0, 10, 20, 30).to_bytes(), vec![0, 10, 20, 30]);
        assert_eq!(OscArgument::MidiMessage(255, 128, 64, 32).to_bytes(), vec![255, 128, 64, 32]);
        assert_eq!(OscArgument::MidiMessage(0, 0, 0, 0).to_bytes(), vec![0, 0, 0, 0]);
        assert_eq!(OscArgument::MidiMessage(255, 255, 255, 255).to_bytes(), vec![255, 255, 255, 255]);
    }

    #[test]
    fn test_arguments_to_bytes_other() {
        assert_eq!(OscArgument::True.to_bytes(), vec![]);
        assert_eq!(OscArgument::False.to_bytes(), vec![]);
        assert_eq!(OscArgument::Nil.to_bytes(), vec![]);
        assert_eq!(OscArgument::Infinitum.to_bytes(), vec![]);
        assert_eq!(OscArgument::ArrayBegin.to_bytes(), vec![]);
        assert_eq!(OscArgument::ArrayEnd.to_bytes(), vec![]);
    }

    //----------------------------------------------------------------
    // Test argument from bytes conversion
    #[test]
    fn test_arguments_from_bytes_int32() {

        // Simple case
        let mut index = 0;
        assert_eq!(
            OscArgument::from_bytes(&123_i32.to_be_bytes(), &mut index, 'i'),
            Ok(OscArgument::Int32(123))
        );
        assert_eq!(index, 4);

        // Negative number
        index = 0;
        assert_eq!(
            OscArgument::from_bytes(&(-456_i32).to_be_bytes(), &mut index, 'i'),
            Ok(OscArgument::Int32(-456))
        );
        assert_eq!(index, 4);

        // MAX value
        index = 0;
        assert_eq!(
            OscArgument::from_bytes(&i32::MAX.to_be_bytes(), &mut index, 'i'),
            Ok(OscArgument::Int32(i32::MAX))
        );
        assert_eq!(index, 4);

        // MIN value
        index = 0;
        assert_eq!(
            OscArgument::from_bytes(&i32::MIN.to_be_bytes(), &mut index, 'i'),
            Ok(OscArgument::Int32(i32::MIN))
        );
        assert_eq!(index, 4);

        // Test with non-zero index (simulating sequential parsing)
        let data = [
            0xFF, 0xFF, 0xFF, 0xFF,  // junk padding (4 bytes)
            0x00, 0x00, 0x01, 0x2C,  // 300 in big-endian (4 bytes)
            0xAA, 0xBB, 0xCC, 0xDD,  // more junk after (4 bytes)
        ];
        index = 4; // Start after the padding
        assert_eq!(
            OscArgument::from_bytes(&data, &mut index, 'i'),
            Ok(OscArgument::Int32(300))
        );
        assert_eq!(index, 8); // Should advance by 4 bytes (from 4 to 8)

        // Test with different starting index
        let data2 = [
            0xAA, 0xBB,              // 2 bytes padding
            0xFF, 0xFF, 0xFE, 0x0C,  // -500 in big-endian
            0xCC, 0xDD,              // 2 bytes after
        ];
        index = 2;
        assert_eq!(
            OscArgument::from_bytes(&data2, &mut index, 'i'),
            Ok(OscArgument::Int32(-500))
        );
        assert_eq!(index, 6);
    }

    #[test]
    fn test_arguments_from_bytes_float32() {
        // Simple case
        let mut index = 0;
        assert_eq!(
            OscArgument::from_bytes(&567.3_f32.to_be_bytes(), &mut index, 'f'),
            Ok(OscArgument::Float32(567.3))
        );
        assert_eq!(index, 4);

        // Negative number
        index = 0;
        assert_eq!(
            OscArgument::from_bytes(&(-123.456_f32).to_be_bytes(), &mut index, 'f'),
            Ok(OscArgument::Float32(-123.456))
        );
        assert_eq!(index, 4);

        // Zero
        index = 0;
        assert_eq!(
            OscArgument::from_bytes(&0.0_f32.to_be_bytes(), &mut index, 'f'),
            Ok(OscArgument::Float32(0.0))
        );
        assert_eq!(index, 4);

        // Negative zero
        index = 0;
        assert_eq!(
            OscArgument::from_bytes(&(-0.0_f32).to_be_bytes(), &mut index, 'f'),
            Ok(OscArgument::Float32(-0.0))
        );
        assert_eq!(index, 4);

        // Infinity
        index = 0;
        assert_eq!(
            OscArgument::from_bytes(&f32::INFINITY.to_be_bytes(), &mut index, 'f'),
            Ok(OscArgument::Float32(f32::INFINITY))
        );
        assert_eq!(index, 4);

        // Negative infinity
        index = 0;
        assert_eq!(
            OscArgument::from_bytes(&f32::NEG_INFINITY.to_be_bytes(), &mut index, 'f'),
            Ok(OscArgument::Float32(f32::NEG_INFINITY))
        );
        assert_eq!(index, 4);

        // MAX value
        index = 0;
        assert_eq!(
            OscArgument::from_bytes(&f32::MAX.to_be_bytes(), &mut index, 'f'),
            Ok(OscArgument::Float32(f32::MAX))
        );
        assert_eq!(index, 4);

        // MIN value
        index = 0;
        assert_eq!(
            OscArgument::from_bytes(&f32::MIN.to_be_bytes(), &mut index, 'f'),
            Ok(OscArgument::Float32(f32::MIN))
        );
        assert_eq!(index, 4);

        // Test with non-zero index (simulating sequential parsing)
        let data = [
            0xFF, 0xFF, 0xFF, 0xFF,  // junk padding (4 bytes)
            0x43, 0x96, 0x00, 0x00,  // 300.0 in big-endian (4 bytes)
            0xAA, 0xBB, 0xCC, 0xDD,  // more junk after (4 bytes)
        ];
        index = 4; // Start after the padding
        assert_eq!(
            OscArgument::from_bytes(&data, &mut index, 'f'),
            Ok(OscArgument::Float32(300.0))
        );
        assert_eq!(index, 8); // Should advance by 4 bytes (from 4 to 8)

        // Test with different starting index
        let data2 = [
            0xAA, 0xBB,              // 2 bytes padding
            0xC2, 0xFA, 0x00, 0x00,  // -125.0 in big-endian
            0xCC, 0xDD,              // 2 bytes after
        ];
        index = 2;
        assert_eq!(
            OscArgument::from_bytes(&data2, &mut index, 'f'),
            Ok(OscArgument::Float32(-125.0))
        );
        assert_eq!(index, 6);
    }

    #[test]
    fn test_arguments_from_bytes_string() {
        // Simple case
        let mut index = 0;
        assert_eq!(
            OscArgument::from_bytes(&helpers::osc_string_as_bytes("hello"), &mut index, 's'),
            Ok(OscArgument::String("hello".to_string()))
        );
        assert_eq!(index, 8); // "hello" = 5 + null + 2 pad = 8

        // Empty string
        index = 0;
        assert_eq!(
            OscArgument::from_bytes(&[0x00, 0x00, 0x00, 0x00], &mut index, 's'),
            Ok(OscArgument::String("".to_string()))
        );
        assert_eq!(index, 4);

        // 1 character (1 + null + 2 pad = 4)
        index = 0;
        assert_eq!(
            OscArgument::from_bytes(&[b'a', 0x00, 0x00, 0x00], &mut index, 's'),
            Ok(OscArgument::String("a".to_string()))
        );
        assert_eq!(index, 4);

        // 2 characters (2 + null + 1 pad = 4)
        index = 0;
        assert_eq!(
            OscArgument::from_bytes(&[b'h', b'i', 0x00, 0x00], &mut index, 's'),
            Ok(OscArgument::String("hi".to_string()))
        );
        assert_eq!(index, 4);

        // 3 characters (3 + null = 4, no padding needed)
        index = 0;
        assert_eq!(
            OscArgument::from_bytes(&[b'a', b'b', b'c', 0x00], &mut index, 's'),
            Ok(OscArgument::String("abc".to_string()))
        );
        assert_eq!(index, 4);

        // 4 characters (4 + null + 3 pad = 8)
        index = 0;
        assert_eq!(
            OscArgument::from_bytes(&[b'a', b'b', b'c', b'd', 0x00, 0x00, 0x00, 0x00], &mut index, 's'),
            Ok(OscArgument::String("abcd".to_string()))
        );
        assert_eq!(index, 8);

        // Longer string (7 + null = 8, no padding needed)
        index = 0;
        assert_eq!(
            OscArgument::from_bytes(&[b't', b'e', b's', b't', b'i', b'n', b'g', 0x00], &mut index, 's'),
            Ok(OscArgument::String("testing".to_string()))
        );
        assert_eq!(index, 8);

        // Test with non-zero index (simulating sequential parsing)
        let data = [
            0xFF, 0xFF, 0xFF, 0xFF,  // junk padding (4 bytes)
            b'o', b's', b'c', 0x00,  // "osc" + null = 4 bytes
            0xAA, 0xBB, 0xCC, 0xDD,  // more junk after (4 bytes)
        ];
        index = 4; // Start after the padding
        assert_eq!(
            OscArgument::from_bytes(&data, &mut index, 's'),
            Ok(OscArgument::String("osc".to_string()))
        );
        assert_eq!(index, 8); // Should advance by 4 bytes (from 4 to 8)

        // Test with different starting index and longer string
        let data2 = [
            0xAA, 0xBB,                              // 2 bytes padding
            b'm', b'u', b's', b'i', b'c', 0x00, 0x00, 0x00,  // "music" (5 + null + 2 pad = 8)
            0xCC, 0xDD,                              // 2 bytes after
        ];
        index = 2;
        assert_eq!(
            OscArgument::from_bytes(&data2, &mut index, 's'),
            Ok(OscArgument::String("music".to_string()))
        );
        assert_eq!(index, 10); // 2 + 8 = 10
    }

    #[test]
    fn test_arguments_from_bytes_blob() {
        // Empty blob
        let mut index = 0;
        assert_eq!(
            OscArgument::from_bytes(&[0x00, 0x00, 0x00, 0x00], &mut index, 'b'),
            Ok(OscArgument::Blob(vec![]))
        );
        assert_eq!(index, 4); // Just the length field

        // 1-byte blob (3 bytes padding)
        index = 0;
        assert_eq!(
            OscArgument::from_bytes(&[0x00, 0x00, 0x00, 0x01, 0xFF, 0x00, 0x00, 0x00], &mut index, 'b'),
            Ok(OscArgument::Blob(vec![0xFF]))
        );
        assert_eq!(index, 8); // 4 (length) + 1 (data) + 3 (pad) = 8

        // 2-byte blob (2 bytes padding)
        index = 0;
        assert_eq!(
            OscArgument::from_bytes(&[0x00, 0x00, 0x00, 0x02, 0xAA, 0xBB, 0x00, 0x00], &mut index, 'b'),
            Ok(OscArgument::Blob(vec![0xAA, 0xBB]))
        );
        assert_eq!(index, 8); // 4 (length) + 2 (data) + 2 (pad) = 8

        // 3-byte blob (1 byte padding)
        index = 0;
        assert_eq!(
            OscArgument::from_bytes(&[0x00, 0x00, 0x00, 0x03, 0x01, 0x02, 0x03, 0x00], &mut index, 'b'),
            Ok(OscArgument::Blob(vec![0x01, 0x02, 0x03]))
        );
        assert_eq!(index, 8); // 4 (length) + 3 (data) + 1 (pad) = 8

        // 4-byte blob (no padding)
        index = 0;
        assert_eq!(
            OscArgument::from_bytes(&[0x00, 0x00, 0x00, 0x04, 0x01, 0x02, 0x03, 0x04], &mut index, 'b'),
            Ok(OscArgument::Blob(vec![0x01, 0x02, 0x03, 0x04]))
        );
        assert_eq!(index, 8); // 4 (length) + 4 (data) + 0 (pad) = 8

        // 5-byte blob (3 bytes padding)
        index = 0;
        assert_eq!(
            OscArgument::from_bytes(&[0x00, 0x00, 0x00, 0x05, 0x10, 0x20, 0x30, 0x40, 0x50, 0x00, 0x00, 0x00], &mut index, 'b'),
            Ok(OscArgument::Blob(vec![0x10, 0x20, 0x30, 0x40, 0x50]))
        );
        assert_eq!(index, 12); // 4 (length) + 5 (data) + 3 (pad) = 12

        // 8-byte blob (no padding)
        index = 0;
        assert_eq!(
            OscArgument::from_bytes(&[0x00, 0x00, 0x00, 0x08, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08], &mut index, 'b'),
            Ok(OscArgument::Blob(vec![0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08]))
        );
        assert_eq!(index, 12); // 4 (length) + 8 (data) + 0 (pad) = 12

        // Test with non-zero index (simulating sequential parsing)
        let data = [
            0xFF, 0xFF, 0xFF, 0xFF,  // junk padding (4 bytes)
            0x00, 0x00, 0x00, 0x02,  // length = 2
            0xAB, 0xCD, 0x00, 0x00,  // 2 bytes data + 2 pad
            0xAA, 0xBB, 0xCC, 0xDD,  // more junk after
        ];
        index = 4; // Start after the junk padding
        assert_eq!(
            OscArgument::from_bytes(&data, &mut index, 'b'),
            Ok(OscArgument::Blob(vec![0xAB, 0xCD]))
        );
        assert_eq!(index, 12); // 4 + 4 (length) + 2 (data) + 2 (pad) = 12

        // Test with different starting index
        let data2 = [
            0xAA, 0xBB,              // 2 bytes padding
            0x00, 0x00, 0x00, 0x03,  // length = 3
            0x11, 0x22, 0x33, 0x00,  // 3 bytes data + 1 pad
            0xCC, 0xDD,              // 2 bytes after
        ];
        index = 2;
        assert_eq!(
            OscArgument::from_bytes(&data2, &mut index, 'b'),
            Ok(OscArgument::Blob(vec![0x11, 0x22, 0x33]))
        );
        assert_eq!(index, 10); // 2 + 4 (length) + 3 (data) + 1 (pad) = 10
    }

    #[test]
    fn test_arguments_from_bytes_int64() {
        // Simple case
        let mut index = 0;
        assert_eq!(
            OscArgument::from_bytes(&0x0123_4567_89AB_CDEF_i64.to_be_bytes(), &mut index, 'h'),
            Ok(OscArgument::Int64(0x0123_4567_89AB_CDEF))
        );
        assert_eq!(index, 8);

        // Zero
        index = 0;
        assert_eq!(
            OscArgument::from_bytes(&0_i64.to_be_bytes(), &mut index, 'h'),
            Ok(OscArgument::Int64(0))
        );
        assert_eq!(index, 8);

        // Negative number
        index = 0;
        assert_eq!(
            OscArgument::from_bytes(&(-10000_i64).to_be_bytes(), &mut index, 'h'),
            Ok(OscArgument::Int64(-10000))
        );
        assert_eq!(index, 8);

        // MAX value
        index = 0;
        assert_eq!(
            OscArgument::from_bytes(&i64::MAX.to_be_bytes(), &mut index, 'h'),
            Ok(OscArgument::Int64(i64::MAX))
        );
        assert_eq!(index, 8);

        // MIN value
        index = 0;
        assert_eq!(
            OscArgument::from_bytes(&i64::MIN.to_be_bytes(), &mut index, 'h'),
            Ok(OscArgument::Int64(i64::MIN))
        );
        assert_eq!(index, 8);

        // Test with non-zero index (simulating sequential parsing)
        let data = [
            0xFF, 0xFF, 0xFF, 0xFF,  // junk padding (4 bytes)
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x03, 0xE8,  // 1000 in big-endian (8 bytes)
            0xAA, 0xBB, 0xCC, 0xDD,  // more junk after (4 bytes)
        ];
        index = 4; // Start after the padding
        assert_eq!(
            OscArgument::from_bytes(&data, &mut index, 'h'),
            Ok(OscArgument::Int64(1000))
        );
        assert_eq!(index, 12); // Should advance by 8 bytes (from 4 to 12)

        // Test with different starting index
        let data2 = [
            0xAA, 0xBB,              // 2 bytes padding
            0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFC, 0x18,  // -1000 in big-endian
            0xCC, 0xDD,              // 2 bytes after
        ];
        index = 2;
        assert_eq!(
            OscArgument::from_bytes(&data2, &mut index, 'h'),
            Ok(OscArgument::Int64(-1000))
        );
        assert_eq!(index, 10); // 2 + 8 = 10
    }

    #[test]
    fn test_arguments_from_bytes_timetag() {
        // Simple case
        let mut index = 0;
        assert_eq!(
            OscArgument::from_bytes(&[0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC, 0xDE, 0xF0], &mut index, 't'),
            Ok(OscArgument::TimeTag(OscTimeTag { seconds: 0x12345678, fractional: 0x9ABCDEF0 }))
        );
        assert_eq!(index, 8);

        // Zero timetag (immediate)
        index = 0;
        assert_eq!(
            OscArgument::from_bytes(&[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00], &mut index, 't'),
            Ok(OscArgument::TimeTag(OscTimeTag { seconds: 0, fractional: 0 }))
        );
        assert_eq!(index, 8);

        // Max seconds, zero fractional
        index = 0;
        assert_eq!(
            OscArgument::from_bytes(&[0xFF, 0xFF, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00], &mut index, 't'),
            Ok(OscArgument::TimeTag(OscTimeTag { seconds: 0xFFFFFFFF, fractional: 0 }))
        );
        assert_eq!(index, 8);

        // Zero seconds, max fractional
        index = 0;
        assert_eq!(
            OscArgument::from_bytes(&[0x00, 0x00, 0x00, 0x00, 0xFF, 0xFF, 0xFF, 0xFF], &mut index, 't'),
            Ok(OscArgument::TimeTag(OscTimeTag { seconds: 0, fractional: 0xFFFFFFFF }))
        );
        assert_eq!(index, 8);

        // Max timetag
        index = 0;
        assert_eq!(
            OscArgument::from_bytes(&[0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF], &mut index, 't'),
            Ok(OscArgument::TimeTag(OscTimeTag { seconds: 0xFFFFFFFF, fractional: 0xFFFFFFFF }))
        );
        assert_eq!(index, 8);

        // Test with non-zero index (simulating sequential parsing)
        let data = [
            0xFF, 0xFF, 0xFF, 0xFF,  // junk padding (4 bytes)
            0x00, 0x00, 0x00, 0x01, 0x80, 0x00, 0x00, 0x00,  // 1 second + 0.5 fractional (8 bytes)
            0xAA, 0xBB, 0xCC, 0xDD,  // more junk after (4 bytes)
        ];
        index = 4; // Start after the padding
        assert_eq!(
            OscArgument::from_bytes(&data, &mut index, 't'),
            Ok(OscArgument::TimeTag(OscTimeTag { seconds: 1, fractional: 0x80000000 }))
        );
        assert_eq!(index, 12); // Should advance by 8 bytes (from 4 to 12)

        // Test with different starting index
        let data2 = [
            0xAA, 0xBB,              // 2 bytes padding
            0xAB, 0xCD, 0xEF, 0x01, 0x23, 0x45, 0x67, 0x89,  // arbitrary timetag (8 bytes)
            0xCC, 0xDD,              // 2 bytes after
        ];
        index = 2;
        assert_eq!(
            OscArgument::from_bytes(&data2, &mut index, 't'),
            Ok(OscArgument::TimeTag(OscTimeTag { seconds: 0xABCDEF01, fractional: 0x23456789 }))
        );
        assert_eq!(index, 10); // 2 + 8 = 10
    }

    #[test]
    fn test_arguments_from_bytes_float64() {
        // Simple case
        let mut index = 0;
        assert_eq!(
            OscArgument::from_bytes(&123456789.1234_f64.to_be_bytes(), &mut index, 'd'),
            Ok(OscArgument::Float64(123456789.1234))
        );
        assert_eq!(index, 8);

        // Zero
        index = 0;
        assert_eq!(
            OscArgument::from_bytes(&0.0_f64.to_be_bytes(), &mut index, 'd'),
            Ok(OscArgument::Float64(0.0))
        );
        assert_eq!(index, 8);

        // Negative number
        index = 0;
        assert_eq!(
            OscArgument::from_bytes(&(-10000.0_f64).to_be_bytes(), &mut index, 'd'),
            Ok(OscArgument::Float64(-10000.0))
        );
        assert_eq!(index, 8);

        // Negative zero
        index = 0;
        assert_eq!(
            OscArgument::from_bytes(&(-0.0_f64).to_be_bytes(), &mut index, 'd'),
            Ok(OscArgument::Float64(-0.0))
        );
        assert_eq!(index, 8);

        // Infinity
        index = 0;
        assert_eq!(
            OscArgument::from_bytes(&f64::INFINITY.to_be_bytes(), &mut index, 'd'),
            Ok(OscArgument::Float64(f64::INFINITY))
        );
        assert_eq!(index, 8);

        // Negative infinity
        index = 0;
        assert_eq!(
            OscArgument::from_bytes(&f64::NEG_INFINITY.to_be_bytes(), &mut index, 'd'),
            Ok(OscArgument::Float64(f64::NEG_INFINITY))
        );
        assert_eq!(index, 8);

        // MAX value
        index = 0;
        assert_eq!(
            OscArgument::from_bytes(&f64::MAX.to_be_bytes(), &mut index, 'd'),
            Ok(OscArgument::Float64(f64::MAX))
        );
        assert_eq!(index, 8);

        // MIN value (most negative)
        index = 0;
        assert_eq!(
            OscArgument::from_bytes(&f64::MIN.to_be_bytes(), &mut index, 'd'),
            Ok(OscArgument::Float64(f64::MIN))
        );
        assert_eq!(index, 8);

        // Test with non-zero index (simulating sequential parsing)
        let data = [
            0xFF, 0xFF, 0xFF, 0xFF,  // junk padding (4 bytes)
            0x40, 0x72, 0xC0, 0x00, 0x00, 0x00, 0x00, 0x00,  // 300.0 in big-endian (8 bytes)
            0xAA, 0xBB, 0xCC, 0xDD,  // more junk after (4 bytes)
        ];
        index = 4; // Start after the padding
        assert_eq!(
            OscArgument::from_bytes(&data, &mut index, 'd'),
            Ok(OscArgument::Float64(300.0))
        );
        assert_eq!(index, 12); // Should advance by 8 bytes (from 4 to 12)

        // Test with different starting index
        let data2 = [
            0xAA, 0xBB,              // 2 bytes padding
            0xC0, 0x5F, 0x40, 0x00, 0x00, 0x00, 0x00, 0x00,  // -125.0 in big-endian
            0xCC, 0xDD,              // 2 bytes after
        ];
        index = 2;
        assert_eq!(
            OscArgument::from_bytes(&data2, &mut index, 'd'),
            Ok(OscArgument::Float64(-125.0))
        );
        assert_eq!(index, 10); // 2 + 8 = 10
    }

    #[test]
    fn test_arguments_from_bytes_alternate_type() {
        // Simple case
        let mut index = 0;
        assert_eq!(
            OscArgument::from_bytes(&helpers::osc_string_as_bytes("alternate"), &mut index, 'S'),
            Ok(OscArgument::AlternateType("alternate".to_string()))
        );
        assert_eq!(index, 12); // "alternate" = 9 + null + 2 pad = 12

        // Empty string
        index = 0;
        assert_eq!(
            OscArgument::from_bytes(&[0x00, 0x00, 0x00, 0x00], &mut index, 'S'),
            Ok(OscArgument::AlternateType("".to_string()))
        );
        assert_eq!(index, 4);

        // 1 character (1 + null + 2 pad = 4)
        index = 0;
        assert_eq!(
            OscArgument::from_bytes(&[b'a', 0x00, 0x00, 0x00], &mut index, 'S'),
            Ok(OscArgument::AlternateType("a".to_string()))
        );
        assert_eq!(index, 4);

        // 2 characters (2 + null + 1 pad = 4)
        index = 0;
        assert_eq!(
            OscArgument::from_bytes(&[b'y', b'o', 0x00, 0x00], &mut index, 'S'),
            Ok(OscArgument::AlternateType("yo".to_string()))
        );
        assert_eq!(index, 4);

        // 3 characters (3 + null = 4, no padding needed)
        index = 0;
        assert_eq!(
            OscArgument::from_bytes(&[b'x', b'y', b'z', 0x00], &mut index, 'S'),
            Ok(OscArgument::AlternateType("xyz".to_string()))
        );
        assert_eq!(index, 4);

        // 4 characters (4 + null + 3 pad = 8)
        index = 0;
        assert_eq!(
            OscArgument::from_bytes(&[b'a', b'b', b'c', b'd', 0x00, 0x00, 0x00, 0x00], &mut index, 'S'),
            Ok(OscArgument::AlternateType("abcd".to_string()))
        );
        assert_eq!(index, 8);

        // Longer string
        index = 0;
        assert_eq!(
            OscArgument::from_bytes(&[b's', b'y', b'm', b'b', b'o', b'l', 0x00, 0x00], &mut index, 'S'),
            Ok(OscArgument::AlternateType("symbol".to_string()))
        );
        assert_eq!(index, 8); // 6 + null + 1 pad = 8

        // Test with non-zero index (simulating sequential parsing)
        let data = [
            0xFF, 0xFF, 0xFF, 0xFF,  // junk padding (4 bytes)
            b't', b'y', b'p', 0x00,  // "typ" + null = 4 bytes
            0xAA, 0xBB, 0xCC, 0xDD,  // more junk after (4 bytes)
        ];
        index = 4; // Start after the padding
        assert_eq!(
            OscArgument::from_bytes(&data, &mut index, 'S'),
            Ok(OscArgument::AlternateType("typ".to_string()))
        );
        assert_eq!(index, 8); // Should advance by 4 bytes (from 4 to 8)

        // Test with different starting index and longer string
        let data2 = [
            0xAA, 0xBB,                              // 2 bytes padding
            b'a', b't', b'o', b'm', 0x00, 0x00, 0x00, 0x00,  // "atom" (4 + null + 3 pad = 8)
            0xCC, 0xDD,                              // 2 bytes after
        ];
        index = 2;
        assert_eq!(
            OscArgument::from_bytes(&data2, &mut index, 'S'),
            Ok(OscArgument::AlternateType("atom".to_string()))
        );
        assert_eq!(index, 10); // 2 + 8 = 10
    }
    
    #[test]
    fn test_arguments_from_bytes_ascii() {
        // Printable ASCII characters
        let mut index = 0;
        assert_eq!(
            OscArgument::from_bytes(&[0x00, 0x00, 0x00, 0x41], &mut index, 'c'),
            Ok(OscArgument::AsciiCharacter(b'A'))
        );
        assert_eq!(index, 4);

        index = 0;
        assert_eq!(
            OscArgument::from_bytes(&[0x00, 0x00, 0x00, 0x42], &mut index, 'c'),
            Ok(OscArgument::AsciiCharacter(b'B'))
        );
        assert_eq!(index, 4);

        index = 0;
        assert_eq!(
            OscArgument::from_bytes(&[0x00, 0x00, 0x00, 0x78], &mut index, 'c'),
            Ok(OscArgument::AsciiCharacter(b'x'))
        );
        assert_eq!(index, 4);

        index = 0;
        assert_eq!(
            OscArgument::from_bytes(&[0x00, 0x00, 0x00, 0x7A], &mut index, 'c'),
            Ok(OscArgument::AsciiCharacter(b'z'))
        );
        assert_eq!(index, 4);

        // Space character
        index = 0;
        assert_eq!(
            OscArgument::from_bytes(&[0x00, 0x00, 0x00, 0x20], &mut index, 'c'),
            Ok(OscArgument::AsciiCharacter(b' '))
        );
        assert_eq!(index, 4);

        // Digit characters
        index = 0;
        assert_eq!(
            OscArgument::from_bytes(&[0x00, 0x00, 0x00, 0x30], &mut index, 'c'),
            Ok(OscArgument::AsciiCharacter(b'0'))
        );
        assert_eq!(index, 4);

        index = 0;
        assert_eq!(
            OscArgument::from_bytes(&[0x00, 0x00, 0x00, 0x39], &mut index, 'c'),
            Ok(OscArgument::AsciiCharacter(b'9'))
        );
        assert_eq!(index, 4);

        // Non-printable ASCII (null)
        index = 0;
        assert_eq!(
            OscArgument::from_bytes(&[0x00, 0x00, 0x00, 0x00], &mut index, 'c'),
            Ok(OscArgument::AsciiCharacter(0))
        );
        assert_eq!(index, 4);

        // Non-printable ASCII (DEL)
        index = 0;
        assert_eq!(
            OscArgument::from_bytes(&[0x00, 0x00, 0x00, 0x7F], &mut index, 'c'),
            Ok(OscArgument::AsciiCharacter(127))
        );
        assert_eq!(index, 4);

        // Extended ASCII (beyond standard ASCII range)
        index = 0;
        assert_eq!(
            OscArgument::from_bytes(&[0x00, 0x00, 0x00, 0xFF], &mut index, 'c'),
            Ok(OscArgument::AsciiCharacter(255))
        );
        assert_eq!(index, 4);

        // Test with non-zero index (simulating sequential parsing)
        let data = [
            0xFF, 0xFF, 0xFF, 0xFF,  // junk padding (4 bytes)
            0x00, 0x00, 0x00, 0x4D,  // 'M' character (4 bytes)
            0xAA, 0xBB, 0xCC, 0xDD,  // more junk after (4 bytes)
        ];
        index = 4; // Start after the padding
        assert_eq!(
            OscArgument::from_bytes(&data, &mut index, 'c'),
            Ok(OscArgument::AsciiCharacter(b'M'))
        );
        assert_eq!(index, 8); // Should advance by 4 bytes (from 4 to 8)

        // Test with different starting index
        let data2 = [
            0xAA, 0xBB,              // 2 bytes padding
            0x00, 0x00, 0x00, 0x21,  // '!' character
            0xCC, 0xDD,              // 2 bytes after
        ];
        index = 2;
        assert_eq!(
            OscArgument::from_bytes(&data2, &mut index, 'c'),
            Ok(OscArgument::AsciiCharacter(b'!'))
        );
        assert_eq!(index, 6); // 2 + 4 = 6
    }

    #[test]
    fn test_arguments_from_bytes_colour() {
        // Test all combinations of key values (0, 128, 255) for each channel
        for r in [0, 128, 255] {
            for g in [0, 128, 255] {
                for b in [0, 128, 255] {
                    for a in [0, 128, 255] {
                        let mut index = 0;
                        assert_eq!(
                            OscArgument::from_bytes(&[r, g, b, a], &mut index, 'r'),
                            Ok(OscArgument::Colour(OscColour { red: r, green: g, blue: b, alpha: a }))
                        );
                        assert_eq!(index, 4);
                    }
                }
            }
        }

        // Test with non-zero index (simulating sequential parsing)
        let data = [
            0xFF, 0xFF, 0xFF, 0xFF,  // junk padding (4 bytes)
            64, 128, 192, 255,       // color (4 bytes)
            0xAA, 0xBB, 0xCC, 0xDD,  // more junk after (4 bytes)
        ];
        let mut index = 4; // Start after the padding
        assert_eq!(
            OscArgument::from_bytes(&data, &mut index, 'r'),
            Ok(OscArgument::Colour(OscColour { red: 64, green: 128, blue: 192, alpha: 255 }))
        );
        assert_eq!(index, 8); // Should advance by 4 bytes (from 4 to 8)

        // Test with different starting index
        let data2 = [
            0xAA, 0xBB,              // 2 bytes padding
            255, 0, 128, 64,         // color (4 bytes)
            0xCC, 0xDD,              // 2 bytes after
        ];
        index = 2;
        assert_eq!(
            OscArgument::from_bytes(&data2, &mut index, 'r'),
            Ok(OscArgument::Colour(OscColour { red: 255, green: 0, blue: 128, alpha: 64 }))
        );
        assert_eq!(index, 6); // 2 + 4 = 6
    }

    #[test]
    fn test_arguments_from_bytes_midi() {
        // Test various combinations of MIDI message bytes
        for port in [0, 1, 15, 255] {
            for status in [0x80, 0x90, 0xB0, 0xC0, 0xE0, 0xF0] {
                for data1 in [0, 64, 127] {
                    for data2 in [0, 64, 127] {
                        let mut index = 0;
                        assert_eq!(
                            OscArgument::from_bytes(&[port, status, data1, data2], &mut index, 'm'),
                            Ok(OscArgument::MidiMessage(port, status, data1, data2))
                        );
                        assert_eq!(index, 4);
                    }
                }
            }
        }

        // Test with non-zero index (simulating sequential parsing)
        let data = [
            0xFF, 0xFF, 0xFF, 0xFF,  // junk padding (4 bytes)
            1, 0x90, 60, 100,        // Note On, port 1, middle C, velocity 100 (4 bytes)
            0xAA, 0xBB, 0xCC, 0xDD,  // more junk after (4 bytes)
        ];
        let mut index = 4; // Start after the padding
        assert_eq!(
            OscArgument::from_bytes(&data, &mut index, 'm'),
            Ok(OscArgument::MidiMessage(1, 0x90, 60, 100))
        );
        assert_eq!(index, 8); // Should advance by 4 bytes (from 4 to 8)

        // Test with different starting index
        let data2 = [
            0xAA, 0xBB,              // 2 bytes padding
            0, 0x80, 60, 0,          // Note Off, port 0, middle C (4 bytes)
            0xCC, 0xDD,              // 2 bytes after
        ];
        index = 2;
        assert_eq!(
            OscArgument::from_bytes(&data2, &mut index, 'm'),
            Ok(OscArgument::MidiMessage(0, 0x80, 60, 0))
        );
        assert_eq!(index, 6); // 2 + 4 = 6
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