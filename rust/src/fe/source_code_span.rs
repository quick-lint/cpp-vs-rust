use crate::container::winkable::*;
use crate::util::narrow_cast::*;

// TODO(port): Custom implementation of Debug.
#[derive(Clone, Copy, Debug)]
pub struct SourceCodeSpan<'code> {
    begin: *const u8,
    end: *const u8,

    phantom: std::marker::PhantomData<&'code u8>,
}

impl<'code> SourceCodeSpan<'code> {
    // A source_code_span with no contained characters.
    // TODO(port): Is this interface sane?
    pub unsafe fn unit(c: *const u8) -> SourceCodeSpan<'code> {
        SourceCodeSpan {
            begin: c,
            end: c,
            phantom: std::marker::PhantomData,
        }
    }

    // TODO(port): Is this interface sane?
    pub unsafe fn new(begin: *const u8, end: *const u8) -> SourceCodeSpan<'code> {
        SourceCodeSpan {
            begin: begin,
            end: end,
            phantom: std::marker::PhantomData,
        }
    }

    pub fn from_slice(slice: &'code [u8]) -> SourceCodeSpan<'code> {
        let begin: *const u8 = slice.as_ptr();
        unsafe { SourceCodeSpan::new(begin, begin.add(slice.len())) }
    }

    pub fn begin_ptr(&self) -> *const u8 {
        self.begin
    }

    pub fn end_ptr(&self) -> *const u8 {
        self.end
    }

    pub fn as_slice(&self) -> &'code [u8] {
        unsafe { std::slice::from_raw_parts(self.begin, self.size() as usize) }
    }

    pub fn size(&self) -> i32 {
        narrow_cast(unsafe { self.end.offset_from(self.begin) })
    }
}

impl<'code> Winkable for SourceCodeSpan<'code> {}

// Returns true of the given source_code_span-s refer to the same span of code
// (i.e. are completely identical).
pub fn same_pointers(a: SourceCodeSpan, b: SourceCodeSpan) -> bool {
    a.begin == b.begin && a.end == b.end
}
