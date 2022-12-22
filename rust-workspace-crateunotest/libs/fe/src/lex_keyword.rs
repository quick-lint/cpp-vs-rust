// Copyright (C) 2020  Matthew "strager" Glazar
// See end of file for extended copyright information.

// NOTE(port): The C++ version of this file was generated by gperf. This file was hand-converted
// because gperf cannot output Rust code. A more realistic port would use some proc macro magic
// instead to generate the perfect hash table and code (e.g. with the Rust-PHF crate).

use crate::token::*;

struct KeywordEntry {
    string_offset: i32,
    type_: TokenType,
}

fn hash(s: &[u8]) -> u32 {
    const ASSO_VALUES: [u8; 256] = [
        162, 162, 162, 162, 162, 162, 162, 162, 162, 162, 162, 162, 162, 162, 162, 162, 162, 162,
        162, 162, 162, 162, 162, 162, 162, 162, 162, 162, 162, 162, 162, 162, 162, 162, 162, 162,
        162, 162, 162, 162, 162, 162, 162, 162, 162, 162, 162, 162, 162, 162, 162, 162, 162, 162,
        162, 162, 162, 162, 162, 162, 162, 162, 162, 162, 162, 162, 162, 162, 162, 162, 162, 162,
        162, 162, 162, 162, 162, 162, 162, 162, 162, 162, 162, 162, 162, 162, 162, 162, 162, 162,
        162, 162, 162, 162, 162, 162, 162, 20, 85, 0, 60, 5, 15, 5, 30, 0, 162, 60, 75, 60, 5, 30,
        70, 162, 0, 0, 0, 35, 10, 25, 25, 75, 162, 162, 162, 162, 162, 162, 162, 162, 162, 162,
        162, 162, 162, 162, 162, 162, 162, 162, 162, 162, 162, 162, 162, 162, 162, 162, 162, 162,
        162, 162, 162, 162, 162, 162, 162, 162, 162, 162, 162, 162, 162, 162, 162, 162, 162, 162,
        162, 162, 162, 162, 162, 162, 162, 162, 162, 162, 162, 162, 162, 162, 162, 162, 162, 162,
        162, 162, 162, 162, 162, 162, 162, 162, 162, 162, 162, 162, 162, 162, 162, 162, 162, 162,
        162, 162, 162, 162, 162, 162, 162, 162, 162, 162, 162, 162, 162, 162, 162, 162, 162, 162,
        162, 162, 162, 162, 162, 162, 162, 162, 162, 162, 162, 162, 162, 162, 162, 162, 162, 162,
        162, 162, 162, 162, 162, 162, 162, 162, 162, 162, 162, 162, 162, 162, 162, 162,
    ];
    (s.len() as u32)
        + (ASSO_VALUES[s[1] as usize] as u32)
        + (ASSO_VALUES[s[0] as usize] as u32)
        + (ASSO_VALUES[s[s.len() - 1] as usize] as u32)
}

const STRINGPOOL: &[u8; 525] = b"\
    is\0static\0set\0true\0infer\0\
    string\0in\0get\0intrinsic\0never\0\
    return\0require\0interface\0as\0async\0\
    assert\0asserts\0case\0instanceof\0if\0\
    var\0this\0const\0export\0extends\0\
    new\0namespace\0super\0constructor\0continue\0\
    false\0number\0for\0await\0unique\0\
    unknown\0override\0catch\0with\0throw\0\
    switch\0of\0function\0while\0import\0\
    out\0implements\0default\0debugger\0enum\0\
    delete\0declare\0try\0from\0class\0\
    private\0let\0type\0keyof\0readonly\0\
    else\0bigint\0typeof\0finally\0module\0\
    package\0any\0void\0undefined\0public\0\
    abstract\0null\0object\0do\0boolean\0\
    protected\0yield\0break\0symbol\0global\0\
    ";

fn look_up(input: &[u8]) -> Option<&'static KeywordEntry> {
    const MIN_WORD_LENGTH: usize = 2;
    const MAX_WORD_LENGTH: usize = 11;
    const MAX_HASH_VALUE: u32 = 161;

    const INVALID: KeywordEntry = KeywordEntry {
        string_offset: -1,
        type_: TokenType::Identifier, // Arbitrary.
    };
    const fn word(string_offset: i32, type_: TokenType) -> KeywordEntry {
        KeywordEntry {
            string_offset: string_offset,
            type_: type_,
        }
    }
    const WORDLIST: [KeywordEntry; (MAX_HASH_VALUE + 1) as usize] = [
        INVALID,
        INVALID,
        word(0, TokenType::KWIs),
        INVALID,
        INVALID,
        INVALID,
        word(3, TokenType::KWStatic),
        INVALID,
        word(10, TokenType::KWSet),
        word(14, TokenType::KWTrue),
        word(19, TokenType::KWInfer),
        word(25, TokenType::KWString),
        word(32, TokenType::KWIn),
        word(35, TokenType::KWGet),
        word(39, TokenType::KWIntrinsic),
        word(49, TokenType::KWNever),
        word(55, TokenType::KWReturn),
        word(62, TokenType::KWRequire),
        INVALID,
        word(70, TokenType::KWInterface),
        INVALID,
        INVALID,
        word(80, TokenType::KWAs),
        INVALID,
        INVALID,
        word(83, TokenType::KWAsync),
        word(89, TokenType::KWAssert),
        word(96, TokenType::KWAsserts),
        INVALID,
        word(104, TokenType::KWCase),
        word(109, TokenType::KWInstanceof),
        INVALID,
        word(120, TokenType::KWIf),
        word(123, TokenType::KWVar),
        word(127, TokenType::KWThis),
        word(132, TokenType::KWConst),
        word(138, TokenType::KWExport),
        word(145, TokenType::KWExtends),
        word(153, TokenType::KWNew),
        word(157, TokenType::KWNamespace),
        word(167, TokenType::KWSuper),
        word(173, TokenType::KWConstructor),
        INVALID,
        word(185, TokenType::KWContinue),
        INVALID,
        word(194, TokenType::KWFalse),
        word(200, TokenType::KWNumber),
        INVALID,
        word(207, TokenType::KWFor),
        INVALID,
        word(211, TokenType::KWAwait),
        word(217, TokenType::KWUnique),
        word(224, TokenType::KWUnknown),
        word(232, TokenType::KWOverride),
        INVALID,
        word(241, TokenType::KWCatch),
        INVALID,
        INVALID,
        INVALID,
        word(247, TokenType::KWWith),
        word(252, TokenType::KWThrow),
        word(258, TokenType::KWSwitch),
        word(265, TokenType::KWOf),
        word(268, TokenType::KWFunction),
        INVALID,
        word(277, TokenType::KWWhile),
        word(283, TokenType::KWImport),
        INVALID,
        word(290, TokenType::KWOut),
        INVALID,
        word(294, TokenType::KWImplements),
        INVALID,
        word(305, TokenType::KWDefault),
        word(313, TokenType::KWDebugger),
        word(322, TokenType::KWEnum),
        INVALID,
        word(327, TokenType::KWDelete),
        word(334, TokenType::KWDeclare),
        word(342, TokenType::KWTry),
        word(346, TokenType::KWFrom),
        word(351, TokenType::KWClass),
        INVALID,
        word(357, TokenType::KWPrivate),
        word(365, TokenType::KWLet),
        word(369, TokenType::KWType),
        word(374, TokenType::KWKeyof),
        INVALID,
        INVALID,
        word(380, TokenType::KWReadonly),
        word(389, TokenType::KWElse),
        INVALID,
        word(394, TokenType::KWBigint),
        INVALID,
        INVALID,
        INVALID,
        INVALID,
        word(401, TokenType::KWTypeof),
        word(408, TokenType::KWFinally),
        INVALID,
        INVALID,
        INVALID,
        word(416, TokenType::KWModule),
        word(423, TokenType::KWPackage),
        word(431, TokenType::KWAny),
        word(435, TokenType::KWVoid),
        INVALID,
        INVALID,
        INVALID,
        INVALID,
        word(440, TokenType::KWUndefined),
        INVALID,
        word(450, TokenType::KWPublic),
        INVALID,
        word(457, TokenType::KWAbstract),
        INVALID,
        INVALID,
        INVALID,
        INVALID,
        INVALID,
        word(466, TokenType::KWNull),
        INVALID,
        word(471, TokenType::KWObject),
        word(478, TokenType::KWDo),
        INVALID,
        INVALID,
        INVALID,
        INVALID,
        word(481, TokenType::KWBoolean),
        INVALID,
        INVALID,
        INVALID,
        INVALID,
        INVALID,
        INVALID,
        INVALID,
        INVALID,
        INVALID,
        INVALID,
        INVALID,
        word(489, TokenType::KWProtected),
        word(499, TokenType::KWYield),
        INVALID,
        INVALID,
        INVALID,
        INVALID,
        INVALID,
        INVALID,
        INVALID,
        INVALID,
        INVALID,
        word(505, TokenType::KWBreak),
        INVALID,
        INVALID,
        INVALID,
        INVALID,
        INVALID,
        word(511, TokenType::KWSymbol),
        INVALID,
        INVALID,
        INVALID,
        INVALID,
        word(518, TokenType::KWGlobal),
    ];

    let len: usize = input.len();
    if (MIN_WORD_LENGTH..=MAX_WORD_LENGTH).contains(&len) {
        let key: u32 = hash(input);

        if key <= MAX_HASH_VALUE {
            unsafe {
                let entry: &KeywordEntry = &WORDLIST.get_unchecked(key as usize);
                let o: i32 = entry.string_offset;
                if o >= 0 {
                    let s: &[u8] = &STRINGPOOL.get_unchecked((o as usize)..);

                    if input.get_unchecked(0) == s.get_unchecked(0)
                        && input.get_unchecked(1..len - 1) == s.get_unchecked(1..len - 1)
                        && *s.get_unchecked(len) == b'\0'
                    {
                        return Some(entry);
                    }
                }
            }
        }
    }
    None
}

pub fn identifier_token_type(identifier: &[u8]) -> TokenType {
    match look_up(identifier) {
        Some(entry) => entry.type_,
        None => TokenType::Identifier,
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
