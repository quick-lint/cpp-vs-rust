#![allow(clippy::redundant_static_lifetimes)]

use cpp_vs_rust::fe::diagnostic::*;
use cpp_vs_rust::fe::diagnostic_formatter::*;
use cpp_vs_rust::fe::identifier::*;
use cpp_vs_rust::fe::language::*;
use cpp_vs_rust::fe::source_code_span::*;
use cpp_vs_rust::qljs_translatable;
use cpp_vs_rust::i18n::translation::*;
use cpp_vs_rust::qljs_offset_of;

fn empty_span() -> SourceCodeSpan<'static> {
    unsafe { SourceCodeSpan::new(std::ptr::null(), std::ptr::null()) }
}

struct StringDiagnosticFormatter {
    message: Vec<u8>,
}

impl StringDiagnosticFormatter {
    fn new() -> StringDiagnosticFormatter {
        StringDiagnosticFormatter {
            message: Vec::<u8>::new(),
        }
    }
}

impl DiagnosticFormatter for StringDiagnosticFormatter {
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
        self.message.extend_from_slice(message_part);
    }

    fn write_after_message(
        &mut self,
        _code: &str,
        _severity: DiagnosticSeverity,
        _origin: SourceCodeSpan<'_>,
    ) {
        self.message.push(b'\n');
    }

    fn translator(&self) -> Translator {
        Translator::new_using_messages_from_source_code()
    }
}

#[test]
fn origin_span() {
    struct TestDiagnosticFormatter {
        expected_span: SourceCodeSpan<'static>,
        write_before_message_call_count: i32,
        write_after_message_call_count: i32,
    }

    impl DiagnosticFormatter for TestDiagnosticFormatter {
        fn write_before_message(
            &mut self,
            _code: &str,
            _severity: DiagnosticSeverity,
            origin: SourceCodeSpan<'_>,
        ) {
            assert!(same_pointers(origin, self.expected_span));
            self.write_before_message_call_count += 1;
        }

        fn write_message_part(
            &mut self,
            _code: &str,
            _severity: DiagnosticSeverity,
            _message_part: &[u8],
        ) {
        }

        fn write_after_message(
            &mut self,
            _code: &str,
            _severity: DiagnosticSeverity,
            origin: SourceCodeSpan<'_>,
        ) {
            assert!(same_pointers(origin, self.expected_span));
            self.write_after_message_call_count += 1;
        }

        fn translator(&self) -> Translator {
            Translator::new_using_messages_from_source_code()
        }
    }

    const CODE: &'static [u8] = b"hello world";
    let span: SourceCodeSpan<'static> = SourceCodeSpan::from_slice(&CODE[0..5]);

    let mut formatter = TestDiagnosticFormatter {
        expected_span: span,
        write_before_message_call_count: 0,
        write_after_message_call_count: 0,
    };
    formatter.format_message(
        "E9999",
        DiagnosticSeverity::Error,
        qljs_translatable!("something happened"),
        &[
            DiagnosticMessageArgInfo::new(0, DiagnosticArgType::SourceCodeSpan),
            DiagnosticMessageArgInfo::empty(),
            DiagnosticMessageArgInfo::empty(),
        ],
        &span as *const _ as *const u8,
    );

    assert_eq!(formatter.write_before_message_call_count, 1);
    assert_eq!(formatter.write_after_message_call_count, 1);
}

#[test]
fn single_span_simple_message() {
    let mut formatter = StringDiagnosticFormatter::new();
    formatter.format_message(
        "E9999",
        DiagnosticSeverity::Error,
        qljs_translatable!("something happened"),
        &[
            DiagnosticMessageArgInfo::new(0, DiagnosticArgType::SourceCodeSpan),
            DiagnosticMessageArgInfo::empty(),
            DiagnosticMessageArgInfo::empty(),
        ],
        &empty_span() as *const _ as *const u8,
    );
    assert_eq!(formatter.message, b"something happened\n");
}

#[test]
fn diagnostic_with_single_message() {
    let info = DiagnosticInfo::new(
        9999,
        DiagnosticSeverity::Error,
        [
            qljs_translatable!("something happened"),
            TranslatableMessage::unallocated(),
        ],
        [
            [
                DiagnosticMessageArgInfo::new(0, DiagnosticArgType::SourceCodeSpan),
                DiagnosticMessageArgInfo::empty(),
                DiagnosticMessageArgInfo::empty(),
            ],
            [
                DiagnosticMessageArgInfo::empty(),
                DiagnosticMessageArgInfo::empty(),
                DiagnosticMessageArgInfo::empty(),
            ],
        ],
    );

    let mut formatter = StringDiagnosticFormatter::new();
    formatter.format(&info, &empty_span() as *const _ as *const u8);
    assert_eq!(formatter.message, b"something happened\n");
}

#[test]
fn diagnostic_with_two_messages() {
    let info = DiagnosticInfo::new(
        9999,
        DiagnosticSeverity::Error,
        [
            qljs_translatable!("something happened"),
            qljs_translatable!("see here"),
        ],
        [
            [
                DiagnosticMessageArgInfo::new(0, DiagnosticArgType::SourceCodeSpan),
                DiagnosticMessageArgInfo::empty(),
                DiagnosticMessageArgInfo::empty(),
            ],
            [
                DiagnosticMessageArgInfo::new(0, DiagnosticArgType::SourceCodeSpan),
                DiagnosticMessageArgInfo::empty(),
                DiagnosticMessageArgInfo::empty(),
            ],
        ],
    );

    let mut formatter = StringDiagnosticFormatter::new();
    formatter.format(&info, &empty_span() as *const _ as *const u8);
    assert_eq!(formatter.message, b"something happened\nsee here\n");
}

#[test]
fn message_with_zero_placeholder() {
    const CODE: &'static [u8] = b"hello world";
    let hello_span: SourceCodeSpan<'static> = SourceCodeSpan::from_slice(&CODE[0..5]);

    let mut formatter = StringDiagnosticFormatter::new();
    formatter.format_message(
        "E9999",
        DiagnosticSeverity::Error,
        qljs_translatable!("this {0} looks fishy"),
        &[
            DiagnosticMessageArgInfo::new(0, DiagnosticArgType::SourceCodeSpan),
            DiagnosticMessageArgInfo::empty(),
            DiagnosticMessageArgInfo::empty(),
        ],
        &hello_span as *const _ as *const u8,
    );
    assert_eq!(formatter.message, b"this hello looks fishy\n");
}

#[test]
fn message_with_extra_identifier_placeholder() {
    const CODE: &'static [u8] = b"hello world";

    struct TestDiag {
        hello: SourceCodeSpan<'static>,
        world: Identifier<'static, 'static>,
    }
    let diag = TestDiag {
        hello: SourceCodeSpan::from_slice(&CODE[0..5]),
        world: Identifier::from_source_code_span(SourceCodeSpan::from_slice(&CODE[6..11])),
    };

    let mut formatter = StringDiagnosticFormatter::new();
    formatter.format_message(
        "E9999",
        DiagnosticSeverity::Error,
        qljs_translatable!("this {1} looks fishy"),
        &[
            DiagnosticMessageArgInfo::new(
                qljs_offset_of!(TestDiag, hello),
                DiagnosticArgType::SourceCodeSpan,
            ),
            DiagnosticMessageArgInfo::new(
                qljs_offset_of!(TestDiag, world),
                DiagnosticArgType::Identifier,
            ),
            DiagnosticMessageArgInfo::empty(),
        ],
        &diag as *const _ as *const u8,
    );
    assert_eq!(formatter.message, b"this world looks fishy\n");
}

#[test]
fn message_with_multiple_span_placeholders() {
    const CODE: &'static [u8] = b"let me = be(free);";
    struct TestDiag {
        let_span: SourceCodeSpan<'static>,
        me_span: SourceCodeSpan<'static>,
        be_span: SourceCodeSpan<'static>,
    }
    let diag = TestDiag {
        let_span: SourceCodeSpan::from_slice(&CODE[0..3]),
        me_span: SourceCodeSpan::from_slice(&CODE[4..6]),
        be_span: SourceCodeSpan::from_slice(&CODE[9..11]),
    };
    assert_eq!(diag.let_span.as_slice(), b"let");
    assert_eq!(diag.me_span.as_slice(), b"me");
    assert_eq!(diag.be_span.as_slice(), b"be");

    let mut formatter = StringDiagnosticFormatter::new();
    formatter.format_message(
        "E9999",
        DiagnosticSeverity::Error,
        qljs_translatable!("free {1} and {0} {1} {2}"),
        &[
            DiagnosticMessageArgInfo::new(
                qljs_offset_of!(TestDiag, let_span),
                DiagnosticArgType::SourceCodeSpan,
            ),
            DiagnosticMessageArgInfo::new(
                qljs_offset_of!(TestDiag, me_span),
                DiagnosticArgType::SourceCodeSpan,
            ),
            DiagnosticMessageArgInfo::new(
                qljs_offset_of!(TestDiag, be_span),
                DiagnosticArgType::SourceCodeSpan,
            ),
        ],
        &diag as *const _ as *const u8,
    );
    assert_eq!(formatter.message, b"free me and let me be\n");
}

#[test]
fn message_with_char_placeholder() {
    struct TestDiag {
        span: SourceCodeSpan<'static>,
        c: u8,
    }
    let diag = TestDiag {
        span: empty_span(),
        c: b'Q',
    };
    let mut formatter = StringDiagnosticFormatter::new();
    formatter.format_message(
        "E9999",
        DiagnosticSeverity::Error,
        qljs_translatable!("what is this '{1}' nonsense?"),
        &[
            DiagnosticMessageArgInfo::new(
                qljs_offset_of!(TestDiag, span),
                DiagnosticArgType::SourceCodeSpan,
            ),
            DiagnosticMessageArgInfo::new(qljs_offset_of!(TestDiag, c), DiagnosticArgType::Char8),
            DiagnosticMessageArgInfo::empty(),
        ],
        &diag as *const _ as *const u8,
    );
    assert_eq!(formatter.message, b"what is this 'Q' nonsense?\n");
}

#[test]
fn message_with_escaped_curlies() {
    const CODE: &'static [u8] = b"hello world";
    let code_span = SourceCodeSpan::from_slice(&CODE[0..3]);

    let mut formatter = StringDiagnosticFormatter::new();
    formatter.format_message(
        "E9999",
        DiagnosticSeverity::Error,
        qljs_translatable!("a {{0} b }} c"),
        &[
            DiagnosticMessageArgInfo::new(0, DiagnosticArgType::SourceCodeSpan),
            DiagnosticMessageArgInfo::empty(),
            DiagnosticMessageArgInfo::empty(),
        ],
        &code_span as *const _ as *const u8,
    );
    assert_eq!(formatter.message, b"a {0} b }} c\n");
}

#[test]
fn enum_kind_placeholder() {
    struct TestDiag {
        empty_span: SourceCodeSpan<'static>,
        kind: EnumKind,
    }
    let message_args: DiagnosticMessageArgs = [
        DiagnosticMessageArgInfo::new(
            qljs_offset_of!(TestDiag, empty_span),
            DiagnosticArgType::SourceCodeSpan,
        ),
        DiagnosticMessageArgInfo::new(qljs_offset_of!(TestDiag, kind), DiagnosticArgType::EnumKind),
        DiagnosticMessageArgInfo::empty(),
    ];

    {
        let diag = TestDiag {
            empty_span: empty_span(),
            kind: EnumKind::Normal,
        };
        let mut formatter = StringDiagnosticFormatter::new();
        formatter.format_message(
            "E9999",
            DiagnosticSeverity::Error,
            qljs_translatable!("expected {1:headlinese}"),
            &message_args,
            &diag as *const _ as *const u8,
        );
        assert_eq!(formatter.message, b"expected enum\n");
    }
}

#[test]
fn statement_kind_placeholder() {
    struct TestDiag {
        empty_span: SourceCodeSpan<'static>,
        statement: StatementKind,
    }
    let message_args: DiagnosticMessageArgs = [
        DiagnosticMessageArgInfo::new(
            qljs_offset_of!(TestDiag, empty_span),
            DiagnosticArgType::SourceCodeSpan,
        ),
        DiagnosticMessageArgInfo::new(
            qljs_offset_of!(TestDiag, statement),
            DiagnosticArgType::StatementKind,
        ),
        DiagnosticMessageArgInfo::empty(),
    ];

    {
        let diag = TestDiag {
            empty_span: empty_span(),
            statement: StatementKind::DoWhileLoop,
        };
        let mut formatter = StringDiagnosticFormatter::new();
        formatter.format_message(
            "E9999",
            DiagnosticSeverity::Error,
            qljs_translatable!("expected {1:headlinese}"),
            &message_args,
            &diag as *const _ as *const u8,
        );
        assert_eq!(formatter.message, b"expected 'do-while' loop\n");
    }

    {
        let diag = TestDiag {
            empty_span: empty_span(),
            statement: StatementKind::DoWhileLoop,
        };
        let mut formatter = StringDiagnosticFormatter::new();
        formatter.format_message(
            "E9999",
            DiagnosticSeverity::Error,
            qljs_translatable!("expected {1:singular}"),
            &message_args,
            &diag as *const _ as *const u8,
        );
        assert_eq!(formatter.message, b"expected a 'do-while' loop\n");
    }

    {
        let diag = TestDiag {
            empty_span: empty_span(),
            statement: StatementKind::ForLoop,
        };
        let mut formatter = StringDiagnosticFormatter::new();
        formatter.format_message(
            "E9999",
            DiagnosticSeverity::Error,
            qljs_translatable!("expected {1:headlinese}"),
            &message_args,
            &diag as *const _ as *const u8,
        );
        assert_eq!(formatter.message, b"expected 'for' loop\n");
    }

    {
        let diag = TestDiag {
            empty_span: empty_span(),
            statement: StatementKind::ForLoop,
        };
        let mut formatter = StringDiagnosticFormatter::new();
        formatter.format_message(
            "E9999",
            DiagnosticSeverity::Error,
            qljs_translatable!("expected {1:singular}"),
            &message_args,
            &diag as *const _ as *const u8,
        );
        assert_eq!(formatter.message, b"expected a 'for' loop\n");
    }

    {
        let diag = TestDiag {
            empty_span: empty_span(),
            statement: StatementKind::IfStatement,
        };
        let mut formatter = StringDiagnosticFormatter::new();
        formatter.format_message(
            "E9999",
            DiagnosticSeverity::Error,
            qljs_translatable!("expected {1:headlinese}"),
            &message_args,
            &diag as *const _ as *const u8,
        );
        assert_eq!(formatter.message, b"expected 'if' statement\n");
    }

    {
        let diag = TestDiag {
            empty_span: empty_span(),
            statement: StatementKind::IfStatement,
        };
        let mut formatter = StringDiagnosticFormatter::new();
        formatter.format_message(
            "E9999",
            DiagnosticSeverity::Error,
            qljs_translatable!("expected {1:singular}"),
            &message_args,
            &diag as *const _ as *const u8,
        );
        assert_eq!(formatter.message, b"expected an 'if' statement\n");
    }

    {
        let diag = TestDiag {
            empty_span: empty_span(),
            statement: StatementKind::WhileLoop,
        };
        let mut formatter = StringDiagnosticFormatter::new();
        formatter.format_message(
            "E9999",
            DiagnosticSeverity::Error,
            qljs_translatable!("expected {1:headlinese}"),
            &message_args,
            &diag as *const _ as *const u8,
        );
        assert_eq!(formatter.message, b"expected 'while' loop\n");
    }

    {
        let diag = TestDiag {
            empty_span: empty_span(),
            statement: StatementKind::WhileLoop,
        };
        let mut formatter = StringDiagnosticFormatter::new();
        formatter.format_message(
            "E9999",
            DiagnosticSeverity::Error,
            qljs_translatable!("expected {1:singular}"),
            &message_args,
            &diag as *const _ as *const u8,
        );
        assert_eq!(formatter.message, b"expected a 'while' loop\n");
    }

    {
        let diag = TestDiag {
            empty_span: empty_span(),
            statement: StatementKind::WithStatement,
        };
        let mut formatter = StringDiagnosticFormatter::new();
        formatter.format_message(
            "E9999",
            DiagnosticSeverity::Error,
            qljs_translatable!("expected {1:headlinese}"),
            &message_args,
            &diag as *const _ as *const u8,
        );
        assert_eq!(formatter.message, b"expected 'with' statement\n");
    }

    {
        let diag = TestDiag {
            empty_span: empty_span(),
            statement: StatementKind::WithStatement,
        };
        let mut formatter = StringDiagnosticFormatter::new();
        formatter.format_message(
            "E9999",
            DiagnosticSeverity::Error,
            qljs_translatable!("expected {1:singular}"),
            &message_args,
            &diag as *const _ as *const u8,
        );
        assert_eq!(formatter.message, b"expected a 'with' statement\n");
    }

    {
        let diag = TestDiag {
            empty_span: empty_span(),
            statement: StatementKind::LabelledStatement,
        };
        let mut formatter = StringDiagnosticFormatter::new();
        formatter.format_message(
            "E9999",
            DiagnosticSeverity::Error,
            qljs_translatable!("expected {1:headlinese}"),
            &message_args,
            &diag as *const _ as *const u8,
        );
        assert_eq!(formatter.message, b"expected labelled statement\n");
    }

    {
        let diag = TestDiag {
            empty_span: empty_span(),
            statement: StatementKind::LabelledStatement,
        };
        let mut formatter = StringDiagnosticFormatter::new();
        formatter.format_message(
            "E9999",
            DiagnosticSeverity::Error,
            qljs_translatable!("expected {1:singular}"),
            &message_args,
            &diag as *const _ as *const u8,
        );
        assert_eq!(formatter.message, b"expected a labelled statement\n");
    }
}
