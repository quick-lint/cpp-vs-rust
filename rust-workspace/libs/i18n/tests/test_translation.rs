#![allow(clippy::redundant_static_lifetimes)]

use cpp_vs_rust_fe::diag_reporter::*;
use cpp_vs_rust_fe::diagnostic::*;
use cpp_vs_rust_fe::diagnostic_formatter::*;
use cpp_vs_rust_fe::diagnostic_types::*;
use cpp_vs_rust_fe::source_code_span::*;
use cpp_vs_rust_i18n::translation::*;

struct BasicTextDiagReporter {
    translator: Translator,
    messages: std::cell::RefCell<Vec<Vec<u8>>>,
}

impl BasicTextDiagReporter {
    fn new(t: Translator) -> BasicTextDiagReporter {
        BasicTextDiagReporter {
            translator: t,
            messages: std::cell::RefCell::new(vec![]),
        }
    }

    fn get_messages(&self) -> Vec<Vec<u8>> {
        self.messages.borrow_mut().clone()
    }
}

impl DiagReporter for BasicTextDiagReporter {
    fn report_impl(&self, type_: DiagType, diag: *const u8) {
        let mut formatter = BasicTextDiagFormatter {
            reporter_messages: &self.messages,
            translator: self.translator.clone(),
            current_message: Vec::<u8>::new(),
        };
        formatter.format(get_diagnostic_info(type_), diag);
    }
}

struct BasicTextDiagFormatter<'reporter> {
    reporter_messages: &'reporter std::cell::RefCell<Vec<Vec<u8>>>,
    translator: Translator,
    current_message: Vec<u8>,
}

impl<'reporter> DiagnosticFormatter for BasicTextDiagFormatter<'reporter> {
    fn write_before_message(
        &mut self,
        _code: &str,
        _severity: DiagnosticSeverity,
        _origin: SourceCodeSpan<'_>,
    ) {
    }

    fn write_message_part(
        &mut self,
        _code: &str,
        _severity: DiagnosticSeverity,
        message_part: &[u8],
    ) {
        self.current_message.extend_from_slice(message_part);
    }

    fn write_after_message(
        &mut self,
        _code: &str,
        _severity: DiagnosticSeverity,
        _origin: SourceCodeSpan<'_>,
    ) {
        self.reporter_messages
            .borrow_mut()
            .push(std::mem::take(&mut self.current_message));
    }

    fn translator(&self) -> Translator {
        self.translator.clone()
    }
}

#[test]
fn c_language_does_not_translate_diagnostics() {
    let mut t = Translator::new_using_messages_from_source_code();
    t.use_messages_from_locale("C");
    let reporter = BasicTextDiagReporter::new(t);
    report(
        &reporter,
        DiagUnexpectedHashCharacter {
            where_: dummy_span(),
        },
    );
    assert_eq!(reporter.get_messages(), vec![b"unexpected '#'"],);
}

#[test]
fn english_snarky_translates() {
    let mut t = Translator::new_using_messages_from_source_code();
    t.use_messages_from_locale("en_US.utf8@snarky");
    let reporter = BasicTextDiagReporter::new(t);
    report(
        &reporter,
        DiagUnexpectedHashCharacter {
            where_: dummy_span(),
        },
    );
    assert_eq!(reporter.get_messages(), vec![b"#unexpected"],);
}

fn dummy_span() -> SourceCodeSpan<'static> {
    const HELLO: &'static [u8] = b"hello";
    SourceCodeSpan::from_slice(&HELLO[0..5])
}
