use crate::container::padded_string::*;
use crate::fe::source_code_span::*;
use crate::util::narrow_cast::*;

// TODO(port): Create a higher-fidelity analog to diag_matcher from the C++ code.

// Checks that the SourceCodeSpan begins at begin_offset and ends at begin_offset+text.len().
//
// TODO(strager): Also ensure the SourceCodeSpan's content equals text.
pub fn offsets_match(
    span: &SourceCodeSpan,
    code: PaddedStringView,
    begin_offset: usize,
    text: &[u8],
) -> bool {
    offsets_match_begin_end(span, code, begin_offset, begin_offset + text.len())
}

// Checks that the SourceCodeSpan begins at begin_offset and ends at end_offset.
pub fn offsets_match_begin_end(
    span: &SourceCodeSpan,
    code: PaddedStringView,
    begin_offset: usize,
    end_offset: usize,
) -> bool {
    let span_begin_offset: usize =
        narrow_cast::<usize, _>(unsafe { span.begin_ptr().offset_from(code.c_str()) });
    let span_end_offset: usize =
        narrow_cast::<usize, _>(unsafe { span.end_ptr().offset_from(code.c_str()) });
    span_begin_offset == begin_offset && span_end_offset == end_offset
}
