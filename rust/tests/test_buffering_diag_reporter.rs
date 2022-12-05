use cpp_vs_rust::assert_matches;
use cpp_vs_rust::container::monotonic_allocator::*;
use cpp_vs_rust::container::padded_string::*;
use cpp_vs_rust::fe::buffering_diag_reporter::*;
use cpp_vs_rust::fe::diag_reporter::*;
use cpp_vs_rust::fe::diagnostic_types::*;
use cpp_vs_rust::fe::source_code_span::*;
use cpp_vs_rust::test::diag_collector::*;

#[test]
fn buffers_all_visits() {
    let bom_code = PaddedString::from_str("bom");
    let string_code = PaddedString::from_str("\"");

    let memory = MonotonicAllocator::new("test");
    let mut diag_reporter = BufferingDiagReporter::new(&memory);
    report(
        &diag_reporter,
        DiagUnexpectedBomBeforeShebang {
            bom: span_of(&bom_code),
        },
    );
    report(
        &diag_reporter,
        DiagInvalidQuotesAroundStringLiteral {
            opening_quote: span_of(&string_code),
            suggested_quote: b'\'',
        },
    );

    let collector = DiagCollector::new();
    diag_reporter.move_into(&collector);

    assert_eq!(collector.len(), 2);
    assert_matches!(
        collector.index(0),
        AnyDiag::DiagUnexpectedBomBeforeShebang(diag)
            if same_pointers(diag.bom, span_of(&bom_code)),
    );
    assert_matches!(
        collector.index(1),
        AnyDiag::DiagInvalidQuotesAroundStringLiteral(diag)
            if same_pointers(diag.opening_quote, span_of(&string_code))
                && diag.suggested_quote == b'\'',
    );
}

#[test]
fn not_destructing_does_not_leak() {
    // This test relies on a leak checker such as Valgrind's memtest or LLVM's LeakSanitizer.

    let memory = MonotonicAllocator::new("test");
    let diag_reporter = BufferingDiagReporter::new(&memory);

    let bom_code = PaddedString::from_str("bom");
    report(
        &diag_reporter,
        DiagUnexpectedBomBeforeShebang {
            bom: span_of(&bom_code),
        },
    );

    // Destruct memory, but don't drop diag_reporter.
    std::mem::forget(diag_reporter);
}

fn span_of<'a>(code: &'a PaddedString) -> SourceCodeSpan<'a> {
    unsafe { SourceCodeSpan::new(code.c_str(), code.null_terminator()) }
}
