use crate::scoped_trace;
use crate::util::padded_string::*;
use crate::util::utf_8::*;

#[test]
fn encode_ascii() {
    let mut buffer: [u8; 1] = [0];
    let len: usize = encode_utf_8('x' as u32, &mut buffer);
    assert_eq!(len, 1);
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
    let len: usize = encode_utf_8(code_point, &mut out);
    assert_eq!(len, expected.len());
    assert_eq!(out, expected);
}

#[test]
fn decode_empty_string() {
    let result: DecodeUTF8Result = decode_utf_8(PaddedString::from_slice(b"").view());
    assert_eq!(result.size, 0);
    assert!(!result.ok);
}

#[test]
fn decode_ascii() {
    {
        let result: DecodeUTF8Result = decode_utf_8(PaddedString::from_slice(b"a").view());
        assert_eq!(result.size, 1);
        assert!(result.ok);
        assert_eq!(result.code_point, 'a');
    }

    {
        let result: DecodeUTF8Result = decode_utf_8(PaddedString::from_slice(b"12345").view());
        assert_eq!(result.size, 1);
        assert!(result.ok);
        assert_eq!(result.code_point, '1');
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
        scoped_trace!(code_unit);

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
    expect_decode_utf_8_single_code_point(&[0xc2, 0xa2], '\u{00a2}');
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
        assert_eq!(result.code_point, '\u{00a2}');
    }
}

#[test]
fn decode_three_byte_character() {
    expect_decode_utf_8_single_code_point(&[0xe0, 0xa4, 0xb9], '\u{0939}');
    expect_decode_utf_8_single_code_point(&[0xe2, 0x82, 0xac], '\u{20ac}');
    expect_decode_utf_8_single_code_point(&[0xed, 0x95, 0x9c], '\u{d55c}');
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
    expect_decode_utf_8_single_code_point(&[0xf0, 0x90, 0x8d, 0x88], '\u{010348}');
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
        scoped_trace!(input);

        let mut i = 0;
        while i < input.len() {
            let current_input = PaddedStringView::from(&input).substr(i);
            scoped_trace!(current_input);
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
        scoped_trace!(input);
        let mut i = 0;
        while i < input.len() {
            let current_input = PaddedStringView::from(&input).substr(i);
            scoped_trace!(current_input);
            let result: DecodeUTF8Result = decode_utf_8(current_input);
            assert_eq!(result.size, 1);
            assert!(!result.ok);
            assert!(result.size >= 1);
            i += result.size;
        }
    }
}

fn expect_decode_utf_8_single_code_point(input: &[u8], expected: char) {
    let input_string = PaddedString::from_slice(input);
    let result: DecodeUTF8Result = decode_utf_8(input_string.view());
    assert_eq!(result.size, input_string.len());
    assert!(result.ok);
    assert_eq!(result.code_point, expected);
}

fn count_lsp_characters_in_utf_8_full(utf_8: PaddedStringView) -> isize {
    return count_lsp_characters_in_utf_8(utf_8, utf_8.len());
}

#[test]
fn count_lsp_characters_empty_string() {
    assert_eq!(
        count_lsp_characters_in_utf_8_full(PaddedString::from_slice(b"").view()),
        0
    );
}

#[test]
fn count_lsp_characters_ascii_characters_count_as_one() {
    assert_eq!(
        count_lsp_characters_in_utf_8_full(PaddedString::from_slice(b"abcdef").view()),
        6
    );
}

#[test]
fn count_lsp_characters_non_ascii_basic_multilingual_plane_characters_count_as_one() {
    assert_eq!(
        count_lsp_characters_in_utf_8_full(PaddedString::from_slice("\u{2306}".as_bytes()).view()),
        1
    );
}

#[test]
fn count_lsp_characters_supplementary_plane_characters_count_as_two() {
    assert_eq!(
        count_lsp_characters_in_utf_8_full(
            PaddedString::from_slice("\u{01F430}".as_bytes()).view()
        ),
        2
    );
}

#[test]
fn count_lsp_characters_middle_of_single_character_is_not_counted() {
    // U+0100 has two UTF-8 code units.
    assert_eq!(
        count_lsp_characters_in_utf_8(PaddedString::from_slice("\u{0100}".as_bytes()).view(), 1),
        0
    );
    assert_eq!(
        count_lsp_characters_in_utf_8(PaddedString::from_slice("x\u{0100}y".as_bytes()).view(), 2),
        1
    );

    // U+2306 has three UTF-8 code units.
    assert_eq!(
        count_lsp_characters_in_utf_8(PaddedString::from_slice("\u{2306}".as_bytes()).view(), 1),
        0
    );
    assert_eq!(
        count_lsp_characters_in_utf_8(PaddedString::from_slice("\u{2306}".as_bytes()).view(), 2),
        0
    );

    // U+1F430 has four UTF-8 code units.
    assert_eq!(
        count_lsp_characters_in_utf_8(PaddedString::from_slice("\u{01F430}".as_bytes()).view(), 1),
        0
    );
    assert_eq!(
        count_lsp_characters_in_utf_8(PaddedString::from_slice("\u{01F430}".as_bytes()).view(), 2),
        0
    );
    assert_eq!(
        count_lsp_characters_in_utf_8(PaddedString::from_slice("\u{01F430}".as_bytes()).view(), 3),
        0
    );
}

#[test]
fn count_lsp_characters_invalid_surrogate_sequences_count_as_one_per_byte() {
    for input in [
        b"\xed\xa0\x80", // U+D800
        b"\xed\xad\xbf", // U+DB7F
        b"\xed\xae\x80", // U+DB80
        b"\xed\xaf\xbf", // U+DBFF
        b"\xed\xb0\x80", // U+DC00
        b"\xed\xbe\x80", // U+DF80
        b"\xed\xbf\xbf", // U+DFFF
    ] {
        let input = PaddedString::from_slice(input);
        scoped_trace!(input);
        assert_eq!(
            count_lsp_characters_in_utf_8(input.view(), input.len()),
            input.len() as isize
        );
    }
}

#[test]
fn count_lsp_characters_overlong_sequences_count_as_one_per_byte() {
    for input in [
        b"\xc0\x80" as &[u8],        // U+0000
        b"\xe0\x80\x80",             // U+0000
        b"\xf0\x80\x80\x80",         // U+0000
        b"\xf8\x80\x80\x80\x80",     // U+0000
        b"\xfc\x80\x80\x80\x80\x80", // U+0000
        b"\xc0\xaf",                 // U+002F
        b"\xe0\x80\xaf",             // U+002F
        b"\xf0\x80\x80\xaf",         // U+002F
        b"\xf8\x80\x80\x80\xaf",     // U+002F
        b"\xfc\x80\x80\x80\x80\xaf", // U+002F
        b"\xc1\xbf",                 // U+007F
        b"\xe0\x9f\xbf",             // U+07FF
        b"\xf0\x8f\xbf\xbf",         // U+FFFF
        b"\xf8\x87\xbf\xbf\xbf",     // U+001FFFFF
        b"\xfc\x83\xbf\xbf\xbf\xbf", // U+03FFFFFF
    ] {
        let input = PaddedString::from_slice(input);
        scoped_trace!(input);
        assert_eq!(
            count_lsp_characters_in_utf_8(input.view(), input.len()),
            input.len() as isize
        );
    }
}

#[test]
fn count_lsp_characters_incomplete_sequences_count_as_one_per_byte() {
    assert_eq!(
        count_lsp_characters_in_utf_8_full(PaddedString::from_slice(b"\xf0\x90\x8d").view()),
        3
    );
    assert_eq!(
        count_lsp_characters_in_utf_8_full(PaddedString::from_slice(b"\xf0\x90\x8d?????").view()),
        8
    );
    assert_eq!(
        count_lsp_characters_in_utf_8_full(PaddedString::from_slice(b"\xf0\x90").view()),
        2
    );
    assert_eq!(
        count_lsp_characters_in_utf_8_full(PaddedString::from_slice(b"\xf0\x90?").view()),
        3
    );
    assert_eq!(
        count_lsp_characters_in_utf_8_full(PaddedString::from_slice(b"\xf0\x90??????").view()),
        8
    );
    assert_eq!(
        count_lsp_characters_in_utf_8_full(PaddedString::from_slice(b"\xf0").view()),
        1
    );
    assert_eq!(
        count_lsp_characters_in_utf_8_full(PaddedString::from_slice(b"\xf0?").view()),
        2
    );
    assert_eq!(
        count_lsp_characters_in_utf_8_full(PaddedString::from_slice(b"\xf0??").view()),
        3
    );
    assert_eq!(
        count_lsp_characters_in_utf_8_full(PaddedString::from_slice(b"\xf0????????").view()),
        9
    );

    assert_eq!(
        count_lsp_characters_in_utf_8(PaddedString::from_slice(b"\xf0\x90").view(), 1),
        1
    );
    assert_eq!(
        count_lsp_characters_in_utf_8(PaddedString::from_slice(b"\xf0\x90??").view(), 1),
        1
    );
}

fn count_utf_8_characters_full(utf_8: PaddedStringView) -> usize {
    return count_utf_8_characters(utf_8, utf_8.len() as usize);
}

#[test]
fn count_utf_8_characters_empty_string() {
    let n: usize = count_utf_8_characters_full(PaddedString::from_slice(b"").view());
    assert_eq!(n, 0);
}

#[test]
fn count_utf_8_characters_ascii() {
    let n: usize = count_utf_8_characters_full(PaddedString::from_slice(b"foobar").view());
    assert_eq!(n, 6);
}

#[test]
fn count_utf_8_characters_ascii_num() {
    let n: usize = count_utf_8_characters_full(PaddedString::from_slice(b"1,2,3,4").view());
    assert_eq!(n, 7);
}

#[test]
fn count_utf_8_characters_multi_byte() {
    let n: usize = count_utf_8_characters_full(
        PaddedString::from_slice("\u{263a}\u{263b}\u{2639}".as_bytes()).view(),
    );
    assert_eq!(n, 3);
}

#[test]
fn count_utf_8_characters_multi_byte_offset() {
    let n: usize = count_utf_8_characters(
        PaddedString::from_slice("\u{263a}\u{263b}\u{2639}".as_bytes()).view(),
        6,
    );
    assert_eq!(n, 2);
}

#[test]
fn count_utf_8_characters_invalid_counts_as_one() {
    let n: usize = count_utf_8_characters_full(PaddedString::from_slice(b"\xe2\x80").view());
    assert_eq!(n, 2);
}

#[test]
fn count_utf_8_characters_invalid_conuts_as_one_with_null() {
    let n: usize = count_utf_8_characters_full(PaddedString::from_slice(b"\xe2\x00").view());
    assert_eq!(n, 2);
}

#[test]
fn count_utf_8_characters_mixed_ascii_with_invalid() {
    let n: usize = count_utf_8_characters_full(PaddedString::from_slice(b"a\xe2\x80").view());
    assert_eq!(n, 3);
}
