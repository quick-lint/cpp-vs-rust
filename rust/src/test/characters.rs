// TODO(port): Avoid specifying generic arguments to 'concat'.

use crate::test::array::*;

const LINE_TERMINATORS_EXCEPT_LS_PS: [&'static str; 3] = ["\n", "\r", "\r\n"];

pub const LS_AND_PS: [&'static str; 2] = [
    "\u{2028}", // 0xe2 0x80 0xa8 Line Separator
    "\u{2029}", // 0xe2 0x80 0xa9 Paragraph Separator
];

pub const LINE_TERMINATORS: [&'static str; 5] =
    concat::<5, _, 3, 2>(&LINE_TERMINATORS_EXCEPT_LS_PS, &LS_AND_PS);

pub const CONTROL_CHARACTERS_EXCEPT_WHITESPACE: [&'static str; 28] = [
    "\u{0000}", // NUL Null character
    "\u{0001}", // SOH Start of Heading
    "\u{0002}", // STX Start of Text
    "\u{0003}", // ETX End-of-text character
    "\u{0004}", // EOT End-of-transmission character
    "\u{0005}", // ENQ Enquiry character
    "\u{0006}", // ACK Acknowledge character
    "\u{0007}", // BEL Bell character
    "\u{0008}", // BS Backspace
    "\u{000e}", // SO Shift Out
    "\u{000f}", // SI Shift In
    "\u{0010}", // DLE Data Link Escape
    "\u{0011}", // DC1 Device Control 1
    "\u{0012}", // DC2 Device Control 2
    "\u{0013}", // DC3 Device Control 3
    "\u{0014}", // DC4 Device Control 4
    "\u{0015}", // NAK Negative-acknowledge character
    "\u{0016}", // SYN Synchronous Idle
    "\u{0017}", // ETB End of Transmission Block
    "\u{0018}", // CAN Cancel character
    "\u{0019}", // EM End of Medium
    "\u{001a}", // SUB Substitute character
    "\u{001b}", // ESC Escape character
    "\u{001c}", // FS File Separator
    "\u{001d}", // GS Group Separator
    "\u{001e}", // RS Record Separator
    "\u{001f}", // US Unit Separator
    "\u{007f}", // DEL Delete
];

pub const CONTROL_CHARACTERS_EXCEPT_LINE_TERMINATORS: [&'static str; 31] = concat::<31, _, 28, 3>(
    &CONTROL_CHARACTERS_EXCEPT_WHITESPACE,
    &[
        "\u{0009}", // HT Horizontal tab
        "\u{000b}", // VT Vertical tab
        "\u{000c}", // FF Form feed
    ],
);
