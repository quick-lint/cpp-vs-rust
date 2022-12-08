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

#[macro_export]
macro_rules! qljs_assert_diags {
    (
        $errors:expr, // &Vec<AnyDiag>
        $input:expr,  // PaddedStringView
        $diag_0_name:ident {
            $diag_0_field_0:ident: $diag_0_field_0_begin:literal..$diag_0_field_0_end:literal $(,)?
        } $(,)?
    ) => {
        // TODO(port): Better error messages on failure.
        assert_matches!(
            &$errors[..],
            [AnyDiag::$diag_0_name(diag)]
                if offsets_match_begin_end(
                    &diag.$diag_0_field_0,
                    $input,
                    $crate::test::diag_matcher::BeginOffsetLike::to_begin_offset($diag_0_field_0_begin),
                    $crate::test::diag_matcher::EndOffsetLike::to_end_offset(
                        $diag_0_field_0_end,
                        $crate::test::diag_matcher::BeginOffsetLike::to_begin_offset($diag_0_field_0_begin),
                    ),
                )
        );
    };

    (
        $errors:expr, // &Vec<AnyDiag>
        $input:expr,  // PaddedStringView
        $diag_0_name:ident {
            $diag_0_field_0:ident: $diag_0_field_0_begin:literal..$diag_0_field_0_end:literal,
            $diag_0_field_1:ident: $diag_0_field_1_value:literal $(,)?
        } $(,)?
    ) => {
        // TODO(port): Better error messages on failure.
        assert_matches!(
            &$errors[..],
            [AnyDiag::$diag_0_name(diag)]
                if offsets_match_begin_end(
                    &diag.$diag_0_field_0,
                    $input,
                    $crate::test::diag_matcher::BeginOffsetLike::to_begin_offset($diag_0_field_0_begin),
                    $crate::test::diag_matcher::EndOffsetLike::to_end_offset(
                        $diag_0_field_0_end,
                        $crate::test::diag_matcher::BeginOffsetLike::to_begin_offset($diag_0_field_0_begin),
                    ),
                )
                    && diag.$diag_0_field_1 == $diag_0_field_1_value
        );
    };
}

pub trait BeginOffsetLike {
    fn to_begin_offset(self) -> usize;
}
impl BeginOffsetLike for usize {
    fn to_begin_offset(self) -> usize {
        self
    }
}
impl BeginOffsetLike for &[u8] {
    fn to_begin_offset(self) -> usize {
        self.len()
    }
}
impl<const N: usize> BeginOffsetLike for &[u8; N] {
    fn to_begin_offset(self) -> usize {
        self.len()
    }
}

pub trait EndOffsetLike {
    fn to_end_offset(self, begin_offset: usize) -> usize;
}
impl EndOffsetLike for usize {
    fn to_end_offset(self, _begin_offset: usize) -> usize {
        self
    }
}
impl EndOffsetLike for &[u8] {
    fn to_end_offset(self, begin_offset: usize) -> usize {
        begin_offset + self.len()
    }
}
impl<const N: usize> EndOffsetLike for &[u8; N] {
    fn to_end_offset(self, begin_offset: usize) -> usize {
        begin_offset + self.len()
    }
}
