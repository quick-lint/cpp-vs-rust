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
        $diag_0_name:ident $(,)?
    ) => {
        assert_matches!(&$errors[..], [AnyDiag::$diag_0_name(_)]);
    };

    (
        $errors:expr, // &Vec<AnyDiag>
        $diag_0_name:ident,
        $diag_1_name:ident $(,)?
    ) => {
        assert_matches!(&$errors[..], [AnyDiag::$diag_0_name(_), AnyDiag::$diag_1_name(_)]);
    };

    (
        $errors:expr, // &Vec<AnyDiag>
        $diag_0_name:ident,
        $diag_1_name:ident,
        $diag_2_name:ident $(,)?
    ) => {
        assert_matches!(&$errors[..], [AnyDiag::$diag_0_name(_), AnyDiag::$diag_1_name(_), AnyDiag::$diag_2_name(_)]);
    };

    (
        $errors:expr, // &Vec<AnyDiag>
        $input:expr,  // PaddedStringView
        $diag_0_name:ident $diag_0_fields:tt $(,)?
    ) => {
        // TODO(port): Better error messages on failure.
        assert_matches!(
            &$errors[..],
            [AnyDiag::$diag_0_name(diag)]
                if $crate::qljs_match_diag_fields!(
                    diag,
                    $input,
                    $diag_0_fields,
                )
        );
    };

    (
        $errors:expr, // &Vec<AnyDiag>
        $input:expr,  // PaddedStringView
        $diag_0_name:ident $diag_0_fields:tt,
        $diag_1_name:ident $diag_1_fields:tt $(,)?
    ) => {
        // TODO(port): Better error messages on failure.
        assert_matches!(
            &$errors[..],
            [AnyDiag::$diag_0_name(diag_0), AnyDiag::$diag_1_name(diag_1)]
                if $crate::qljs_match_diag_fields!(
                    diag_0,
                    $input,
                    $diag_0_fields,
                ) && $crate::qljs_match_diag_fields!(
                    diag_1,
                    $input,
                    $diag_1_fields,
                )
        );
    };

    (
        $errors:expr, // &Vec<AnyDiag>
        $input:expr,  // PaddedStringView
        $diag_0_name:ident,
        $diag_1_name:ident $diag_1_fields:tt $(,)?
    ) => {
        // TODO(port): Better error messages on failure.
        assert_matches!(
            &$errors[..],
            [AnyDiag::$diag_0_name(_), AnyDiag::$diag_1_name(diag)]
                if $crate::qljs_match_diag_fields!(
                    diag,
                    $input,
                    $diag_1_fields,
                )
        );
    };
}

#[macro_export]
macro_rules! qljs_match_diag_fields {
    (
        $diag:expr,   // (any Diag struct)
        $input:expr,  // PaddedStringView
        {
            $field_0:ident: $field_0_begin:literal..$field_0_end:literal $(,)?
        } $(,)?
    ) => {
        $crate::qljs_match_diag_field!($diag, $input, $field_0: $field_0_begin..$field_0_end)
    };

    (
        $diag:expr,   // (any Diag struct)
        $input:expr,  // PaddedStringView
        {
            $field_0:ident: $field_0_begin:literal..$field_0_end:tt $(,)?
        } $(,)?
    ) => {
        $crate::qljs_match_diag_field!($diag, $input, $field_0: $field_0_begin..$field_0_end)
    };

    (
        $diag:expr,   // (any Diag struct)
        $input:expr,  // PaddedStringView
        {
            $field_0:ident: $field_0_begin:tt..$field_0_end:literal $(,)?
        } $(,)?
    ) => {
        $crate::qljs_match_diag_field!($diag, $input, $field_0: $field_0_begin..$field_0_end)
    };

    (
        $diag:expr,   // (any Diag struct)
        $input:expr,  // PaddedStringView
        {
            $field_0:ident: $field_0_value:literal $(,)?
        } $(,)?
    ) => {
        $crate::qljs_match_diag_field!($diag, $input, $field_0: $field_0_value)
    };

    (
        $diag:expr,   // (any Diag struct)
        $input:expr,  // PaddedStringView
        {
            $field_0:ident: $field_0_begin:literal..$field_0_end:literal $(,)?
            $field_1:ident: $field_1_value:literal $(,)?
        } $(,)?
    ) => {
        $crate::qljs_match_diag_field!($diag, $input, $field_0: $field_0_begin..$field_0_end)
            && $crate::qljs_match_diag_field!($diag, $input, $field_1: $field_1_value)
    };

    (
        $diag:expr,   // (any Diag struct)
        $input:expr,  // PaddedStringView
        {
            $field_0:ident: $field_0_begin:literal..$field_0_end:tt $(,)?
            $field_1:ident: $field_1_value:literal $(,)?
        } $(,)?
    ) => {
        $crate::qljs_match_diag_field!($diag, $input, $field_0: $field_0_begin..$field_0_end)
            && $crate::qljs_match_diag_field!($diag, $input, $field_1: $field_1_value)
    };
}

#[macro_export]
macro_rules! qljs_match_diag_field {
    (
        $diag:expr,   // (any Diag struct)
        $input:expr,  // PaddedStringView
        $field:ident: $begin:literal..$end:literal $(,)?
    ) => {
        $crate::qljs_match_diag_field!($diag, $input, $field: ($begin)..($end))
    };

    (
        $diag:expr,   // (any Diag struct)
        $input:expr,  // PaddedStringView
        $field:ident: $begin:literal..$end:tt $(,)?
    ) => {
        $crate::qljs_match_diag_field!($diag, $input, $field: ($begin)..$end)
    };

    (
        $diag:expr,   // (any Diag struct)
        $input:expr,  // PaddedStringView
        $field:ident: $begin:tt..$end:literal $(,)?
    ) => {
        $crate::qljs_match_diag_field!($diag, $input, $field: $begin..($end))
    };

    (
        $diag:expr,   // (any Diag struct)
        $input:expr,  // PaddedStringView
        $field:ident: $begin:tt..$end:tt $(,)?
    ) => {{
        let expected_begin_offset: usize =
            $crate::test::diag_matcher::BeginOffsetLike::to_begin_offset($begin);
        let expected_end_offset: usize =
            $crate::test::diag_matcher::EndOffsetLike::to_end_offset($end, expected_begin_offset);
        offsets_match_begin_end(
            &$diag.$field,
            $input,
            expected_begin_offset,
            expected_end_offset,
        )
    }};

    (
        $diag:expr,   // (any Diag struct)
        $input:expr,  // PaddedStringView
        $field:ident: $value:literal $(,)?
    ) => {
        $diag.$field == $value
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
