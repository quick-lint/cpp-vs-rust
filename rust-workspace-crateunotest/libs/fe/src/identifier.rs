use crate::source_code_span::*;
use cpp_vs_rust_util::narrow_cast::*;

pub struct Identifier<'lexer, 'code> {
    span_begin: *const u8,
    normalized_begin: *const u8,
    span_size: i32,
    normalized_size: i32,

    phantom_lexer: std::marker::PhantomData<&'lexer u8>,
    phantom_code: std::marker::PhantomData<&'code u8>,
}

impl<'lexer, 'code> Identifier<'lexer, 'code> {
    pub fn new(span: SourceCodeSpan<'code>, normalized: &'lexer [u8]) -> Identifier<'lexer, 'code> {
        Identifier {
            span_begin: span.begin_ptr(),
            normalized_begin: normalized.as_ptr(),
            span_size: span.size(),
            normalized_size: narrow_cast(normalized.len()),
            phantom_lexer: std::marker::PhantomData,
            phantom_code: std::marker::PhantomData,
        }
    }

    pub fn span(&self) -> SourceCodeSpan<'code> {
        unsafe {
            SourceCodeSpan::new(
                self.span_begin,
                self.span_begin.offset(narrow_cast(self.span_size)),
            )
        }
    }

    // normalized_name returns the variable's name with escape sequences resolved.
    //
    // For example, a variable named \u{61} in the source code will have
    // normalized_name refer to u8"a".
    //
    // The returned pointers might not reside within the source code string. In
    // other words, the normalized name might be heap-allocated. Call span()
    // instead if you want pointers within the source code input.
    pub fn normalized_name(&self) -> &'lexer [u8] {
        unsafe {
            std::slice::from_raw_parts(self.normalized_begin, narrow_cast(self.normalized_size))
        }
    }
}

impl<'code> Identifier<'code, 'code> {
    // For tests only.
    pub fn from_source_code_span(span: SourceCodeSpan<'code>) -> Identifier<'code, 'code> {
        let span_begin = span.begin_ptr();
        let span_size = span.size();
        Identifier {
            span_begin: span_begin,
            normalized_begin: span_begin,
            span_size: span_size,
            normalized_size: span_size,
            phantom_lexer: std::marker::PhantomData,
            phantom_code: std::marker::PhantomData,
        }
    }
}
