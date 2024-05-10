#![feature(unchecked_math)]
pub fn parse_number(bytes: &[u8; 8]) -> i64 {
    let number_i64 = i64::from_le_bytes(*bytes);
    let decimal_sep_pos = (!number_i64 & 0x10101000).trailing_zeros() as usize;
    convert_into_number(decimal_sep_pos, number_i64)
}

fn convert_into_number(decimal_sep_pos: usize, number_i64: i64) -> i64 {
    let shift = 28 - decimal_sep_pos;
    // signed is -1 if negative, 0 otherwise
    let signed = (!number_i64 << 59) >> 63;
    let design_mask = !(signed & 0xFF);
    // Align the number to a specific position and transform the ascii to digit value
    let digits: i64 = ((number_i64 & design_mask) << shift) & 0x0F000F0F00;
    // Now digits is in the form 0xUU00TTHH00 (UU: units digit, TT: tens digit, HH: hundreds digit)
    // 0xUU00TTHH00 * (100 * 0x1000000 + 10 * 0x10000 + 1) =
    // 0x000000UU00TTHH00 + 0x00UU00TTHH000000 * 10 + 0xUU00TTHH00000000 * 100
    let abs_value = ((unsafe { digits.unchecked_mul(0x640a0001) }) >> 32) & 0x3FF;
    (abs_value ^ signed) - signed
}

#[test]
fn zero() {
    assert_eq!(parse_number(b"0.0abcde"), 0)
}

#[test]
fn neg_one() {
    assert_eq!(parse_number(b"-1.0abcd"), -10)
}

#[test]
fn neg_zero() {
    assert_eq!(parse_number(b"-0.0abcd"), -0)
}

#[test]
fn high() {
    assert_eq!(parse_number(b"54.3abcd"), 543)
}

#[test]
fn low() {
    assert_eq!(parse_number(b"-99.9abc"), -999)
}
