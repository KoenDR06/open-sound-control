
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