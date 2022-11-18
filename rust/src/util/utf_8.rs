// See: https://www.unicode.org/versions/Unicode11.0.0/ch03.pdf
pub fn encode_utf_8<'a>(code_point: u32, out: &'a mut [u8]) -> &'a mut [u8] {
    let byte = |x: u32| {
        assert!(x <= 0x100);
        return x as u8;
    };
    let continuation = 0b1000_0000;
    if code_point >= 0x10000 {
        out[0] = byte(0b1111_0000 | (code_point >> 18));
        out[1] = byte(continuation | ((code_point >> 12) & 0b0011_1111));
        out[2] = byte(continuation | ((code_point >> 6) & 0b0011_1111));
        out[3] = byte(continuation | ((code_point >> 0) & 0b0011_1111));
        &mut out[4..]
    } else if code_point >= 0x0800 {
        out[0] = byte(0b1110_0000 | (code_point >> 12));
        out[1] = byte(continuation | ((code_point >> 6) & 0b0011_1111));
        out[2] = byte(continuation | ((code_point >> 0) & 0b0011_1111));
        &mut out[3..]
    } else if code_point >= 0x80 {
        out[0] = byte(0b1100_0000 | (code_point >> 6));
        out[1] = byte(continuation | ((code_point >> 0) & 0b0011_1111));
        &mut out[2..]
    } else {
        out[0] = byte(code_point);
        &mut out[1..]
    }
}
