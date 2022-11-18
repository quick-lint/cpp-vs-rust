use cpp_vs_rust::util::*;

#[test]
fn ascii() {
    let mut buffer: [u8; 1] = [0];
    let rest: &mut [u8] = encode_utf_8('x' as u32, &mut buffer);
    assert_eq!(unsafe { rest.as_ptr().offset_from(buffer.as_ptr()) }, 1);
    assert_eq!(buffer, ['x' as u8]);
}

#[test]
fn one_byte_output_extremes() {
    expect_encode_utf_8(0x0000, &[0x00]);
    expect_encode_utf_8(0x007f, &[0x7f]);
}

#[test]
fn two_byte_output() {
    expect_encode_utf_8(0x00a2, &[0xc2, 0xa2]);
}

#[test]
fn two_byte_output_extremes() {
    expect_encode_utf_8(0x0080, &[0xc2, 0x80]);
    expect_encode_utf_8(0x07ff, &[0xdf, 0xbf]);
}

#[test]
fn three_byte_output() {
    expect_encode_utf_8(0x0939, &[0xe0, 0xa4, 0xb9]);
    expect_encode_utf_8(0x20ac, &[0xe2, 0x82, 0xac]);
    expect_encode_utf_8(0xd55c, &[0xed, 0x95, 0x9c]);
}

#[test]
fn three_byte_output_extremes() {
    expect_encode_utf_8(0x0800, &[0xe0, 0xa0, 0x80]);
    expect_encode_utf_8(0xd7ff, &[0xed, 0x9f, 0xbf]);
    expect_encode_utf_8(0xe000, &[0xee, 0x80, 0x80]);
    expect_encode_utf_8(0xffff, &[0xef, 0xbf, 0xbf]);
}

#[test]
fn non_standard_surrogate_code_points() {
    expect_encode_utf_8(0xd800, &[0xed, 0xa0, 0x80]);
    expect_encode_utf_8(0xdfff, &[0xed, 0xbf, 0xbf]);
}

#[test]
fn four_byte_output() {
    expect_encode_utf_8(0x00010348, &[0xf0, 0x90, 0x8d, 0x88]);
}

#[test]
fn four_byte_output_extremes() {
    expect_encode_utf_8(0x00010000, &[0xf0, 0x90, 0x80, 0x80]);
    expect_encode_utf_8(0x0010ffff, &[0xf4, 0x8f, 0xbf, 0xbf]);
}

#[test]
fn non_standard_four_byte_output_extremes() {
    expect_encode_utf_8(0x001fffff, &[0xf7, 0xbf, 0xbf, 0xbf]);
}

fn expect_encode_utf_8(code_point: u32, expected: &[u8]) {
    let mut out: Vec<u8> = vec![0; expected.len()];
    let end: &mut [u8] = encode_utf_8(code_point, &mut out);
    assert_eq!(
        unsafe { end.as_ptr().offset_from(out.as_ptr()) },
        expected.len() as isize,
    );
    assert_eq!(out, expected);
}
