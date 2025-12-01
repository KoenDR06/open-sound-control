
/// Pads a Vec<u8> to the next multiple of 4 bytes by appending zeros
pub fn pad_to_multiple_of_4_bytes(bytes: &mut Vec<u8>) {
    let pad_length = (4 - (bytes.len() % 4)) % 4;
    if pad_length > 0 {
        bytes.extend(std::iter::repeat(0).take(pad_length));
    }
}