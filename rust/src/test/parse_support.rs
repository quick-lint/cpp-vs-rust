use crate::qljs_assert;
use lazy_static::lazy_static;

// Escape the first character in the given keyword with a JavaScript identifier
// escape sequence (\u{..}).
//
// Example: break -> \u{62}reak
//
// The returned string will always be 5 bytes longer: +6 bytes for \u{??} and -1
// byte for the replaced character.
pub fn escape_first_character_in_keyword(keyword: &str) -> String {
    let alphabet: &[u8; 16] = b"0123456789abcdef";
    let mut result = String::new();
    let expected_len: usize = keyword.len() + 6 - 1;
    result.reserve(expected_len);
    result += "\\u{";
    unsafe {
        let first_keyword_char: usize = keyword.as_bytes()[0] as usize;
        result.push(char::from_u32_unchecked(
            alphabet[(first_keyword_char >> 4) & 0xf] as u32,
        ));
        result.push(char::from_u32_unchecked(
            alphabet[(first_keyword_char >> 0) & 0xf] as u32,
        ));
    }
    result.push('}');
    result += &keyword[1..];
    qljs_assert!(result.len() == expected_len);
    result
}

macro_rules! string_set {
    ($($values:literal),* $(,)?) => {
        std::collections::BTreeSet::<String>::from([
            $(String::from($values),)*
        ])
    };
}

lazy_static! {
    // Identifiers which are ReservedWord-s only in strict mode.
    // https://262.ecma-international.org/11.0/#sec-keywords-and-reserved-words
    pub static ref STRICT_ONLY_RESERVED_KEYWORDS: std::collections::BTreeSet<String> = string_set![
        "implements", "interface", "package",
        "private",    "protected", "public",
    ];

    // Exclusions from BindingIdentifier (ReservedWord except 'await' and 'yield')
    // https://262.ecma-international.org/11.0/#prod-ReservedWord
    // https://262.ecma-international.org/11.0/#prod-BindingIdentifier
    pub static ref DISALLOWED_BINDING_IDENTIFIER_KEYWORDS: std::collections::BTreeSet<String> = string_set![
        "break",    "case",       "catch",    "class",   "const",
        "continue", "debugger",   "default",  "delete",  "do",
        "else",     "enum",       "export",   "extends", "false",
        "finally",  "for",        "function", "if",      "import",
        "in",       "instanceof", "new",      "null",    "return",
        "super",    "switch",     "this",     "throw",   "true",
        "try",      "typeof",     "var",      "void",    "while",
        "with",
    ];
    pub static ref STRICT_DISALLOWED_BINDING_IDENTIFIER_KEYWORDS: std::collections::BTreeSet<String> =
        &*DISALLOWED_BINDING_IDENTIFIER_KEYWORDS | &*STRICT_ONLY_RESERVED_KEYWORDS;

    // ReservedWord in non-strict mode.
    // https://262.ecma-international.org/11.0/#prod-ReservedWord
    pub static ref RESERVED_KEYWORDS: std::collections::BTreeSet<String>  =
        &*DISALLOWED_BINDING_IDENTIFIER_KEYWORDS |
        &string_set!["await", "yield"];
    // ReservedWord in strict mode. Includes all of reserved_keywords.
    // https://262.ecma-international.org/11.0/#sec-keywords-and-reserved-words
    pub static ref STRICT_RESERVED_KEYWORDS: std::collections::BTreeSet<String>  =
        &*STRICT_DISALLOWED_BINDING_IDENTIFIER_KEYWORDS |
        &string_set!["await", "yield"];

    // TODO(strager): Add 'await' and 'yield'.
    pub static ref CONTEXTUAL_KEYWORDS: std::collections::BTreeSet<String> = string_set![
        "abstract",  "any",      "as",       "assert",      "asserts",
        "async",     "bigint",   "boolean",  "constructor", "declare",
        "from",      "get",      "global",   "infer",       "intrinsic",
        "is",        "keyof",    "let",      "meta",        "module",
        "namespace", "never",    "number",   "object",      "of",
        "out",       "override", "readonly", "require",     "set",
        "static",    "string",   "symbol",   "target",      "type",
        "undefined", "unique",   "unknown",
    ];

    // ReservedWord or contextual keyword in strict mode or non-strict mode.
    pub static ref KEYWORDS: std::collections::BTreeSet<String>  =
        &*STRICT_RESERVED_KEYWORDS | &*CONTEXTUAL_KEYWORDS;

    pub static ref TYPESCRIPT_BUILTIN_TYPE_KEYWORDS: std::collections::BTreeSet<String> = string_set![
        "bigint", "boolean", "null",      "number", "object",
        "string", "symbol",  "undefined", "void",
    ];

    pub static ref TYPESCRIPT_SPECIAL_TYPE_KEYWORDS: std::collections::BTreeSet<String> = string_set![
        "any",
        "never",
        "unknown",
    ];
}
