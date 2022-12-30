use crate::fe::document::*;
use crate::fe::source_code_span::*;
use crate::util::narrow_cast::*;
use crate::util::padded_string::*;
use crate::util::utf_8::*;

pub type WebDemoSourceOffset = u32;

pub struct WebDemoSourceRange {
    pub begin: WebDemoSourceOffset,
    pub end: WebDemoSourceOffset,
}

pub struct WebDemoLocator<'code> {
    input: PaddedStringView<'code>,
}

impl<'code> WebDemoLocator<'code> {
    pub fn new(input: PaddedStringView<'code>) -> WebDemoLocator<'code> {
        WebDemoLocator { input: input }
    }

    pub fn range(&self, span: SourceCodeSpan<'_>) -> WebDemoSourceRange {
        WebDemoSourceRange {
            begin: self.position(span.begin_ptr()),
            end: self.position(span.end_ptr()),
        }
    }

    pub fn position(&self, c: *const u8) -> WebDemoSourceOffset {
        let byte_offset: i32 = narrow_cast::<i32, _>(unsafe { c.offset_from(self.input.c_str()) });
        narrow_cast::<WebDemoSourceOffset, _>(count_lsp_characters_in_utf_8(
            self.input,
            byte_offset,
        ))
    }
}

impl<'code> LocatorLike<'code> for WebDemoLocator<'code> {
    type RangeType = WebDemoSourceRange;

    fn new(s: PaddedStringView<'code>) -> Self {
        WebDemoLocator::new(s)
    }

    fn range(&self, s: SourceCodeSpan<'_>) -> Self::RangeType {
        self.range(s)
    }
}
