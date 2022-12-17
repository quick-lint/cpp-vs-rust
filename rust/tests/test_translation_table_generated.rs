// Code generated by tools/compile-translations.go. DO NOT EDIT.
// source: po/*.po

// Copyright (C) 2020  Matthew "strager" Glazar
// See end of file for extended copyright information.

#![allow(clippy::redundant_static_lifetimes)]

use cpp_vs_rust::i18n::translation::*;
use cpp_vs_rust::qljs_translatable;
use cpp_vs_rust::scoped_trace;

#[rustfmt::skip]
pub const TEST_LOCALE_NAMES: [&'static str; 6] = [
    "",
    "de",
    "en_US@snarky",
    "fr_FR",
    "pt_BR",
    "sv_SE",
];

pub struct TranslatedString {
    pub translatable: TranslatableMessage,
    pub expected_per_locale: [&'static str; 6],
}

pub const TEST_TRANSLATION_TABLE: [TranslatedString; 63] = [
    TranslatedString{
        translatable: qljs_translatable!("'>' is not allowed directly in JSX text; write {{'>'} or &gt; instead"),
        expected_per_locale: [
            "'>' is not allowed directly in JSX text; write {{'>'} or &gt; instead",
            "'>' darf nicht direkt in JSX-Text verwendet werden. Anstattdessen {{'>} oder &gt; schreiben.",
            "Facebook says '>' is not allowed; write {{'>'} or &gt; instead",
            "'>' is not allowed directly in JSX text; write {{'>'} or &gt; instead",
            "'>' n\u{00e3}o \u{00e9} permitido diretamente em um texto JSX; use {{'>'} ou &gt;",
            "'>' is not allowed directly in JSX text; write {{'>'} or &gt; instead",
        ],
    },
    TranslatedString{
        translatable: qljs_translatable!("'do-while' loop"),
        expected_per_locale: [
            "'do-while' loop",
            "do-while-Schleife",
            "do-whiley do",
            "'do-while' loop",
            "loop 'do-while'",
            "'do-while' loop",
        ],
    },
    TranslatedString{
        translatable: qljs_translatable!("'for' loop"),
        expected_per_locale: [
            "'for' loop",
            "for-Schleife",
            "'for' loop \u{1f503}",
            "'for' loop",
            "loop 'for'",
            "'for' loop",
        ],
    },
    TranslatedString{
        translatable: qljs_translatable!("'if' statement"),
        expected_per_locale: [
            "'if' statement",
            "if-Anweisung",
            "when (not if) statement",
            "'if' statement",
            "instru\u{00e7}\u{00e3}o 'if'",
            "'if' statement",
        ],
    },
    TranslatedString{
        translatable: qljs_translatable!("'while' loop"),
        expected_per_locale: [
            "'while' loop",
            "while-Schleife",
            "whenever loop",
            "'while' loop",
            "loop 'while'",
            "'while' loop",
        ],
    },
    TranslatedString{
        translatable: qljs_translatable!("'with' statement"),
        expected_per_locale: [
            "'with' statement",
            "with-Anweisung",
            "what-the-heck-is-wrong-with statement",
            "'with' statement",
            "instru\u{00e7}\u{00e3}o 'with'",
            "'with' statement",
        ],
    },
    TranslatedString{
        translatable: qljs_translatable!("'{0}' is not allowed for strings; use {1} instead"),
        expected_per_locale: [
            "'{0}' is not allowed for strings; use {1} instead",
            "'{0}' ist f\u{00fc}r Strings nicht erlaubt. '{1}' anstattdessen verwenden.",
            "smart quotes \u{1f9e0} require the SmartyPantsJS DLC",
            "'{0}' is not allowed for strings; use {1} instead",
            "'{0}' n\u{00e3}o \u{00e9} permitido para strings; use {1}",
            "'{0}' is not allowed for strings; use {1} instead",
        ],
    },
    TranslatedString{
        translatable: qljs_translatable!("'}' is not allowed directly in JSX text; write {{'}'} instead"),
        expected_per_locale: [
            "'}' is not allowed directly in JSX text; write {{'}'} instead",
            "'}' darf nicht direkt in JSX-Text verwendet werden. Anstattdessen {{'}'} schreiben",
            "Facebook says '}' is not allowed; write {{'}'} instead",
            "'}' is not allowed directly in JSX text; write {{'}'} instead",
            "'}' n\u{00e3}o \u{00e9} permitido diretamente em um texto JSX; use {{'}'}",
            "'}' is not allowed directly in JSX text; write {{'}'} instead",
        ],
    },
    TranslatedString{
        translatable: qljs_translatable!("BigInt literal contains decimal point"),
        expected_per_locale: [
            "BigInt literal contains decimal point",
            "BigInt-Literal mit Dezimalpunkt",
            "it's Big*Int*, not Big*Decimal*",
            "le lit\u{00e9}ral BigInt contient un s\u{00e9}parateur de d\u{00e9}cimales",
            "valor BigInt cont\u{00e9}m casa decimal",
            "BigInt heltallitter\u{00e4}r inneh\u{00e5}ller decimaler",
        ],
    },
    TranslatedString{
        translatable: qljs_translatable!("BigInt literal contains exponent"),
        expected_per_locale: [
            "BigInt literal contains exponent",
            "BigInt-Literal mit Exponenten",
            "BigExponInt is an ES2069 feature",
            "le lit\u{00e9}ral BigInt contient un exposant",
            "valor BigInt cont\u{00e9}m expoente",
            "BigInt heltallitter\u{00e4}r inneh\u{00e5}ller exponent",
        ],
    },
    TranslatedString{
        translatable: qljs_translatable!("RegExp literal flags cannot contain Unicode escapes"),
        expected_per_locale: [
            "RegExp literal flags cannot contain Unicode escapes",
            "RegExp-Literale d\u{00fc}rfen keine Unicode Escapes enthalten",
            "keep your RegExp flags simple, please",
            "un litt\u{00e9}ral RegExp ne peut contenir des \u{00e9}chappements Unicode",
            "flags do RegExp n\u{00e3}o podem conter sequ\u{00ea}ncias de escape Unicode",
            "RegExp literal flags cannot contain Unicode escapes",
        ],
    },
    TranslatedString{
        translatable: qljs_translatable!("a 'do-while' loop"),
        expected_per_locale: [
            "a 'do-while' loop",
            "eine do-While-Schleife",
            "a do-whiley do",
            "a 'do-while' loop",
            "um loop 'do-while'",
            "a 'do-while' loop",
        ],
    },
    TranslatedString{
        translatable: qljs_translatable!("a 'for' loop"),
        expected_per_locale: [
            "a 'for' loop",
            "eine for-Schleife",
            "a 'for' loop \u{1f503}",
            "a 'for' loop",
            "um loop 'for'",
            "a 'for' loop",
        ],
    },
    TranslatedString{
        translatable: qljs_translatable!("a 'while' loop"),
        expected_per_locale: [
            "a 'while' loop",
            "eine while-Schleife",
            "a whenever loop",
            "a 'while' loop",
            "um loop 'while'",
            "a 'while' loop",
        ],
    },
    TranslatedString{
        translatable: qljs_translatable!("a 'with' statement"),
        expected_per_locale: [
            "a 'with' statement",
            "eine with-Anweisung",
            "a what-the-heck-is-wrong-with statement",
            "a 'with' statement",
            "uma instru\u{00e7}\u{00e3}o 'with'",
            "a 'with' statement",
        ],
    },
    TranslatedString{
        translatable: qljs_translatable!("a labelled statement"),
        expected_per_locale: [
            "a labelled statement",
            "a labelled statement",
            "a labelled statement",
            "a labelled statement",
            "uma instru\u{00e7}\u{00e3}o com label",
            "a labelled statement",
        ],
    },
    TranslatedString{
        translatable: qljs_translatable!("a {{0} b }} c"),
        expected_per_locale: [
            "a {{0} b }} c",
            "a {{0} b }} c",
            "a {{0} b }} c",
            "a {{0} b }} c",
            "a {{0} b }} c",
            "a {{0} b }} c",
        ],
    },
    TranslatedString{
        translatable: qljs_translatable!("an 'if' statement"),
        expected_per_locale: [
            "an 'if' statement",
            "eine if-Anweisung",
            "a when (not if) statement",
            "an 'if' statement",
            "uma instru\u{00e7}\u{00e3}o 'if'",
            "an 'if' statement",
        ],
    },
    TranslatedString{
        translatable: qljs_translatable!("binary number literal has no digits"),
        expected_per_locale: [
            "binary number literal has no digits",
            "Bin\u{00e4}res Zahlenliteral ohne Ziffern",
            "binary number lost its genitals",
            "le litt\u{00e9}ral num\u{00e9}rique binaire n'a pas de chiffres",
            "n\u{00fa}mero bin\u{00e1}rio n\u{00e3}o tem d\u{00ed}gitos",
            "bin\u{00e4}ra nummerlitteraler has inga siffror",
        ],
    },
    TranslatedString{
        translatable: qljs_translatable!("character is not allowed in identifiers"),
        expected_per_locale: [
            "character is not allowed in identifiers",
            "Ung\u{00fc}ltiges Zeichen in Bezeichner",
            "hold up! \u{270b} no '{0}' allowed",
            "caract\u{00e8}re non autoris\u{00e9} dans les identifiants",
            "caracter n\u{00e3}o \u{00e9} permitido em identificadores",
            "tecknet \u{00e4}r inte till\u{00e5}tet i indentifierare",
        ],
    },
    TranslatedString{
        translatable: qljs_translatable!("code point in Unicode escape sequence must not be greater than U+10FFFF"),
        expected_per_locale: [
            "code point in Unicode escape sequence must not be greater than U+10FFFF",
            "Codepunkt innerhalb der Unicode-Escapesequenz darf nicht gr\u{00f6}\u{00df}er als U+10FFFF sein",
            "U+10FFFF is the limit. what are you trying to accomplish?",
            "un point de code dans une s\u{00e9}quence d'\u{00e9}chappement Unicode ne peut d\u{00e9}passer la valeur U+10FFFF",
            "code point em sequ\u{00ea}ncias de escape Unicode n\u{00e3}o pode ser maior que U+10FFFF",
            "code point in Unicode escape sequence must not be greater than U+10FFFF",
        ],
    },
    TranslatedString{
        translatable: qljs_translatable!("code point out of range"),
        expected_per_locale: [
            "code point out of range",
            "Codepunkt au\u{00df}erhalb des zul\u{00e4}ssigen Bereichs",
            "it won't fit \u{1f930}",
            "point de code hors limite",
            "code point fora do intervalo permitido",
            "kod punkt ur span",
        ],
    },
    TranslatedString{
        translatable: qljs_translatable!("escaped character is not allowed in identifiers"),
        expected_per_locale: [
            "escaped character is not allowed in identifiers",
            "Escape-Zeichen darf nicht nicht in Bezeichnern verwendet werden",
            "fugitive \u{1f9b9}\u{200d}\u{2642}\u{fe0f} is not allowed in identifiers",
            "caract\u{00e8}re \u{00e9}chapp\u{00e9} non permis dans les identifiants",
            "caracter escapado n\u{00e3}o \u{00e9} permiido em identificadores",
            "flykttecken \u{00e4}r inte till\u{00e5}tet i indentifierare",
        ],
    },
    TranslatedString{
        translatable: qljs_translatable!("escaping '-' is not allowed in tag names; write '-' instead"),
        expected_per_locale: [
            "escaping '-' is not allowed in tag names; write '-' instead",
            "Escape von '-' ist in Tagnamen nicht erlaubt. '-' anstattdessen schreiben",
            "stop being so fancy; just write '-'",
            "escaping '-' is not allowed in tag names; write '-' instead",
            "escapar '-' n\u{00e3}o \u{00e9} permitido em nomes de tags; use '-'",
            "escaping '-' is not allowed in tag names; write '-' instead",
        ],
    },
    TranslatedString{
        translatable: qljs_translatable!("expected hexadecimal digits in Unicode escape sequence"),
        expected_per_locale: [
            "expected hexadecimal digits in Unicode escape sequence",
            "Hexadezimale Ziffern in Unicode-Escapesequenz erwartet",
            "what are you trying to do? This is a Unicode escape sequence, not a Wendy's \u{1f354}",
            "nombres hexadecimaux attendus dans une s\u{00e9}quence d'\u{00e9}chappement Unicode",
            "d\u{00ed}gitos hexadecimais s\u{00e3}o esperados em uma sequ\u{00ea}ncia de escape Unicode",
            "f\u{00f6}rv\u{00e4}ntade hexadecimala siffror i Unicode flyktsekvens",
        ],
    },
    TranslatedString{
        translatable: qljs_translatable!("expected {1:headlinese}"),
        expected_per_locale: [
            "expected {1:headlinese}",
            "{1:headlinese} erwartet",
            "expected {1:headlinese}",
            "expected {1:headlinese}",
            "esperado {1:headlinese}",
            "expected {1:headlinese}",
        ],
    },
    TranslatedString{
        translatable: qljs_translatable!("expected {1:singular}"),
        expected_per_locale: [
            "expected {1:singular}",
            "{1:singular} erwartet",
            "expected {1:singular}",
            "expected {1:singular}",
            "esperado {1:singular}",
            "expected {1:singular}",
        ],
    },
    TranslatedString{
        translatable: qljs_translatable!("free {1} and {0} {1} {2}"),
        expected_per_locale: [
            "free {1} and {0} {1} {2}",
            "freies {1} und {0} {1} {2}",
            "free {1} and {0} {1} {2}",
            "free {1} and {0} {1} {2}",
            "free {1} and {0} {1} {2}",
            "free {1} and {0} {1} {2}",
        ],
    },
    TranslatedString{
        translatable: qljs_translatable!("hex number literal has no digits"),
        expected_per_locale: [
            "hex number literal has no digits",
            "Hexadezimales Zahlenliteral ohne Ziffern",
            "hex number literal has no digits",
            "le litt\u{00e9}ral num\u{00e9}rique hex n'a pas de chiffres",
            "n\u{00fa}mero hexadecimal n\u{00e3}o tem d\u{00ed}gitos",
            "hex nummerlitteral har inga siffror",
        ],
    },
    TranslatedString{
        translatable: qljs_translatable!("integer cannot be represented and will be rounded to '{1}'"),
        expected_per_locale: [
            "integer cannot be represented and will be rounded to '{1}'",
            "integer cannot be represented and will be rounded to '{1}'",
            "this number's too thicc for JavaScript; '{1}' would be used instead",
            "integer cannot be represented and will be rounded to '{1}'",
            "inteiro n\u{00e3}o pode ser representado e vai ser arredondado para '{1}'",
            "integer cannot be represented and will be rounded to '{1}'",
        ],
    },
    TranslatedString{
        translatable: qljs_translatable!("invalid UTF-8 sequence"),
        expected_per_locale: [
            "invalid UTF-8 sequence",
            "Ung\u{00fc}ltige UTF-8 Sequenz",
            "quick-lint-js only works with nonbinary files",
            "s\u{00e9}quence UTF-8 invalide",
            "sequ\u{00ea}ncia UTF-8 inv\u{00e1}lida",
            "ogiltig UTF-8 sekvens",
        ],
    },
    TranslatedString{
        translatable: qljs_translatable!("invalid hex escape sequence: {0}"),
        expected_per_locale: [
            "invalid hex escape sequence: {0}",
            "Ung\u{00fc}ltige Hex-Escapesequenz: {0}",
            "this ain't hex",
            "s\u{00e9}quence d'\u{00e9}chappement hex invalide: {0}",
            "sequ\u{00ea}ncia de escape hex inv\u{00e1}lida: {0}",
            "ogiltig kring\u{00e5}ende hex sekvens: {0}",
        ],
    },
    TranslatedString{
        translatable: qljs_translatable!("keywords cannot contain escape sequences"),
        expected_per_locale: [
            "keywords cannot contain escape sequences",
            "Schl\u{00fc}sselworte d\u{00fc}rfen keine Escapesequenzen beinhalten",
            "that sequence should escape from this keyword cuz it's not allowed here",
            "les mots-cl\u{00e9}s ne peuvent pas contenir de s\u{00e9}quence d'\u{00e9}chappement",
            "palavras-chave n\u{00e3}o podem conter sequ\u{00ea}ncias de escape",
            "nyckelord kan inte inneh\u{00e5}lla en flyktsekvens",
        ],
    },
    TranslatedString{
        translatable: qljs_translatable!("labelled statement"),
        expected_per_locale: [
            "labelled statement",
            "labelled statement",
            "labelled statement",
            "labelled statement",
            "instru\u{00e7}\u{00e3}o com label",
            "labelled statement",
        ],
    },
    TranslatedString{
        translatable: qljs_translatable!("legacy octal literal may not be BigInt"),
        expected_per_locale: [
            "legacy octal literal may not be BigInt",
            "Veraltete Oktalliterale sind in BigInts nicht erlaubt",
            "0Ops",
            "un litt\u{00e9}ral octal classique ne peut pas \u{00ea}tre de type BigInt",
            "n\u{00fa}mero octal legado n\u{00e3}o pode ser BigInt",
            "\u{00e4}rftligt octal nummerlitteral kan inte vara BigInt",
        ],
    },
    TranslatedString{
        translatable: qljs_translatable!("legacy octal literals may not contain underscores"),
        expected_per_locale: [
            "legacy octal literals may not contain underscores",
            "Veraltete Oktalliterale d\u{00fc}rfen keine Unterstriche enthalten",
            "legacy_octal_literals_may_not_contain_underscores",
            "un litt\u{00e9}ral octal classique ne peut pas contenir de tiret de soulignement",
            "n\u{00fa}mero octal legado n\u{00e3}o pode conter underscore",
            "\u{00e4}rftligt octal nummerlitteral kan inte inneh\u{00e5}lla understr\u{00e4}ck",
        ],
    },
    TranslatedString{
        translatable: qljs_translatable!("number literal contains consecutive underscores"),
        expected_per_locale: [
            "number literal contains consecutive underscores",
            "Zahlenliteral darf keine aufeinanderfolgenden Unterstriche enthalten",
            "too__many__underscores",
            "le litt\u{00e9}ral num\u{00e9}rique contient plusieurs tirets de soulignement cons\u{00e9}cutifs",
            "n\u{00fa}mero cont\u{00e9}m underscores consecutivos",
            "numerlitter\u{00e4}r inneh\u{00e5}ller upprepande understr\u{00e4}ck",
        ],
    },
    TranslatedString{
        translatable: qljs_translatable!("number literal contains trailing underscore(s)"),
        expected_per_locale: [
            "number literal contains trailing underscore(s)",
            "Zahlenliteral endet mit Unterstrich(en)",
            "too_many_underscores_____",
            "le litt\u{00e9}ral num\u{00e9}rique est suivi d'un tiret de soulignement",
            "n\u{00fa}mero cont\u{00e9}m underscore(s) no final",
            "nummerlitter\u{00e4}r inneh\u{00e5}ller efterf\u{00f6}ljande understr\u{00e4}ck",
        ],
    },
    TranslatedString{
        translatable: qljs_translatable!("octal literal may not have decimal"),
        expected_per_locale: [
            "octal literal may not have decimal",
            "Oktalliterale mit Dezimalpunkt sind nicht erlaubt",
            "but you said '0o'...",
            "un litt\u{00e9}ral octal ne peut avoir de partie d\u{00e9}cimale",
            "n\u{00fa}mero octal n\u{00e3}o pode ter casa decimal",
            "oktal nummerlitter\u{00e4}l kan inte ha decimaler",
        ],
    },
    TranslatedString{
        translatable: qljs_translatable!("octal literal may not have exponent"),
        expected_per_locale: [
            "octal literal may not have exponent",
            "Oktalliterale mit Exponenten sind nicht erlaubt",
            "scientists don't use octal",
            "un litt\u{00e9}ral octal ne peut avoir d'exposant",
            "n\u{00fa}mero octal n\u{00e3}o pode ter expoente",
            "oktal nummerlitter\u{00e4}l kan inte ha exponent",
        ],
    },
    TranslatedString{
        translatable: qljs_translatable!("octal number literal has no digits"),
        expected_per_locale: [
            "octal number literal has no digits",
            "Oktales Zahlenliteral ohne Ziffern",
            "<octupus-with-no-legs> has no digits",
            "le litt\u{00e9}ral num\u{00e9}rique octal n'a pas de chiffres",
            "n\u{00fa}mero octal n\u{00e3}o tem d\u{00ed}gitos",
            "oktal nummerlitteral har inga siffror",
        ],
    },
    TranslatedString{
        translatable: qljs_translatable!("second message here"),
        expected_per_locale: [
            "second message here",
            "second message here",
            "second message here",
            "second message here",
            "second message here",
            "second message here",
        ],
    },
    TranslatedString{
        translatable: qljs_translatable!("see here"),
        expected_per_locale: [
            "see here",
            "siehe hier",
            "see here",
            "see here",
            "veja aqui",
            "see here",
        ],
    },
    TranslatedString{
        translatable: qljs_translatable!("something happened"),
        expected_per_locale: [
            "something happened",
            "etwas geschah",
            "I wish you never happened",
            "something happened",
            "algo aconteceu",
            "something happened",
        ],
    },
    TranslatedString{
        translatable: qljs_translatable!("test for multiple messages"),
        expected_per_locale: [
            "test for multiple messages",
            "test for multiple messages",
            "test for multiple messages",
            "test for multiple messages",
            "test for multiple messages",
            "test for multiple messages",
        ],
    },
    TranslatedString{
        translatable: qljs_translatable!("this {0} looks fishy"),
        expected_per_locale: [
            "this {0} looks fishy",
            "dieses {0} sieht merkw\u{00fc}rdig aus",
            "this {0} looks fishy",
            "this {0} looks fishy",
            "isso {0} parece suspeito",
            "this {0} looks fishy",
        ],
    },
    TranslatedString{
        translatable: qljs_translatable!("this {1} looks fishy"),
        expected_per_locale: [
            "this {1} looks fishy",
            "dieses {1} sieht merkw\u{00fc}rdig aus",
            "this {1} looks fishy",
            "this {1} looks fishy",
            "isso {1} parece suspeito",
            "this {1} looks fishy",
        ],
    },
    TranslatedString{
        translatable: qljs_translatable!("unclosed block comment"),
        expected_per_locale: [
            "unclosed block comment",
            "Blockkommentar ohne Ende",
            "you accidentally commented out your whole program",
            "commentaire de bloc non ferm\u{00e9}",
            "bloco de coment\u{00e1}rio n\u{00e3}o encerrado",
            "oavslutad kommentationsstycke",
        ],
    },
    TranslatedString{
        translatable: qljs_translatable!("unclosed identifier escape sequence"),
        expected_per_locale: [
            "unclosed identifier escape sequence",
            "Unbeendete Bezeichner-Escapesequenz",
            "runaway \\u!",
            "s\u{00e9}quence d'\u{00e9}chappement d'identifiant non ferm\u{00e9}e",
            "sequ\u{00ea}ncia de escape n\u{00e3}o foi fechada",
            "oavslutad identifierare flyktsekvens",
        ],
    },
    TranslatedString{
        translatable: qljs_translatable!("unclosed regexp literal"),
        expected_per_locale: [
            "unclosed regexp literal",
            "Unbeendetes RegExp-Literal",
            "/unclosed regexp literal",
            "litt\u{00e9}ral regexp non ferm\u{00e9}",
            "regexp n\u{00e3}o encerrado",
            "oavslutad regexplitteral",
        ],
    },
    TranslatedString{
        translatable: qljs_translatable!("unclosed string literal"),
        expected_per_locale: [
            "unclosed string literal",
            "Zeichenkette ohne Ende",
            "\"unclosed string literal",
            "litt\u{00e9}ral string non ferm\u{00e9}",
            "string n\u{00e3}o encerrada",
            "oavslutad str\u{00e4}nglitteral",
        ],
    },
    TranslatedString{
        translatable: qljs_translatable!("unclosed template"),
        expected_per_locale: [
            "unclosed template",
            "Template ohne Ende",
            "`unclosed template",
            "template non ferm\u{00e9}",
            "template n\u{00e3}o foi fechado",
            "oavslutad mall",
        ],
    },
    TranslatedString{
        translatable: qljs_translatable!("unexpected '#'"),
        expected_per_locale: [
            "unexpected '#'",
            "Unerwartete '#'",
            "#unexpected",
            "'#' inattendu",
            "'#' inesperado",
            "of\u{00f6}rv\u{00e4}ntad '#'",
        ],
    },
    TranslatedString{
        translatable: qljs_translatable!("unexpected '@'"),
        expected_per_locale: [
            "unexpected '@'",
            "Unerwartetes '@'",
            "unexp@cted",
            "'@' inattendu",
            "'@' inesperado",
            "of\u{00f6}rv\u{00e4}ntad '@'",
        ],
    },
    TranslatedString{
        translatable: qljs_translatable!("unexpected '\\' in identifier"),
        expected_per_locale: [
            "unexpected '\\' in identifier",
            "Unerwartetes '\\' in Bezeichner",
            "unex\\pected",
            "'\\' inattendu dans un identifiant",
            "'\\' inesperado em um identificador",
            "of\u{00f6}rv\u{00e4}ntad '\\' i identifierare",
        ],
    },
    TranslatedString{
        translatable: qljs_translatable!("unexpected characters in binary literal"),
        expected_per_locale: [
            "unexpected characters in binary literal",
            "Unerwartete Zeichen in bin\u{00e4}rem Zahlenliteral",
            "this number does not identify as binary",
            "caract\u{00e8}res inattendus dans un litt\u{00e9}ral binaire",
            "caracteres inesperados em um n\u{00fa}mero bin\u{00e1}rio",
            "of\u{00f6}rv\u{00e4}ntat tecken i bin\u{00e4}rlitteral",
        ],
    },
    TranslatedString{
        translatable: qljs_translatable!("unexpected characters in hex literal"),
        expected_per_locale: [
            "unexpected characters in hex literal",
            "Unerwartete Zeichen in hexadezimalem Zahlenliteral",
            "unexpected characters in hex literal",
            "caract\u{00e8}res inattendus dans un litt\u{00e9}ral hex",
            "caracteres inesperados em um n\u{00fa}mero hexadecimal",
            "of\u{00f6}rv\u{00e4}ntat tecken i hexlitteral",
        ],
    },
    TranslatedString{
        translatable: qljs_translatable!("unexpected characters in number literal"),
        expected_per_locale: [
            "unexpected characters in number literal",
            "Unerwartete Zeichen in Zahlenliteral",
            "does not compute \u{1f916}",
            "caract\u{00e8}res inattendus dans un litt\u{00e9}ral num\u{00e9}rique",
            "caracteres inesperados em um n\u{00fa}mero",
            "of\u{00f6}rv\u{00e4}ntat tecken i nummerlitteral",
        ],
    },
    TranslatedString{
        translatable: qljs_translatable!("unexpected characters in octal literal"),
        expected_per_locale: [
            "unexpected characters in octal literal",
            "Unerwartete Zeichen in oktalem Zahlenliteral",
            "Cthulhu \u{1f419} is not happy",
            "caract\u{00e8}res inattendus dans un litt\u{00e9}ral octal",
            "caracteres inesperados em um n\u{00fa}mero octal",
            "of\u{00f6}rv\u{00e4}ntat tecken i oktallitteral",
        ],
    },
    TranslatedString{
        translatable: qljs_translatable!("unexpected control character"),
        expected_per_locale: [
            "unexpected control character",
            "Unerwartetes Steuerzeichen",
            "you lost control of your code",
            "caract\u{00e8}re de contr\u{00f4}le inattendu",
            "caracter de control inesperado",
            "of\u{00f6}rv\u{00e4}ntat kontrolltecken",
        ],
    },
    TranslatedString{
        translatable: qljs_translatable!("unicode byte order mark (BOM) cannot appear before #! at beginning of script"),
        expected_per_locale: [
            "unicode byte order mark (BOM) cannot appear before #! at beginning of script",
            "Die Unicode Bytereihenfolge-Markierung (BOM) darf nicht vor #! zu Beginn eines Skripts erscheinen",
            "your editor BOMd \u{1f4a3} your s#!t \u{1f4a9}",
            "un indicateur d'ordre des octets (BOM) ne peut figurer avant #! au d\u{00e9}but d'un script",
            "unicode byte order mark (BOM) n\u{00e3}o pode aparecer antes do #! no come\u{00e7}o do script",
            "unicode byte ordningsm\u{00e4}rke (BOM) kan inte f\u{00f6}rekomma f\u{00f6}re #! i b\u{00f6}rjan av skript",
        ],
    },
    TranslatedString{
        translatable: qljs_translatable!("unopened block comment"),
        expected_per_locale: [
            "unopened block comment",
            "Blockkommentar ohne Beginn",
            "/*",
            "commentaire de bloc non ouvert",
            "bloco de coment\u{00e1}rio n\u{00e3}o foi aberto",
            "unopened block comment",
        ],
    },
    TranslatedString{
        translatable: qljs_translatable!("what is this '{1}' nonsense?"),
        expected_per_locale: [
            "what is this '{1}' nonsense?",
            "Was soll dieser '{1}' Humbug?",
            "what is this '{1}' nonsense?",
            "what is this '{1}' nonsense?",
            "what is this '{1}' nonsense?",
            "what is this '{1}' nonsense?",
        ],
    },
];

#[test]
fn full_translation_table() {
    for (locale_index, locale_name) in TEST_LOCALE_NAMES.iter().enumerate() {
        let mut messages: Translator = Translator::new_using_messages_from_source_code();
        scoped_trace!(locale_name);
        if locale_name.is_empty() {
            messages.use_messages_from_source_code();
        } else {
            assert!(
                messages.use_messages_from_locale(locale_name),
                "locale_name={:?}",
                locale_name,
            );
        }

        for test_case in TEST_TRANSLATION_TABLE {
            assert!(test_case.translatable.valid());
            assert_eq!(
                messages.translate(test_case.translatable),
                test_case.expected_per_locale[locale_index],
                "locale_name={:?}",
                locale_name,
            );
        }
    }
}

// quick-lint-js finds bugs in JavaScript programs.
// Copyright (C) 2020  Matthew "strager" Glazar
//
// This file is part of quick-lint-js.
//
// quick-lint-js is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// quick-lint-js is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with quick-lint-js.  If not, see <https://www.gnu.org/licenses/>.
