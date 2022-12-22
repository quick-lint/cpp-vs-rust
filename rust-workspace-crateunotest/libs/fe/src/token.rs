use crate::buffering_diag_reporter::*;
use crate::diag_reporter::*;
use crate::diagnostic_types::*;
use crate::identifier::*;
use crate::source_code_span::*;
use cpp_vs_rust_container::monotonic_allocator::*;
use cpp_vs_rust_container::vector::*;
use cpp_vs_rust_util::qljs_assert;

#[macro_export]
macro_rules! qljs_case_reserved_keyword_except_await_and_function_and_yield {
    () => {
        $crate::token::TokenType::KWBreak
            | $crate::token::TokenType::KWCase
            | $crate::token::TokenType::KWCatch
            | $crate::token::TokenType::KWClass
            | $crate::token::TokenType::KWConst
            | $crate::token::TokenType::KWContinue
            | $crate::token::TokenType::KWDebugger
            | $crate::token::TokenType::KWDefault
            | $crate::token::TokenType::KWDelete
            | $crate::token::TokenType::KWDo
            | $crate::token::TokenType::KWElse
            | $crate::token::TokenType::KWEnum
            | $crate::token::TokenType::KWExport
            | $crate::token::TokenType::KWExtends
            | $crate::token::TokenType::KWFalse
            | $crate::token::TokenType::KWFinally
            | $crate::token::TokenType::KWFor
            | $crate::token::TokenType::KWIf
            | $crate::token::TokenType::KWImport
            | $crate::token::TokenType::KWIn
            | $crate::token::TokenType::KWInstanceof
            | $crate::token::TokenType::KWNew
            | $crate::token::TokenType::KWNull
            | $crate::token::TokenType::KWReturn
            | $crate::token::TokenType::KWSuper
            | $crate::token::TokenType::KWSwitch
            | $crate::token::TokenType::KWThis
            | $crate::token::TokenType::KWThrow
            | $crate::token::TokenType::KWTrue
            | $crate::token::TokenType::KWTry
            | $crate::token::TokenType::KWTypeof
            | $crate::token::TokenType::KWVar
            | $crate::token::TokenType::KWVoid
            | $crate::token::TokenType::KWWhile
            | $crate::token::TokenType::KWWith
    };
}

#[macro_export]
macro_rules! qljs_case_reserved_keyword_except_function {
    () => {
        $crate::qljs_case_reserved_keyword_except_await_and_function_and_yield!()
            | $crate::token::TokenType::KWAwait
            | $crate::token::TokenType::KWYield
    };
}

#[macro_export]
macro_rules! qljs_case_reserved_keyword_except_await_and_yield {
    () => {
        $crate::qljs_case_reserved_keyword_except_await_and_function_and_yield!()
            | $crate::token::TokenType::KWFunction
    };
}

// Non-contextual keywords, including future reserved words, for non-strict
// mode.
#[macro_export]
macro_rules! qljs_case_reserved_keyword {
    () => {
        $crate::qljs_case_reserved_keyword_except_await_and_yield!()
            | $crate::token::TokenType::KWAwait
            | $crate::token::TokenType::KWYield
    };
}

// Non-contextual keywords, including future reserved words, for strict mode.
// Includes everything from qljs_case_reserved_keyword!().
#[macro_export]
macro_rules! qljs_case_strict_reserved_keyword {
    () => {
        $crate::qljs_case_reserved_keyword!() | qljs_case_strict_only_reserved_keyword!()
    };
}

// Everything in qljs_case_strict_reserved_keyword!() except everything in
// qljs_case_reserved_keyword!().
#[macro_export]
macro_rules! qljs_case_strict_only_reserved_keyword {
    () => {
        $crate::token::TokenType::KWImplements
            | $crate::token::TokenType::KWInterface
            | $crate::token::TokenType::KWPackage
            | $crate::token::TokenType::KWPrivate
            | $crate::token::TokenType::KWProtected
            | $crate::token::TokenType::KWPublic
    };
}

#[macro_export]
macro_rules! qljs_case_typescript_only_contextual_keyword_except_type {
    () => {
        $crate::token::TokenType::KWAbstract
            | $crate::token::TokenType::KWAny
            | $crate::token::TokenType::KWAssert
            | $crate::token::TokenType::KWAsserts
            | $crate::token::TokenType::KWBigint
            | $crate::token::TokenType::KWBoolean
            | $crate::token::TokenType::KWConstructor
            | $crate::token::TokenType::KWDeclare
            | $crate::token::TokenType::KWGlobal
            | $crate::token::TokenType::KWInfer
            | $crate::token::TokenType::KWIntrinsic
            | $crate::token::TokenType::KWIs
            | $crate::token::TokenType::KWKeyof
            | $crate::token::TokenType::KWModule
            | $crate::token::TokenType::KWNamespace
            | $crate::token::TokenType::KWNever
            | $crate::token::TokenType::KWNumber
            | $crate::token::TokenType::KWObject
            | $crate::token::TokenType::KWOut
            | $crate::token::TokenType::KWOverride
            | $crate::token::TokenType::KWReadonly
            | $crate::token::TokenType::KWRequire
            | $crate::token::TokenType::KWString
            | $crate::token::TokenType::KWSymbol
            | $crate::token::TokenType::KWUndefined
            | $crate::token::TokenType::KWUnique
            | $crate::token::TokenType::KWUnknown
    };
}

#[macro_export]
macro_rules! qljs_case_typescript_only_contextual_keyword {
    () => {
        $crate::qljs_case_typescript_only_contextual_keyword_except_type!()
            | $crate::token::TokenType::KWType
    };
}

#[macro_export]
macro_rules! qljs_case_contextual_keyword_except_async_and_get_and_set_and_static_and_type {
    () => {
        $crate::qljs_case_typescript_only_contextual_keyword_except_type!()
            | $crate::token::TokenType::KWAs
            | $crate::token::TokenType::KWFrom
            | $crate::token::TokenType::KWLet
            | $crate::token::TokenType::KWOf
    };
}

#[macro_export]
macro_rules! qljs_case_contextual_keyword_except_async_and_get_and_set {
    () => {
        $crate::qljs_case_contextual_keyword_except_async_and_get_and_set_and_static_and_type!()
            | $crate::token::TokenType::KWStatic
            | $crate::token::TokenType::KWType
    };
}

// Keywords which are sometimes treated as identifiers; i.e. identifiers which
// are sometimes treated as keywords.
#[macro_export]
macro_rules! qljs_case_contextual_keyword {
    () => {
        $crate::qljs_case_contextual_keyword_except_async_and_get_and_set!()
            | $crate::token::TokenType::KWAsync
            | $crate::token::TokenType::KWGet
            | $crate::token::TokenType::KWSet
    };
}

// Any kind of keyword in strict or non-strict mode.
#[macro_export]
macro_rules! qljs_case_keyword {
    () => {
        $crate::qljs_case_contextual_keyword!() | $crate::qljs_case_strict_reserved_keyword!()
    };
}

#[macro_export]
macro_rules! qljs_case_binary_only_operator_symbol_except_less_less_and_star {
    () => {
        $crate::token::TokenType::Ampersand
            | $crate::token::TokenType::AmpersandAmpersand
            | $crate::token::TokenType::BangEqual
            | $crate::token::TokenType::BangEqualEqual
            | $crate::token::TokenType::Circumflex
            | $crate::token::TokenType::EqualEqual
            | $crate::token::TokenType::EqualEqualEqual
            | $crate::token::TokenType::Greater
            | $crate::token::TokenType::GreaterEqual
            | $crate::token::TokenType::GreaterGreater
            | $crate::token::TokenType::GreaterGreaterGreater
            | $crate::token::TokenType::LessEqual
            | $crate::token::TokenType::Percent
            | $crate::token::TokenType::Pipe
            | $crate::token::TokenType::PipePipe
            | $crate::token::TokenType::QuestionQuestion
            | $crate::token::TokenType::StarStar
    };
}

#[macro_export]
macro_rules! qljs_case_binary_only_operator_symbol_except_star {
    () => {
        $crate::qljs_case_binary_only_operator_symbol_except_less_less_and_star!()
            | $crate::token::TokenType::LessLess
    };
}

#[macro_export]
macro_rules! qljs_case_binary_only_operator_symbol {
    () => {
        $crate::qljs_case_binary_only_operator_symbol_except_star!()
            | $crate::token::TokenType::Star
    };
}

#[macro_export]
macro_rules! qljs_case_binary_only_operator {
    () => {
        $crate::qljs_case_binary_only_operator_symbol!() | $crate::token::TokenType::KWInstanceof
    };
}

#[macro_export]
macro_rules! qljs_case_compound_assignment_operator_except_slash_equal {
    () => {
        $crate::token::TokenType::AmpersandEqual
            | $crate::token::TokenType::CircumflexEqual
            | $crate::token::TokenType::GreaterGreaterEqual
            | $crate::token::TokenType::GreaterGreaterGreaterEqual
            | $crate::token::TokenType::LessLessEqual
            | $crate::token::TokenType::MinusEqual
            | $crate::token::TokenType::PercentEqual
            | $crate::token::TokenType::PipeEqual
            | $crate::token::TokenType::PlusEqual
            | $crate::token::TokenType::StarEqual
            | $crate::token::TokenType::StarStarEqual
    };
}

#[macro_export]
macro_rules! qljs_case_compound_assignment_operator {
    () => {
        $crate::token::TokenType::SlashEqual
            | $crate::qljs_case_compound_assignment_operator_except_slash_equal!()
    };
}

#[macro_export]
macro_rules! qljs_case_conditional_assignment_operator {
    () => {
        $crate::token::TokenType::AmpersandAmpersandEqual
            | $crate::token::TokenType::PipePipeEqual
            | $crate::token::TokenType::QuestionQuestionEqual
    };
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TokenType {
    // Single-character symbols:
    Ampersand = '&' as isize,
    Bang = '!' as isize,
    Circumflex = '^' as isize,
    Colon = ':' as isize,
    Comma = ',' as isize,
    Slash = '/' as isize,
    Dot = '.' as isize,
    Equal = '=' as isize,
    Greater = '>' as isize,
    LeftCurly = '{' as isize,
    LeftParen = '(' as isize,
    LeftSquare = '[' as isize,
    Less = '<' as isize,
    Minus = '-' as isize,
    Percent = '%' as isize,
    Pipe = '|' as isize,
    Plus = '+' as isize,
    Question = '?' as isize,
    RightCurly = '}' as isize,
    RightParen = ')' as isize,
    RightSquare = ']' as isize,
    Semicolon = ';' as isize,
    Star = '*' as isize,
    Tilde = '~' as isize,

    CompleteTemplate, // `text` or }text`
    EndOfFile,
    Identifier,
    IncompleteTemplate, // `text${
    Number,
    PrivateIdentifier, // #name
    Regexp,
    String,

    // An identifier which contains escape sequences and which, if unescaped,
    // matches a reserved keyword. For example, the token `\u{69}\u{66}` unescaped
    // is `if`.
    //
    // Such identifiers are sometimes legal and sometimes illegal depending on the
    // parser's context, hence we distinguish them from TokenType::Identifier.
    ReservedKeywordWithEscapeSequence,

    // Reserved words, future reserved words, conditionally reserved words, and
    // contextual keywords ('KW' stands for 'KeyWord'):
    KWAs,
    KWAsync,
    KWAwait,
    KWBreak,
    KWCase,
    KWCatch,
    KWClass,
    KWConst,
    KWContinue,
    KWDebugger,
    KWDefault,
    KWDelete,
    KWDo,
    KWElse,
    KWEnum,
    KWExport,
    KWExtends,
    KWFalse,
    KWFinally,
    KWFor,
    KWFrom,
    KWFunction,
    KWGet,
    KWIf,
    KWImplements,
    KWImport,
    KWIn,
    KWInstanceof,
    KWInterface,
    KWLet,
    KWNew,
    KWNull,
    KWOf,
    KWPackage,
    KWPrivate,
    KWProtected,
    KWPublic,
    KWReturn,
    KWSet,
    KWStatic,
    KWSuper,
    KWSwitch,
    KWThis,
    KWThrow,
    KWTrue,
    KWTry,
    KWTypeof,
    KWVar,
    KWVoid,
    KWWhile,
    KWWith,
    KWYield,

    // TypeScript-only keywords.
    KWAbstract,
    KWAny,
    KWAssert,
    KWAsserts,
    KWBigint,
    KWBoolean,
    KWConstructor,
    KWDeclare,
    KWGlobal,
    KWInfer,
    KWIntrinsic,
    KWIs,
    KWKeyof,
    KWModule,
    KWNamespace,
    KWNever,
    KWNumber,
    KWObject,
    KWOut,
    KWOverride,
    KWReadonly,
    KWRequire,
    KWString,
    KWSymbol,
    KWType,
    KWUndefined,
    KWUnique,
    KWUnknown,

    // Symbols:
    AmpersandAmpersand,         // &&
    AmpersandAmpersandEqual,    // &&=
    AmpersandEqual,             // &=
    BangEqual,                  // !=
    BangEqualEqual,             // !==
    CircumflexEqual,            // ^=
    DotDotDot,                  // ...
    EqualEqual,                 // ==
    EqualEqualEqual,            // ===
    EqualGreater,               // =>
    GreaterEqual,               // >=
    GreaterGreater,             // >>
    GreaterGreaterEqual,        // >>=
    GreaterGreaterGreater,      // >>>
    GreaterGreaterGreaterEqual, // >>>=
    LessEqual,                  // <=
    LessLess,                   // <<
    LessLessEqual,              // <<=
    MinusEqual,                 // -=
    MinusMinus,                 // --
    PercentEqual,               // %=
    PipeEqual,                  // |=
    PipePipe,                   // ||
    PipePipeEqual,              // ||=
    PlusEqual,                  // +=
    PlusPlus,                   // ++
    QuestionDot,                // ?.
    QuestionQuestion,           // ??
    QuestionQuestionEqual,      // ??=
    SlashEqual,                 // /=
    StarEqual,                  // *=
    StarStar,                   // **
    StarStarEqual,              // **=
}

impl std::fmt::Display for TokenType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        // NOTE(port): This used to be in lex-debug.cpp, not token.h.
        write!(f, "{}", to_string(*self))
    }
}

// NOTE(port): This used to be in lex.cpp, not token.h.
fn to_string(token: TokenType) -> &'static str {
    match token {
        TokenType::Ampersand => "ampersand",
        TokenType::AmpersandAmpersand => "ampersand_ampersand",
        TokenType::AmpersandAmpersandEqual => "ampersand_ampersand_equal",
        TokenType::AmpersandEqual => "ampersand_equal",
        TokenType::Bang => "bang",
        TokenType::BangEqual => "bang_equal",
        TokenType::BangEqualEqual => "bang_equal_equal",
        TokenType::Circumflex => "circumflex",
        TokenType::CircumflexEqual => "circumflex_equal",
        TokenType::Colon => "colon",
        TokenType::Comma => "comma",
        TokenType::CompleteTemplate => "complete_template",
        TokenType::Dot => "dot",
        TokenType::DotDotDot => "dot_dot_dot",
        TokenType::EndOfFile => "end_of_file",
        TokenType::Equal => "equal",
        TokenType::EqualEqual => "equal_equal",
        TokenType::EqualEqualEqual => "equal_equal_equal",
        TokenType::EqualGreater => "equal_greater",
        TokenType::Greater => "greater",
        TokenType::GreaterEqual => "greater_equal",
        TokenType::GreaterGreater => "greater_greater",
        TokenType::GreaterGreaterEqual => "greater_greater_equal",
        TokenType::GreaterGreaterGreater => "greater_greater_greater",
        TokenType::GreaterGreaterGreaterEqual => "greater_greater_greater_equal",
        TokenType::Identifier => "identifier",
        TokenType::IncompleteTemplate => "incomplete_template",
        TokenType::KWAbstract => "kw_abstract",
        TokenType::KWAny => "kw_any",
        TokenType::KWAs => "kw_as",
        TokenType::KWAssert => "kw_assert",
        TokenType::KWAsserts => "kw_asserts",
        TokenType::KWAsync => "kw_async",
        TokenType::KWAwait => "kw_await",
        TokenType::KWBigint => "kw_bigint",
        TokenType::KWBoolean => "kw_boolean",
        TokenType::KWBreak => "kw_break",
        TokenType::KWCase => "kw_case",
        TokenType::KWCatch => "kw_catch",
        TokenType::KWClass => "kw_class",
        TokenType::KWConst => "kw_const",
        TokenType::KWConstructor => "kw_constructor",
        TokenType::KWContinue => "kw_continue",
        TokenType::KWDebugger => "kw_debugger",
        TokenType::KWDeclare => "kw_declare",
        TokenType::KWDefault => "kw_default",
        TokenType::KWDelete => "kw_delete",
        TokenType::KWDo => "kw_do",
        TokenType::KWElse => "kw_else",
        TokenType::KWEnum => "kw_enum",
        TokenType::KWExport => "kw_export",
        TokenType::KWExtends => "kw_extends",
        TokenType::KWFalse => "kw_false",
        TokenType::KWFinally => "kw_finally",
        TokenType::KWFor => "kw_for",
        TokenType::KWFrom => "kw_from",
        TokenType::KWFunction => "kw_function",
        TokenType::KWGet => "kw_get",
        TokenType::KWGlobal => "kw_global",
        TokenType::KWIf => "kw_if",
        TokenType::KWImplements => "kw_implements",
        TokenType::KWImport => "kw_import",
        TokenType::KWIn => "kw_in",
        TokenType::KWInfer => "kw_infer",
        TokenType::KWInstanceof => "kw_instanceof",
        TokenType::KWInterface => "kw_interface",
        TokenType::KWIntrinsic => "kw_intrinsic",
        TokenType::KWIs => "kw_is",
        TokenType::KWKeyof => "kw_keyof",
        TokenType::KWLet => "kw_let",
        TokenType::KWModule => "kw_module",
        TokenType::KWNamespace => "kw_namespace",
        TokenType::KWNever => "kw_never",
        TokenType::KWNew => "kw_new",
        TokenType::KWNull => "kw_null",
        TokenType::KWNumber => "kw_number",
        TokenType::KWObject => "kw_object",
        TokenType::KWOf => "kw_of",
        TokenType::KWOut => "kw_out",
        TokenType::KWOverride => "kw_override",
        TokenType::KWPackage => "kw_package",
        TokenType::KWPrivate => "kw_private",
        TokenType::KWProtected => "kw_protected",
        TokenType::KWPublic => "kw_public",
        TokenType::KWReadonly => "kw_readonly",
        TokenType::KWRequire => "kw_require",
        TokenType::KWReturn => "kw_return",
        TokenType::KWSet => "kw_set",
        TokenType::KWStatic => "kw_static",
        TokenType::KWString => "kw_string",
        TokenType::KWSuper => "kw_super",
        TokenType::KWSwitch => "kw_switch",
        TokenType::KWSymbol => "kw_symbol",
        TokenType::KWThis => "kw_this",
        TokenType::KWThrow => "kw_throw",
        TokenType::KWTrue => "kw_true",
        TokenType::KWTry => "kw_try",
        TokenType::KWType => "kw_type",
        TokenType::KWTypeof => "kw_typeof",
        TokenType::KWUndefined => "kw_undefined",
        TokenType::KWUnique => "kw_unique",
        TokenType::KWUnknown => "kw_unknown",
        TokenType::KWVar => "kw_var",
        TokenType::KWVoid => "kw_void",
        TokenType::KWWhile => "kw_while",
        TokenType::KWWith => "kw_with",
        TokenType::KWYield => "kw_yield",
        TokenType::LeftCurly => "left_curly",
        TokenType::LeftParen => "left_paren",
        TokenType::LeftSquare => "left_square",
        TokenType::Less => "less",
        TokenType::LessEqual => "less_equal",
        TokenType::LessLess => "less_less",
        TokenType::LessLessEqual => "less_less_equal",
        TokenType::Minus => "minus",
        TokenType::MinusEqual => "minus_equal",
        TokenType::MinusMinus => "minus_minus",
        TokenType::Number => "number",
        TokenType::Percent => "percent",
        TokenType::PercentEqual => "percent_equal",
        TokenType::Pipe => "pipe",
        TokenType::PipeEqual => "pipe_equal",
        TokenType::PipePipe => "pipe_pipe",
        TokenType::PipePipeEqual => "pipe_pipe_equal",
        TokenType::Plus => "plus",
        TokenType::PlusEqual => "plus_equal",
        TokenType::PlusPlus => "plus_plus",
        TokenType::PrivateIdentifier => "private_identifier",
        TokenType::Question => "question",
        TokenType::QuestionDot => "question_dot",
        TokenType::QuestionQuestion => "question_question",
        TokenType::QuestionQuestionEqual => "question_question_equal",
        TokenType::Regexp => "regexp",
        TokenType::ReservedKeywordWithEscapeSequence => "reserved_keyword_with_escape_sequence",
        TokenType::RightCurly => "right_curly",
        TokenType::RightParen => "right_paren",
        TokenType::RightSquare => "right_square",
        TokenType::Semicolon => "semicolon",
        TokenType::Slash => "slash",
        TokenType::SlashEqual => "slash_equal",
        TokenType::Star => "star",
        TokenType::StarEqual => "star_equal",
        TokenType::StarStar => "star_star",
        TokenType::StarStarEqual => "star_star_equal",
        TokenType::String => "string",
        TokenType::Tilde => "tilde",
    }
}

pub type EscapeSequenceList<'alloc, 'code> =
    BumpVector<'alloc, SourceCodeSpan<'code>, MonotonicAllocator>;

#[derive(Clone, Debug)]
pub struct Token<'alloc, 'code: 'alloc> {
    pub type_: TokenType,

    pub begin: *const u8,
    pub end: *const u8,

    pub has_leading_newline: bool,

    // Used only if this is a keyword token or an identifier token.
    // If the token contains no escape sequences, .normalized_identifier is
    // equivalent to string8_view(.begin, .end).
    pub normalized_identifier: &'alloc [u8],

    pub extras: TokenExtras<'alloc, 'code>,
}

pub union TokenExtras<'alloc, 'code> {
    pub no_data: (),
    // Used only if this is a ReservedKeywordWithEscapeSequence token.
    pub identifier_escape_sequences: &'alloc EscapeSequenceList<'alloc, 'code>,
    // Used only if this is a CompleteTemplate or IncompleteTemplate token.
    pub template_escape_sequence_diagnostics:
        std::mem::ManuallyDrop<Option<&'alloc mut BufferingDiagReporter<'alloc, 'code>>>,
}

impl<'alloc, 'code> std::fmt::Debug for TokenExtras<'alloc, 'code> {
    fn fmt(&self, _formatter: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        // TODO(port-later)
        Ok(())
    }
}

impl<'alloc, 'code> Clone for TokenExtras<'alloc, 'code> {
    fn clone(&self) -> Self {
        unsafe { std::mem::transmute_copy(self) }
    }
}

impl<'alloc, 'code> Token<'alloc, 'code> {
    // NOTE(port): This used to be in lex.cpp, not token.h.
    pub fn identifier_name(&self) -> Identifier<'alloc, 'code> {
        match self.type_ {
            qljs_case_keyword!()
            | TokenType::Identifier
            | TokenType::PrivateIdentifier
            | TokenType::ReservedKeywordWithEscapeSequence => {}
            _ => {
                qljs_assert!(false);
            }
        }
        Identifier::new(self.span(), /*normalized=*/ self.normalized_identifier)
    }

    pub fn span(&self) -> SourceCodeSpan<'code> {
        unsafe { SourceCodeSpan::new(self.begin, self.end) }
    }

    // Report DiagKeywordsCannotContainEscapeSequences for each escape
    // sequence in the most recently parsed keyword-looking identifier.
    //
    // Precondition:
    //   self.type_ == TokenType::ReservedKeywordWithEscapeSequence
    // Precondition: This function was not previously called for the same token.
    pub fn report_errors_for_escape_sequences_in_keyword(&self, reporter: &dyn DiagReporter) {
        qljs_assert!(self.type_ == TokenType::ReservedKeywordWithEscapeSequence);
        let escape_sequences: &EscapeSequenceList =
            unsafe { self.extras.identifier_escape_sequences };
        qljs_assert!(!escape_sequences.is_empty());
        for escape_sequence in escape_sequences.as_slice() {
            report(
                reporter,
                DiagKeywordsCannotContainEscapeSequences {
                    escape_sequence: *escape_sequence,
                },
            );
        }
    }

    // Report errors for each invalid escape sequence in the most recently parsed
    // template.
    //
    // Precondition:
    //   self.type_ == TokenType::CompleteTemplate ||
    //   self.type_ == TokenType::IncompleteTemplate
    // Precondition: This function was not previously called for the same token.
    pub fn report_errors_for_escape_sequences_in_template(&self, reporter: &dyn DiagReporter) {
        qljs_assert!(
            self.type_ == TokenType::CompleteTemplate
                || self.type_ == TokenType::IncompleteTemplate
        );
        match unsafe { &*self.extras.template_escape_sequence_diagnostics } {
            // NOTE(port): In the C++ code, this called move_into. We call copy_into to avoid const
            // correctness issues.
            Some(diags) => {
                diags.copy_into(reporter);
            }
            None => {}
        }
    }
}
