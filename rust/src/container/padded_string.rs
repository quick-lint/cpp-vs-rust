use crate::qljs_assert;
use crate::qljs_const_assert;
use crate::util::narrow_cast::*;

pub type PaddedStringSizeType = i32;

pub const PADDED_STRING_PADDING_SIZE: PaddedStringSizeType = 64;

qljs_const_assert!(
    padded_string::padding_size >= 32, /*::simdjson::SIMDJSON_PADDING*/
    "padded_string must have enough padded to satisfy simdjson",
);

static EMPTY_STRING: [u8; PADDED_STRING_PADDING_SIZE as usize] =
    [0; PADDED_STRING_PADDING_SIZE as usize];

// Like std::string, but guaranteed to have several null bytes at the end.
//
// padded_string enables using SIMD instructions without extra bounds checking.
pub struct PaddedString {
    data: *mut u8,
    size_excluding_padding_bytes: PaddedStringSizeType,
}

impl PaddedString {
    pub fn new() -> PaddedString {
        PaddedString {
            data: EMPTY_STRING.as_ptr() as *mut u8,
            size_excluding_padding_bytes: 0,
        }
    }

    pub fn from_slice(s: &[u8]) -> PaddedString {
        let size_excluding_padding_bytes: PaddedStringSizeType = narrow_cast(s.len());
        let size_including_padding_bytes =
            size_excluding_padding_bytes + PADDED_STRING_PADDING_SIZE;
        unsafe {
            let data: *mut u8 =
                std::alloc::alloc(layout_for_padded_size(size_including_padding_bytes));
            std::ptr::copy_nonoverlapping(s.as_ptr(), data, size_excluding_padding_bytes as usize);
            std::ptr::write_bytes(
                data.offset(size_excluding_padding_bytes as isize),
                0,
                PADDED_STRING_PADDING_SIZE as usize,
            );
            PaddedString {
                data: data,
                size_excluding_padding_bytes: size_excluding_padding_bytes,
            }
        }
    }

    pub fn from_str(s: &str) -> PaddedString {
        PaddedString::from_slice(s.as_bytes())
    }

    pub fn from_string(s: String) -> PaddedString {
        PaddedString::from_str(&s)
    }

    pub fn c_str(&self) -> *const u8 {
        self.data
    }

    pub fn data_ptr(&mut self) -> *mut u8 {
        self.data
    }

    pub fn size(&self) -> PaddedStringSizeType {
        self.size_excluding_padding_bytes
    }

    pub fn padded_size(&self) -> PaddedStringSizeType {
        self.size() + PADDED_STRING_PADDING_SIZE
    }

    pub fn resize(&mut self, new_size: PaddedStringSizeType) {
        let old_size = self.size_excluding_padding_bytes;
        if new_size == old_size {
            // Do nothing.
        } else if new_size < old_size {
            // Shrink. Do not reallocate and copy.
            unsafe {
                std::ptr::write_bytes(
                    self.data.offset(new_size as isize),
                    0,
                    PADDED_STRING_PADDING_SIZE as usize,
                );
            }
            self.size_excluding_padding_bytes = new_size;
        } else {
            // Grow. Need to reallocate and copy.
            self.resize_grow_uninitialized(new_size);
            unsafe {
                std::ptr::write_bytes(
                    self.data.offset(old_size as isize),
                    0,
                    narrow_cast(new_size - old_size),
                );
            }
        }
    }

    pub fn resize_grow_uninitialized(&mut self, new_size: PaddedStringSizeType) {
        let old_size = self.size_excluding_padding_bytes;
        qljs_assert!(new_size > old_size);
        let new_size_including_padding_bytes = new_size + PADDED_STRING_PADDING_SIZE;

        unsafe {
            let new_data: *mut u8 = if self.data == (EMPTY_STRING.as_ptr() as *mut u8) {
                std::alloc::alloc(layout_for_padded_size(new_size_including_padding_bytes))
            } else {
                std::alloc::realloc(
                    self.data,
                    layout_for_padded_size(self.padded_size()),
                    new_size_including_padding_bytes as usize,
                )
            };

            // Only null-terminate. Do not write between &new_data[old_size] and
            // &new_data[new_size].
            std::ptr::write_bytes(
                new_data.offset(new_size as isize),
                0,
                PADDED_STRING_PADDING_SIZE as usize,
            );

            self.data = new_data;
            self.size_excluding_padding_bytes = new_size;
        }
    }

    pub fn null_terminator(&self) -> *const u8 {
        unsafe { self.data.offset(self.size() as isize) }
    }

    // TODO(port): Rename to as_slice.
    pub fn slice(&self) -> &[u8] {
        unsafe {
            std::slice::from_raw_parts(self.data, narrow_cast(self.size_excluding_padding_bytes))
        }
    }

    pub fn view<'a>(&'a self) -> PaddedStringView<'a> {
        PaddedStringView::from(self)
    }
}

impl Drop for PaddedString {
    fn drop(&mut self) {
        if self.data != (EMPTY_STRING.as_ptr() as *mut u8) {
            unsafe {
                std::alloc::dealloc(self.data, layout_for_padded_size(self.padded_size()));
            }
        }
    }
}

fn layout_for_padded_size(padded_size: PaddedStringSizeType) -> std::alloc::Layout {
    std::alloc::Layout::array::<u8>(narrow_cast(padded_size)).unwrap()
}

impl std::fmt::Debug for PaddedString {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        self.view().fmt(formatter)
    }
}

#[derive(Clone, Copy)]
pub struct PaddedStringView<'a> {
    data: *const u8,
    length: PaddedStringSizeType,

    phantom: std::marker::PhantomData<&'a u8>,
}

impl<'a> PaddedStringView<'a> {
    pub fn from(s: &'a PaddedString) -> PaddedStringView<'a> {
        let result = PaddedStringView {
            data: s.c_str(),
            length: s.size(),
            phantom: std::marker::PhantomData,
        };
        qljs_assert!(unsafe { *result.null_terminator() } == 0);
        result
    }

    pub fn from_slice(s: &'a [u8]) -> PaddedStringView<'a> {
        PaddedStringView {
            data: s.as_ptr(),
            length: narrow_cast(s.len()),
            phantom: std::marker::PhantomData,
        }
    }

    pub unsafe fn from_begin_end(begin: *const u8, end: *const u8) -> PaddedStringView<'a> {
        PaddedStringView {
            data: begin,
            length: narrow_cast(end.offset_from(begin)),
            phantom: std::marker::PhantomData,
        }
    }

    pub fn c_str(&self) -> *const u8 {
        self.data
    }

    pub fn size(&self) -> PaddedStringSizeType {
        self.length
    }

    pub fn padded_size(&self) -> PaddedStringSizeType {
        self.size() + PADDED_STRING_PADDING_SIZE
    }

    pub fn null_terminator(&self) -> *const u8 {
        unsafe { self.data.offset(self.length as isize) }
    }

    pub fn slice(&self) -> &'a [u8] {
        unsafe { std::slice::from_raw_parts(self.data, narrow_cast(self.length)) }
    }

    pub fn slice_with_padding(&self) -> &'a [u8] {
        unsafe {
            std::slice::from_raw_parts(
                self.data,
                narrow_cast(self.length + PADDED_STRING_PADDING_SIZE),
            )
        }
    }

    pub fn substr(&self, offset: PaddedStringSizeType) -> PaddedStringView<'a> {
        PaddedStringView::from_slice(&self.slice()[offset as usize..])
    }
}

impl<'a> std::ops::Index<PaddedStringSizeType> for PaddedStringView<'a> {
    type Output = u8;

    fn index(&self, index: PaddedStringSizeType) -> &u8 {
        qljs_assert!(index >= 0);
        qljs_assert!(index <= self.size());
        unsafe { &*self.data.offset(index as isize) }
    }
}

impl<'a> std::fmt::Debug for PaddedStringView<'a> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        String::from_utf8_lossy(self.slice()).fmt(formatter)
    }
}
