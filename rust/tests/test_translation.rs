#![allow(clippy::redundant_static_lifetimes)]

use cpp_vs_rust::fe::diag_reporter::*;
use cpp_vs_rust::fe::diagnostic::*;
use cpp_vs_rust::fe::diagnostic_formatter::*;
use cpp_vs_rust::fe::diagnostic_types::*;
use cpp_vs_rust::fe::source_code_span::*;
use cpp_vs_rust::i18n::translation::*;

struct BasicTextDiagReporter {
    translator: Translator,
    messages: std::cell::RefCell<Vec<String>>,
}

impl BasicTextDiagReporter {
    fn new(t: Translator) -> BasicTextDiagReporter {
        BasicTextDiagReporter {
            translator: t,
            messages: std::cell::RefCell::new(vec![]),
        }
    }

    fn get_messages(&self) -> Vec<String> {
        self.messages.borrow_mut().clone()
    }
}

impl DiagReporter for BasicTextDiagReporter {
    fn report_impl(&self, type_: DiagType, diag: *const u8) {
        let mut formatter = BasicTextDiagFormatter {
            reporter_messages: &self.messages,
            translator: self.translator.clone(),
            current_message: String::new(),
        };
        formatter.format(get_diagnostic_info(type_), diag);
    }
}

struct BasicTextDiagFormatter<'reporter> {
    reporter_messages: &'reporter std::cell::RefCell<Vec<String>>,
    translator: Translator,
    current_message: String,
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
        message_part: &str,
    ) {
        self.current_message += message_part;
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
    assert_eq!(reporter.get_messages(), vec!["unexpected '#'"],);
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
    assert_eq!(reporter.get_messages(), vec!["#unexpected"],);
}

fn dummy_span() -> SourceCodeSpan<'static> {
    const HELLO: &'static [u8] = b"hello";
    SourceCodeSpan::from_slice(&HELLO[0..5])
}
