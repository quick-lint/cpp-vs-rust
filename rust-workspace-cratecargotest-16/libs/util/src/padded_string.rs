// TODO(port-later): Find a better crate to put this in. The C++ code had this in 'container', but
// we needed to put it in 'util' to break cyclic dependencies.

use crate::narrow_cast::*;
use crate::qljs_assert;
use crate::qljs_const_assert;

pub type PaddedStringSizeType = i32;

pub const PADDED_STRING_PADDING_LEN: PaddedStringSizeType = 64;

qljs_const_assert!(
    padded_string::padding_len >= 32, /*::simdjson::SIMDJSON_PADDING*/
    "padded_string must have enough padded to satisfy simdjson",
);

static EMPTY_STRING: [u8; PADDED_STRING_PADDING_LEN as usize] =
    [0; PADDED_STRING_PADDING_LEN as usize];

// Like String, but guaranteed to have several null bytes at the end.
//
// PaddedString enables using SIMD instructions without extra bounds checking.
pub struct PaddedString {
    data: *mut u8,
    len_excluding_padding_bytes: PaddedStringSizeType,
    capacity: PaddedStringSizeType,
}

impl PaddedString {
    pub fn new() -> PaddedString {
        PaddedString {
            data: EMPTY_STRING.as_ptr() as *mut u8,
            len_excluding_padding_bytes: 0,
            capacity: 0,
        }
    }

    pub fn from_slice(s: &[u8]) -> PaddedString {
        let len_excluding_padding_bytes: PaddedStringSizeType = narrow_cast(s.len());
        let len_including_padding_bytes = len_excluding_padding_bytes + PADDED_STRING_PADDING_LEN;
        unsafe {
            let layout: std::alloc::Layout = layout_for_padded_len(len_including_padding_bytes);
            let data: *mut u8 = std::alloc::alloc(layout);
            std::ptr::copy_nonoverlapping(s.as_ptr(), data, len_excluding_padding_bytes as usize);
            std::ptr::write_bytes(
                data.offset(len_excluding_padding_bytes as isize),
                0,
                PADDED_STRING_PADDING_LEN as usize,
            );
            PaddedString {
                data: data,
                len_excluding_padding_bytes: len_excluding_padding_bytes,
                capacity: narrow_cast::<PaddedStringSizeType, _>(layout.size()),
            }
        }
    }

    pub fn c_str(&self) -> *const u8 {
        self.data
    }

    pub fn data_ptr(&mut self) -> *mut u8 {
        self.data
    }

    pub fn len(&self) -> PaddedStringSizeType {
        self.len_excluding_padding_bytes
    }

    pub fn padded_len(&self) -> PaddedStringSizeType {
        self.len() + PADDED_STRING_PADDING_LEN
    }

    pub fn resize(&mut self, new_len: PaddedStringSizeType) {
        let old_len = self.len_excluding_padding_bytes;
        if new_len == old_len {
            // Do nothing.
        } else if new_len < old_len {
            // Shrink. Do not reallocate and copy.
            unsafe {
                std::ptr::write_bytes(
                    self.data.offset(new_len as isize),
                    0,
                    PADDED_STRING_PADDING_LEN as usize,
                );
            }
            self.len_excluding_padding_bytes = new_len;
        } else {
            // Grow. Need to reallocate and copy.
            self.resize_grow_uninitialized(new_len);
            unsafe {
                std::ptr::write_bytes(
                    self.data.offset(old_len as isize),
                    0,
                    narrow_cast(new_len - old_len),
                );
            }
        }
    }

    pub fn resize_grow_uninitialized(&mut self, new_len: PaddedStringSizeType) {
        let old_len = self.len_excluding_padding_bytes;
        qljs_assert!(new_len > old_len);
        let new_len_including_padding_bytes = new_len + PADDED_STRING_PADDING_LEN;

        unsafe {
            let new_layout: std::alloc::Layout =
                layout_for_padded_len(new_len_including_padding_bytes);
            let new_data: *mut u8 = if self.data == (EMPTY_STRING.as_ptr() as *mut u8) {
                std::alloc::alloc(new_layout)
            } else {
                std::alloc::realloc(
                    self.data,
                    self.layout(),
                    new_len_including_padding_bytes as usize,
                )
            };

            // Only null-terminate. Do not write between &new_data[old_len] and
            // &new_data[new_len].
            std::ptr::write_bytes(
                new_data.offset(new_len as isize),
                0,
                PADDED_STRING_PADDING_LEN as usize,
            );

            self.data = new_data;
            self.len_excluding_padding_bytes = new_len;
            self.capacity = narrow_cast::<PaddedStringSizeType, _>(new_layout.size());
        }
    }

    pub fn null_terminator(&self) -> *const u8 {
        unsafe { self.data.offset(self.len() as isize) }
    }

    pub fn as_slice(&self) -> &[u8] {
        unsafe {
            std::slice::from_raw_parts(self.data, narrow_cast(self.len_excluding_padding_bytes))
        }
    }

    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        unsafe {
            std::slice::from_raw_parts_mut(self.data, narrow_cast(self.len_excluding_padding_bytes))
        }
    }

    pub fn view<'a>(&'a self) -> PaddedStringView<'a> {
        PaddedStringView::from(self)
    }

    fn layout(&self) -> std::alloc::Layout {
        std::alloc::Layout::array::<u8>(narrow_cast::<usize, _>(self.capacity)).unwrap()
    }
}

impl Drop for PaddedString {
    fn drop(&mut self) {
        if self.data != (EMPTY_STRING.as_ptr() as *mut u8) {
            unsafe {
                std::alloc::dealloc(self.data, self.layout());
            }
        }
    }
}

fn layout_for_padded_len(padded_len: PaddedStringSizeType) -> std::alloc::Layout {
    std::alloc::Layout::array::<u8>(narrow_cast(padded_len)).unwrap()
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
            length: s.len(),
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

    pub fn len(&self) -> PaddedStringSizeType {
        self.length
    }

    pub fn padded_len(&self) -> PaddedStringSizeType {
        self.len() + PADDED_STRING_PADDING_LEN
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
                narrow_cast(self.length + PADDED_STRING_PADDING_LEN),
            )
        }
    }

    pub fn substr(&self, offset: PaddedStringSizeType) -> PaddedStringView<'a> {
        unsafe {
            PaddedStringView::from_begin_end(
                self.data.add(narrow_cast::<usize, _>(offset)),
                self.null_terminator(),
            )
        }
    }
}

impl<'a> std::ops::Index<PaddedStringSizeType> for PaddedStringView<'a> {
    type Output = u8;

    fn index(&self, index: PaddedStringSizeType) -> &u8 {
        qljs_assert!(index >= 0);
        qljs_assert!(index <= self.len());
        unsafe { &*self.data.offset(index as isize) }
    }
}

impl<'a> std::fmt::Debug for PaddedStringView<'a> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        String::from_utf8_lossy(self.slice()).fmt(formatter)
    }
}
