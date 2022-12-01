use crate::fe::diagnostic_types::*;
use crate::i18n::translation::*;
use crate::qljs_assert;

#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DiagnosticSeverity {
    Error,
    Note,
    Warning,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DiagnosticArgType {
    Invalid = 0,

    // TODO(port): Rename some of these.
    Char8,
    EnumKind,
    Identifier,
    SourceCodeSpan,
    StatementKind,
    String8View,
    VariableKind,
}

// If we support more than two infos (i.e. more than one note), the VS Code
// plugin needs to be updated. See NOTE(multiple notes).
pub const DIAGNOSTIC_MAX_MESSAGE_COUNT: usize = 2;

const DIAGNOSTIC_MAX_ARG_COUNT: usize = 3;

// DIAGNOSTIC_MESSAGE_ARG_OFFSET_SHIFT is how many bits are removed in compact_offset.
//
// For example, if DIAGNOSTIC_MESSAGE_ARG_OFFSET_SHIFT is 3, then an arg must be 8-byte aligned.
const DIAGNOSTIC_MESSAGE_ARG_OFFSET_SHIFT: u8 = 1;
const DIAGNOSTIC_MESSAGE_ARG_OFFSET_BITS: u8 = 5;

#[repr(C)]
pub struct DiagnosticMessageArgInfo {
    // C++ equivalent:
    //
    // std::uint8_t compact_offset : offset_bits;
    // diagnostic_arg_type type : (8 - offset_bits);
    data: u8,
}

impl DiagnosticMessageArgInfo {
    pub const fn empty() -> DiagnosticMessageArgInfo {
        DiagnosticMessageArgInfo::new(0, DiagnosticArgType::Invalid)
    }

    pub const fn new(offset: usize, type_: DiagnosticArgType) -> DiagnosticMessageArgInfo {
        let offset = offset as u8;
        assert!(
            (offset >> DIAGNOSTIC_MESSAGE_ARG_OFFSET_SHIFT)
                < (1 << DIAGNOSTIC_MESSAGE_ARG_OFFSET_BITS),
            "offset should be small",
        );
        assert!(
            (offset & ((1 << DIAGNOSTIC_MESSAGE_ARG_OFFSET_SHIFT) - 1)) == 0,
            "offset should be aligned",
        );
        DiagnosticMessageArgInfo {
            data: (offset >> DIAGNOSTIC_MESSAGE_ARG_OFFSET_SHIFT)
                | ((type_ as u8) << DIAGNOSTIC_MESSAGE_ARG_OFFSET_BITS),
        }
    }

    pub const fn offset(&self) -> usize {
        const MASK: u8 = (1 << DIAGNOSTIC_MESSAGE_ARG_OFFSET_BITS) - 1;
        ((self.data & MASK) << DIAGNOSTIC_MESSAGE_ARG_OFFSET_SHIFT) as usize
    }

    pub const fn type_(&self) -> DiagnosticArgType {
        unsafe { std::mem::transmute(self.data >> DIAGNOSTIC_MESSAGE_ARG_OFFSET_BITS) }
    }
}

type DiagnosticMessageArgs = [DiagnosticMessageArgInfo; DIAGNOSTIC_MAX_ARG_COUNT];

const DIAGNOSTIC_INFO_CODE_BITS: u16 = 14;
const DIAGNOSTIC_INFO_CODE_MASK: u16 = (1 << DIAGNOSTIC_INFO_CODE_BITS) - 1;
const DIAGNOSTIC_INFO_SEVERITY_SHIFT: u16 = DIAGNOSTIC_INFO_CODE_BITS;

#[repr(C)]
pub struct DiagnosticInfo {
    // C++ equivalent:
    //
    // std::uint16_t code : 14;
    // diagnostic_severity severity : 2;
    code_and_severity: u16,

    pub message_formats: [TranslatableMessage; DIAGNOSTIC_MAX_MESSAGE_COUNT],
    pub message_args: [DiagnosticMessageArgs; DIAGNOSTIC_MAX_MESSAGE_COUNT],
}

impl DiagnosticInfo {
    pub const fn new(
        code: u16,
        severity: DiagnosticSeverity,
        message_formats: [TranslatableMessage; DIAGNOSTIC_MAX_MESSAGE_COUNT],
        message_args: [DiagnosticMessageArgs; DIAGNOSTIC_MAX_MESSAGE_COUNT],
    ) -> DiagnosticInfo {
        qljs_assert!((code & DIAGNOSTIC_INFO_CODE_MASK) == code);
        DiagnosticInfo {
            code_and_severity: code | ((severity as u16) << DIAGNOSTIC_INFO_SEVERITY_SHIFT),
            message_formats: message_formats,
            message_args: message_args,
        }
    }

    pub const fn code_string(&self) -> [u8; 5] {
        todo!(); // TODO(port)
    }

    pub const fn code(&self) -> u16 {
        self.code_and_severity & DIAGNOSTIC_INFO_CODE_MASK
    }

    pub const fn severity(&self) -> DiagnosticSeverity {
        unsafe {
            std::mem::transmute((self.code_and_severity >> DIAGNOSTIC_INFO_SEVERITY_SHIFT) as u8)
        }
    }
}

pub fn get_diagnostic_info(type_: DiagType) -> &'static DiagnosticInfo {
    &ALL_DIAGNOSTIC_INFOS[type_ as usize]
}

// TODO(port): diag_type_from_code_slow
