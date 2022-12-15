use cpp_vs_rust::container::padded_string::*;

#[test]
fn new_string_has_following_null_bytes() {
    let padded = PaddedString::new();
    assert_eq!(padded.len(), 0);
    expect_null_terminated(&padded);
}

#[test]
fn empty_string_from_string_has_following_null_bytes() {
    let s = String::from("");
    let padded = PaddedString::from_string(s);
    expect_null_terminated(&padded);
}

#[test]
fn len_excludes_padding_bytes() {
    let s = String::from("hello");
    let padded = PaddedString::from_string(s);
    assert_eq!(padded.len(), 5);
}

#[test]
fn resize_with_bigger_len_adds_new_characters() {
    let mut s = PaddedString::from_str("hello");

    s.resize(10);

    assert_eq!(s.len(), 10);
    assert_eq!(s.as_slice(), "hello\0\0\0\0\0".as_bytes());
    expect_null_terminated(&s);
}

#[test]
fn resize_grow_uninitialized_preserves_original_data() {
    let mut s = PaddedString::from_str("hello");

    s.resize_grow_uninitialized(10);

    assert_eq!(s.len(), 10);
    assert_eq!(&s.as_slice()[0..5], "hello".as_bytes());
    expect_null_terminated(&s);
    // Don't read indexes 5 through 9. The data is uninitialized and could be
    // anything.
}

#[test]
fn resize_with_smaller_len_removes_characters() {
    let mut s = PaddedString::from_str("helloworld");

    s.resize(5);

    assert_eq!(s.len(), 5);
    assert_eq!(s.as_slice(), "hello".as_bytes());
    expect_null_terminated(&s);
}

#[test]
fn debug_format_does_not_include_padding_bytes() {
    let s = PaddedString::from_str("hello");
    assert_eq!(
        format!("BEFORE{s:?}AFTER"),
        format!("BEFORE{:?}AFTER", "hello")
    );
}

#[test]
fn as_slice_excludes_padding_bytes() {
    let s = PaddedString::from_string(String::from("hello"));
    assert_eq!(s.as_slice(), "hello".as_bytes());
}

#[test]
fn shrinking_does_not_reallocate() {
    let mut s = PaddedString::from_str("helloworld");
    let old_data: *mut u8 = s.data_ptr();
    s.resize(5);
    assert_eq!(s.data_ptr(), old_data);
    s.resize(1);
    assert_eq!(s.data_ptr(), old_data);
}

#[test]
fn moving_does_not_invalidate_pointers() {
    let mut s1 = PaddedString::from_str("helloworld");
    let old_s1_data: *mut u8 = s1.data_ptr();
    let mut s2 = s1; // Move.
    assert_eq!(s2.data_ptr(), old_s1_data, "moving should not reallocate");
    assert_eq!(
        s2.as_slice(),
        "helloworld".as_bytes(),
        "moving should not change data"
    );
    expect_null_terminated(&s2);
}

#[test]
fn moving_empty_string_does_not_invalidate_pointers() {
    let mut s1 = PaddedString::new();
    let old_s1_data: *mut u8 = s1.data_ptr();
    let mut s2 = s1; // Move.
    assert_eq!(s2.data_ptr(), old_s1_data, "moving should not reallocate");
    assert_eq!(
        s2.as_slice(),
        "".as_bytes(),
        "moving should not change data"
    );
    expect_null_terminated(&s2);
}

fn expect_null_terminated(s: &PaddedString) {
    let data: *const u8 = s.c_str();
    for i in 0..PADDED_STRING_PADDING_LEN {
        let index = s.len() + i;
        assert_eq!(
            unsafe { *data.offset(index as isize) },
            0x00,
            "index={}",
            index
        );
    }
}
