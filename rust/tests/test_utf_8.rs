use cpp_vs_rust::container::padded_string::*;
use cpp_vs_rust::util::utf_8::*;

#[test]
fn encode_ascii() {
    let mut buffer: [u8; 1] = [0];
    let rest: &mut [u8] = encode_utf_8('x' as u32, &mut buffer);
    assert_eq!(unsafe { rest.as_ptr().offset_from(buffer.as_ptr()) }, 1);
    assert_eq!(buffer, [b'x']);
}

#[test]
fn encode_one_byte_output_extremes() {
    expect_encode_utf_8(0x0000, &[0x00]);
    expect_encode_utf_8(0x007f, &[0x7f]);
}

#[test]
fn encode_two_byte_output() {
    expect_encode_utf_8(0x00a2, &[0xc2, 0xa2]);
}

#[test]
fn encode_two_byte_output_extremes() {
    expect_encode_utf_8(0x0080, &[0xc2, 0x80]);
    expect_encode_utf_8(0x07ff, &[0xdf, 0xbf]);
}

#[test]
fn encode_three_byte_output() {
    expect_encode_utf_8(0x0939, &[0xe0, 0xa4, 0xb9]);
    expect_encode_utf_8(0x20ac, &[0xe2, 0x82, 0xac]);
    expect_encode_utf_8(0xd55c, &[0xed, 0x95, 0x9c]);
}

#[test]
fn encode_three_byte_output_extremes() {
    expect_encode_utf_8(0x0800, &[0xe0, 0xa0, 0x80]);
    expect_encode_utf_8(0xd7ff, &[0xed, 0x9f, 0xbf]);
    expect_encode_utf_8(0xe000, &[0xee, 0x80, 0x80]);
    expect_encode_utf_8(0xffff, &[0xef, 0xbf, 0xbf]);
}

#[test]
fn encode_non_standard_surrogate_code_points() {
    expect_encode_utf_8(0xd800, &[0xed, 0xa0, 0x80]);
    expect_encode_utf_8(0xdfff, &[0xed, 0xbf, 0xbf]);
}

#[test]
fn encode_four_byte_output() {
    expect_encode_utf_8(0x00010348, &[0xf0, 0x90, 0x8d, 0x88]);
}

#[test]
fn encode_four_byte_output_extremes() {
    expect_encode_utf_8(0x00010000, &[0xf0, 0x90, 0x80, 0x80]);
    expect_encode_utf_8(0x0010ffff, &[0xf4, 0x8f, 0xbf, 0xbf]);
}

#[test]
fn encode_non_standard_four_byte_output_extremes() {
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

#[test]
fn decode_empty_string() {
    let result: DecodeUTF8Result = decode_utf_8(PaddedString::from_str("").view());
    assert_eq!(result.size, 0);
    assert!(!result.ok);
}

#[test]
fn decode_ascii() {
    {
        let result: DecodeUTF8Result = decode_utf_8(PaddedString::from_str("a").view());
        assert_eq!(result.size, 1);
        assert!(result.ok);
        assert_eq!(result.code_point, 'a' as u32);
    }

    {
        let result: DecodeUTF8Result = decode_utf_8(PaddedString::from_str("12345").view());
        assert_eq!(result.size, 1);
        assert!(result.ok);
        assert_eq!(result.code_point, '1' as u32);
    }
}

#[test]
fn decode_leading_continuation_code_unit_is_an_error() {
    {
        let result: DecodeUTF8Result = decode_utf_8(PaddedString::from_slice(&[0xa2]).view());
        assert_eq!(result.size, 1);
        assert!(!result.ok);
    }

    {
        let result: DecodeUTF8Result =
            decode_utf_8(PaddedString::from_slice(&[0xa2, 0xa2, 0xa2]).view());
        assert_eq!(result.size, 1);
        assert!(!result.ok);
    }
}

#[test]
fn decode_always_invalid_code_unit_is_an_error() {
    for code_unit in [
        0xc0u8, 0xc1, 0xf5, 0xf6, 0xf7, 0xf8, 0xf9, 0xfa, 0xfb, 0xfc, 0xfd, 0xfe, 0xff,
    ] {
        {
            let input = PaddedString::from_slice(&[code_unit]);
            let result: DecodeUTF8Result = decode_utf_8(input.view());
            assert_eq!(result.size, 1);
            assert!(!result.ok);
        }

        {
            let input = PaddedString::from_slice(&[code_unit, b'?', b'?', b'?', b'?', b'?']);
            let result: DecodeUTF8Result = decode_utf_8(input.view());
            assert_eq!(result.size, 1);
            assert!(!result.ok);
        }

        {
            let input = PaddedString::from_slice(&[code_unit, 0xa2, 0xa2, 0xa2, 0xa2, 0xa2]);
            let result: DecodeUTF8Result = decode_utf_8(input.view());
            assert_eq!(result.size, 1);
            assert!(!result.ok);
        }
    }
}

#[test]
fn decode_two_byte_character() {
    expect_decode_utf_8_single_code_point(&[0xc2, 0xa2], 0x00a2);
}

#[test]
fn decode_truncated_two_byte_character() {
    {
        let result: DecodeUTF8Result = decode_utf_8(PaddedString::from_slice(&[0xc2]).view());
        assert_eq!(result.size, 1);
        assert!(!result.ok);
    }

    {
        let result: DecodeUTF8Result = decode_utf_8(PaddedString::from_slice(&[0xc2, b'?']).view());
        assert_eq!(result.size, 1);
        assert!(!result.ok);
    }

    {
        let result: DecodeUTF8Result = decode_utf_8(PaddedString::from_slice(&[0xc2, 0xc2]).view());
        assert_eq!(result.size, 1);
        assert!(!result.ok);
    }
}

#[test]
fn decode_two_byte_character_with_trailing_continuation_bytes() {
    {
        let result: DecodeUTF8Result =
            decode_utf_8(PaddedString::from_slice(&[0xc2, 0xa2, 0xa2]).view());
        assert_eq!(result.size, 2);
        assert!(result.ok);
        assert_eq!(result.code_point, 0x00a2);
    }
}

#[test]
fn decode_three_byte_character() {
    expect_decode_utf_8_single_code_point(&[0xe0, 0xa4, 0xb9], 0x0939);
    expect_decode_utf_8_single_code_point(&[0xe2, 0x82, 0xac], 0x20ac);
    expect_decode_utf_8_single_code_point(&[0xed, 0x95, 0x9c], 0xd55c);
}

#[test]
fn decode_truncated_three_byte_character() {
    {
        let result: DecodeUTF8Result = decode_utf_8(PaddedString::from_slice(&[0xe0, 0xa4]).view());
        assert_eq!(result.size, 2);
        assert!(!result.ok);
    }

    {
        let result: DecodeUTF8Result =
            decode_utf_8(PaddedString::from_slice(&[0xe0, 0xa4, b'?', b'?', b'?']).view());
        assert_eq!(result.size, 2);
        assert!(!result.ok);
    }

    {
        let result: DecodeUTF8Result = decode_utf_8(PaddedString::from_slice(&[0xe0]).view());
        assert_eq!(result.size, 1);
        assert!(!result.ok);
    }

    {
        let result: DecodeUTF8Result = decode_utf_8(PaddedString::from_slice(&[0xe0, b'?']).view());
        assert_eq!(result.size, 1);
        assert!(!result.ok);
    }

    {
        let result: DecodeUTF8Result =
            decode_utf_8(PaddedString::from_slice(&[0xe0, b'?', b'?', b'?', b'?']).view());
        assert_eq!(result.size, 1);
        assert!(!result.ok);
    }
}

#[test]
fn decode_four_byte_character() {
    expect_decode_utf_8_single_code_point(&[0xf0, 0x90, 0x8d, 0x88], 0x00010348);
}

#[test]
fn decode_truncated_four_byte_character() {
    {
        let result: DecodeUTF8Result =
            decode_utf_8(PaddedString::from_slice(&[0xf0, 0x90, 0x8d]).view());
        assert_eq!(result.size, 3);
        assert!(!result.ok);
    }

    {
        let result: DecodeUTF8Result = decode_utf_8(
            PaddedString::from_slice(&[0xf0, 0x90, 0x8d, b'?', b'?', b'?', b'?', b'?']).view(),
        );
        assert_eq!(result.size, 3);
        assert!(!result.ok);
    }

    {
        let result: DecodeUTF8Result = decode_utf_8(PaddedString::from_slice(&[0xf0, 0x90]).view());
        assert_eq!(result.size, 2);
        assert!(!result.ok);
    }

    {
        let result: DecodeUTF8Result =
            decode_utf_8(PaddedString::from_slice(&[0xf0, 0x90, b'?']).view());
        assert_eq!(result.size, 2);
        assert!(!result.ok);
    }

    {
        let result: DecodeUTF8Result = decode_utf_8(
            PaddedString::from_slice(&[0xf0, 0x90, b'?', b'?', b'?', b'?', b'?', b'?']).view(),
        );
        assert_eq!(result.size, 2);
        assert!(!result.ok);
    }

    {
        let result: DecodeUTF8Result = decode_utf_8(PaddedString::from_slice(&[0xf0]).view());
        assert_eq!(result.size, 1);
        assert!(!result.ok);
    }

    {
        let result: DecodeUTF8Result = decode_utf_8(PaddedString::from_slice(&[0xf0, b'?']).view());
        assert_eq!(result.size, 1);
        assert!(!result.ok);
    }

    {
        let result: DecodeUTF8Result =
            decode_utf_8(PaddedString::from_slice(&[0xf0, b'?', b'?']).view());
        assert_eq!(result.size, 1);
        assert!(!result.ok);
    }

    {
        let result: DecodeUTF8Result = decode_utf_8(
            PaddedString::from_slice(&[0xf0, b'?', b'?', b'?', b'?', b'?', b'?', b'?', b'?'])
                .view(),
        );
        assert_eq!(result.size, 1);
        assert!(!result.ok);
    }
}

#[test]
fn decode_overlong_sequences_are_an_error_for_each_code_unit() {
    for input in [
        PaddedString::from_slice(&[0xc0, 0x80]),             // U+0000
        PaddedString::from_slice(&[0xe0, 0x80, 0x80]),       // U+0000
        PaddedString::from_slice(&[0xf0, 0x80, 0x80, 0x80]), // U+0000
        PaddedString::from_slice(&[0xf8, 0x80, 0x80, 0x80, 0x80]), // U+0000
        PaddedString::from_slice(&[0xfc, 0x80, 0x80, 0x80, 0x80, 0x80]), // U+0000
        PaddedString::from_slice(&[0xc0, 0xaf]),             // U+002F
        PaddedString::from_slice(&[0xe0, 0x80, 0xaf]),       // U+002F
        PaddedString::from_slice(&[0xf0, 0x80, 0x80, 0xaf]), // U+002F
        PaddedString::from_slice(&[0xf8, 0x80, 0x80, 0x80, 0xaf]), // U+002F
        PaddedString::from_slice(&[0xfc, 0x80, 0x80, 0x80, 0x80, 0xaf]), // U+002F
        PaddedString::from_slice(&[0xc1, 0xbf]),             // U+007F
        PaddedString::from_slice(&[0xe0, 0x9f, 0xbf]),       // U+07FF
        PaddedString::from_slice(&[0xf0, 0x8f, 0xbf, 0xbf]), // U+FFFF
        PaddedString::from_slice(&[0xf8, 0x87, 0xbf, 0xbf, 0xbf]), // U+001FFFFF
        PaddedString::from_slice(&[0xfc, 0x83, 0xbf, 0xbf, 0xbf, 0xbf]), // U+03FFFFFF
    ] {
        let mut i = 0;
        while i < input.size() {
            let current_input = PaddedStringView::from(&input).substr(i);
            let result: DecodeUTF8Result = decode_utf_8(current_input);
            assert_eq!(result.size, 1);
            assert!(!result.ok);
            assert!(result.size >= 1);
            i += result.size;
        }
    }
}

#[test]
fn decode_surrogate_sequences_are_an_error_for_each_code_unit() {
    for input in [
        PaddedString::from_slice(&[0xed, 0xa0, 0x80]), // U+D800
        PaddedString::from_slice(&[0xed, 0xad, 0xbf]), // U+DB7F
        PaddedString::from_slice(&[0xed, 0xae, 0x80]), // U+DB80
        PaddedString::from_slice(&[0xed, 0xaf, 0xbf]), // U+DBFF
        PaddedString::from_slice(&[0xed, 0xb0, 0x80]), // U+DC00
        PaddedString::from_slice(&[0xed, 0xbe, 0x80]), // U+DF80
        PaddedString::from_slice(&[0xed, 0xbf, 0xbf]), // U+DFFF
    ] {
        let mut i = 0;
        while i < input.size() {
            let current_input = PaddedStringView::from(&input).substr(i);
            let result: DecodeUTF8Result = decode_utf_8(current_input);
            assert_eq!(result.size, 1);
            assert!(!result.ok);
            assert!(result.size >= 1);
            i += result.size;
        }
    }
}

fn expect_decode_utf_8_single_code_point(input: &[u8], expected: u32) {
    let input_string = PaddedString::from_slice(input);
    let result: DecodeUTF8Result = decode_utf_8(input_string.view());
    assert_eq!(result.size, input_string.size());
    assert!(result.ok);
    assert_eq!(result.code_point, expected);
}
