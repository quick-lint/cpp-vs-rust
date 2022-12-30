use crate::c_api::web_demo_location::*;
use crate::fe::source_code_span::*;
use crate::test::characters::*;
use crate::util::narrow_cast::*;
use crate::util::padded_string::*;

#[test]
fn ranges_on_first_line() {
    let code = PaddedString::from_slice(b"let x = 2;");
    let l: WebDemoLocator = WebDemoLocator::new(code.view());
    let x_range: WebDemoSourceRange =
        l.range(unsafe { SourceCodeSpan::new(code.c_str().add(4), code.c_str().add(5)) });

    assert_eq!(x_range.begin, 4);
    assert_eq!(x_range.end, 5);
}

#[test]
fn ranges_on_second_line() {
    for line_terminator in LINE_TERMINATORS_EXCEPT_LS_PS {
        let code: PaddedString =
            PaddedString::from_slice(format!("let x = 2;{line_terminator}let y = 3;").as_bytes());
        let y: *const u8 = strchr(code.view(), b'y');
        let l: WebDemoLocator = WebDemoLocator::new(code.view());
        let x_range: WebDemoSourceRange = l.range(unsafe { SourceCodeSpan::new(y, y.add(1)) });

        assert_eq!(x_range.begin, unsafe { y.offset_from(code.c_str()) }
            as WebDemoSourceOffset);
        assert_eq!(x_range.end, unsafe { y.add(1).offset_from(code.c_str()) }
            as WebDemoSourceOffset);
    }
}

#[test]
fn lf_cr_is_two_line_terminators() {
    let code: PaddedString = PaddedString::from_slice(b"let x = 2;\n\rlet y = 3;");
    let y: *const u8 = strchr(code.view(), b'y');
    let l: WebDemoLocator = WebDemoLocator::new(code.view());
    let y_range: WebDemoSourceRange = l.range(unsafe { SourceCodeSpan::new(y, y.add(1)) });

    assert_eq!(y_range.begin, unsafe { y.offset_from(code.c_str()) }
        as WebDemoSourceOffset);
}

#[test]
fn location_after_null_byte() {
    let code: PaddedString = PaddedString::from_slice(b"hello\0beautiful\nworld");
    let r: *const u8 = unsafe { code.c_str().add(18) };
    assert_eq!(unsafe { *r }, b'r');

    let l: WebDemoLocator = WebDemoLocator::new(code.view());
    let r_range: WebDemoSourceRange = l.range(unsafe { SourceCodeSpan::new(r, r.add(1)) });

    assert_eq!(r_range.begin, unsafe { r.offset_from(code.c_str()) }
        as WebDemoSourceOffset);
}

#[test]
fn position_backwards() {
    let code: PaddedString = PaddedString::from_slice(b"ab\nc\n\nd\nefg\nh");

    let mut expected_positions: Vec<WebDemoSourceOffset> = vec![];
    {
        let l: WebDemoLocator = WebDemoLocator::new(code.view());
        for i in 0..(narrow_cast::<i32, _>(code.len())) {
            expected_positions.push(l.position(unsafe { code.c_str().add(i as usize) }));
        }
    }

    let mut actual_positions: Vec<WebDemoSourceOffset> = vec![];
    {
        let l: WebDemoLocator = WebDemoLocator::new(code.view());
        let mut i: i32 = narrow_cast::<i32, _>(code.len()) - 1;
        while i >= 0 {
            actual_positions.push(l.position(unsafe { code.c_str().add(i as usize) }));
            i -= 1;
        }
    }
    actual_positions.reverse();

    assert_eq!(actual_positions, expected_positions);
}

#[test]
fn position_after_multi_byte_character() {
    // U+2603 has three UTF-8 code units: e2 98 83
    // U+2603 has one UTF-16 code unit: 2603
    let code: PaddedString = PaddedString::from_slice("\u{2603} x".as_bytes());
    let x: *const u8 = strchr(code.view(), b'x');
    let l: WebDemoLocator = WebDemoLocator::new(code.view());
    assert_eq!(l.position(x), 2);
}

#[test]
fn position_after_wide_multi_byte_character() {
    // U+1f496 has four UTF-8 code units: f0 9f 92 96
    // U+1f496 has two UTF-16 code units: D83D DC96
    let code: PaddedString = PaddedString::from_slice("\u{01f496} x".as_bytes());
    let x: *const u8 = strchr(code.view(), b'x');
    let l: WebDemoLocator = WebDemoLocator::new(code.view());
    assert_eq!(l.position(x), 3);
}

fn strchr(haystack: PaddedStringView, needle: u8) -> *const u8 {
    let position: Option<usize> = haystack.slice().iter().position(|c: &u8| *c == needle);
    unsafe { haystack.c_str().add(position.unwrap()) }
}
