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

/* @@@
namespace {
decode_utf_8_result decode_utf_8(const padded_string& code_units) noexcept {
  return quick_lint_js::decode_utf_8(&code_units);
}

padded_string operator""_padded(const char8_t* chars,
                                std::size_t size) noexcept {
  return padded_string(string8_view(chars, size));
}

QLJS_WARNING_PUSH
QLJS_WARNING_IGNORE_GCC("-Wuseless-cast")
padded_string operator""_padded(const char* chars, std::size_t size) noexcept {
  return padded_string(
      string8_view(reinterpret_cast<const char8*>(chars), size));
}
QLJS_WARNING_POP
}

#[test]
fn empty_string() {
  decode_utf_8_result result = decode_utf_8(u8""_padded);
  assert_eq!(result.size, 0);
  EXPECT_FALSE(result.ok);
}

#[test]
fn ascii() {
  {
    decode_utf_8_result result = decode_utf_8(u8"a"_padded);
    assert_eq!(result.size, 1);
    EXPECT_TRUE(result.ok);
    assert_eq!(result.code_point, U'a');
  }

  {
    decode_utf_8_result result = decode_utf_8(u8"12345"_padded);
    assert_eq!(result.size, 1);
    EXPECT_TRUE(result.ok);
    assert_eq!(result.code_point, U'1');
  }
}

#[test]
fn leading_continuation_code_unit_is_an_error() {
  {
    decode_utf_8_result result = decode_utf_8("\xa2"_padded);
    assert_eq!(result.size, 1);
    EXPECT_FALSE(result.ok);
  }

  {
    decode_utf_8_result result = decode_utf_8("\xa2\xa2\xa2"_padded);
    assert_eq!(result.size, 1);
    EXPECT_FALSE(result.ok);
  }
}

QLJS_WARNING_PUSH
QLJS_WARNING_IGNORE_CLANG("-Wimplicit-int-conversion")
QLJS_WARNING_IGNORE_GCC("-Wconversion")
#[test]
fn always_invalid_code_unit_is_an_error() {
  for (char8 code_unit : {
           0xc0,
           0xc1,
           0xf5,
           0xf6,
           0xf7,
           0xf8,
           0xf9,
           0xfa,
           0xfb,
           0xfc,
           0xfd,
           0xfe,
           0xff,
       }) {
    SCOPED_TRACE(static_cast<int>(code_unit));

    {
      padded_string input(string8() + code_unit);
      decode_utf_8_result result = decode_utf_8(&input);
      assert_eq!(result.size, 1);
      EXPECT_FALSE(result.ok);
    }

    {
      padded_string input(code_unit + string8(u8"?????"));
      decode_utf_8_result result = decode_utf_8(&input);
      assert_eq!(result.size, 1);
      EXPECT_FALSE(result.ok);
    }

    {
      padded_string input(code_unit + string8("\xa2\xa2\xa2\xa2"_s8v));
      decode_utf_8_result result = decode_utf_8(&input);
      assert_eq!(result.size, 1);
      EXPECT_FALSE(result.ok);
    }
  }
}
QLJS_WARNING_POP

#[test]
fn two_byte_character() {
  EXPECT_DECODE_UTF_8_SINGLE_CODE_POINT("\xc2\xa2"_padded, U'\u00a2');
}

#[test]
fn truncated_two_byte_character() {
  {
    decode_utf_8_result result = decode_utf_8("\xc2"_padded);
    assert_eq!(result.size, 1);
    EXPECT_FALSE(result.ok);
  }

  {
    decode_utf_8_result result = decode_utf_8("\xc2?"_padded);
    assert_eq!(result.size, 1);
    EXPECT_FALSE(result.ok);
  }

  {
    decode_utf_8_result result = decode_utf_8("\xc2\xc2"_padded);
    assert_eq!(result.size, 1);
    EXPECT_FALSE(result.ok);
  }
}

#[test]
fn two_byte_character_with_trailing_continuation_bytes() {
  {
    decode_utf_8_result result = decode_utf_8("\xc2\xa2\xa2"_padded);
    assert_eq!(result.size, 2);
    EXPECT_TRUE(result.ok);
    assert_eq!(result.code_point, U'\u00a2');
  }
}

#[test]
fn three_byte_character() {
  EXPECT_DECODE_UTF_8_SINGLE_CODE_POINT("\xe0\xa4\xb9"_padded, U'\u0939');
  EXPECT_DECODE_UTF_8_SINGLE_CODE_POINT("\xe2\x82\xac"_padded, U'\u20ac');
  EXPECT_DECODE_UTF_8_SINGLE_CODE_POINT("\xed\x95\x9c"_padded, U'\ud55c');
}

#[test]
fn truncated_three_byte_character() {
  {
    decode_utf_8_result result = decode_utf_8("\xe0\xa4"_padded);
    assert_eq!(result.size, 2);
    EXPECT_FALSE(result.ok);
  }

  {
    decode_utf_8_result result = decode_utf_8("\xe0\xa4???"_padded);
    assert_eq!(result.size, 2);
    EXPECT_FALSE(result.ok);
  }

  {
    decode_utf_8_result result = decode_utf_8("\xe0"_padded);
    assert_eq!(result.size, 1);
    EXPECT_FALSE(result.ok);
  }

  {
    decode_utf_8_result result = decode_utf_8("\xe0?"_padded);
    assert_eq!(result.size, 1);
    EXPECT_FALSE(result.ok);
  }

  {
    decode_utf_8_result result = decode_utf_8("\xe0????"_padded);
    assert_eq!(result.size, 1);
    EXPECT_FALSE(result.ok);
  }
}

#[test]
fn four_byte_character() {
  EXPECT_DECODE_UTF_8_SINGLE_CODE_POINT("\xf0\x90\x8d\x88"_padded,
                                        U'\U00010348');
}

#[test]
fn truncated_four_byte_character() {
  {
    decode_utf_8_result result = decode_utf_8("\xf0\x90\x8d"_padded);
    assert_eq!(result.size, 3);
    EXPECT_FALSE(result.ok);
  }

  {
    decode_utf_8_result result = decode_utf_8("\xf0\x90\x8d?????"_padded);
    assert_eq!(result.size, 3);
    EXPECT_FALSE(result.ok);
  }

  {
    decode_utf_8_result result = decode_utf_8("\xf0\x90"_padded);
    assert_eq!(result.size, 2);
    EXPECT_FALSE(result.ok);
  }

  {
    decode_utf_8_result result = decode_utf_8("\xf0\x90?"_padded);
    assert_eq!(result.size, 2);
    EXPECT_FALSE(result.ok);
  }

  {
    decode_utf_8_result result = decode_utf_8("\xf0\x90??????"_padded);
    assert_eq!(result.size, 2);
    EXPECT_FALSE(result.ok);
  }

  {
    decode_utf_8_result result = decode_utf_8("\xf0"_padded);
    assert_eq!(result.size, 1);
    EXPECT_FALSE(result.ok);
  }

  {
    decode_utf_8_result result = decode_utf_8("\xf0?"_padded);
    assert_eq!(result.size, 1);
    EXPECT_FALSE(result.ok);
  }

  {
    decode_utf_8_result result = decode_utf_8("\xf0??"_padded);
    assert_eq!(result.size, 1);
    EXPECT_FALSE(result.ok);
  }

  {
    decode_utf_8_result result = decode_utf_8("\xf0????????"_padded);
    assert_eq!(result.size, 1);
    EXPECT_FALSE(result.ok);
  }
}

#[test]
fn overlong_sequences_are_an_error_for_each_code_unit() {
  for (const padded_string& input : {
           "\xc0\x80"_padded,                  // U+0000
           "\xe0\x80\x80"_padded,              // U+0000
           "\xf0\x80\x80\x80"_padded,          // U+0000
           "\xf8\x80\x80\x80\x80"_padded,      // U+0000
           "\xfc\x80\x80\x80\x80\x80"_padded,  // U+0000

           "\xc0\xaf"_padded,                  // U+002F
           "\xe0\x80\xaf"_padded,              // U+002F
           "\xf0\x80\x80\xaf"_padded,          // U+002F
           "\xf8\x80\x80\x80\xaf"_padded,      // U+002F
           "\xfc\x80\x80\x80\x80\xaf"_padded,  // U+002F

           "\xc1\xbf"_padded,                  // U+007F
           "\xe0\x9f\xbf"_padded,              // U+07FF
           "\xf0\x8f\xbf\xbf"_padded,          // U+FFFF
           "\xf8\x87\xbf\xbf\xbf"_padded,      // U+001FFFFF
           "\xfc\x83\xbf\xbf\xbf\xbf"_padded,  // U+03FFFFFF
       }) {
    SCOPED_TRACE(input);

    const char8* begin = input.data();
    while (begin != input.null_terminator()) {
      padded_string_view current_input(begin, input.null_terminator());
      SCOPED_TRACE(current_input);
      decode_utf_8_result result = decode_utf_8(current_input);
      assert_eq!(result.size, 1);
      EXPECT_FALSE(result.ok);
      ASSERT_GE(result.size, 1);
      begin += result.size;
    }
  }
}

#[test]
fn surrogate_sequences_are_an_error_for_each_code_unit() {
  for (const padded_string& input : {
           "\xed\xa0\x80"_padded,  // U+D800
           "\xed\xad\xbf"_padded,  // U+DB7F
           "\xed\xae\x80"_padded,  // U+DB80
           "\xed\xaf\xbf"_padded,  // U+DBFF
           "\xed\xb0\x80"_padded,  // U+DC00
           "\xed\xbe\x80"_padded,  // U+DF80
           "\xed\xbf\xbf"_padded,  // U+DFFF
       }) {
    SCOPED_TRACE(input);

    const char8* begin = input.data();
    while (begin != input.null_terminator()) {
      padded_string_view current_input(begin, input.null_terminator());
      SCOPED_TRACE(current_input);
      decode_utf_8_result result = decode_utf_8(current_input);
      assert_eq!(result.size, 1);
      EXPECT_FALSE(result.ok);
      ASSERT_GE(result.size, 1);
      begin += result.size;
    }
  }
}
*/
