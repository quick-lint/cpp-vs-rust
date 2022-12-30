use crate::qljs_const_assert;

// Convert a Rust string literal into a pointer to a null-terminated array.
//
// The returned type is: *const u8
#[macro_export]
macro_rules! qljs_c_string {
    ($s:expr $(,)?) => {
        concat!($s, '\0').as_bytes().as_ptr()
    };
}

// Returns a str for data up until (but not including) a null terminator.
pub unsafe fn read_utf8_c_string_from_slice(bytes: &[u8]) -> &str {
    std::str::from_utf8_unchecked(&bytes[0..bytes.iter().position(|c| *c == 0).unwrap_unchecked()])
}

// Returns a str for data up until (but not including) a null terminator.
pub unsafe fn read_utf8_c_string_from_c_slice(bytes: &[std::ffi::c_char]) -> &str {
    qljs_const_assert!(std::mem::size_of::<u8>() == std::mem::size_of::<std::ffi::c_char>());
    read_utf8_c_string_from_slice(std::mem::transmute::<&[std::ffi::c_char], &[u8]>(bytes))
}

// Returns a str for data up until (but not including) a null terminator.
pub unsafe fn read_utf8_c_string<'a>(bytes: *const u8) -> &'a str {
    std::str::from_utf8_unchecked(std::ffi::CStr::from_ptr(bytes as *const i8).to_bytes())
}
