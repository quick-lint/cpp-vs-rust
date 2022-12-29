#[repr(C)]
pub enum QLJSSeverity {
    Error = 1,
    Warning = 2,
}

#[repr(C)]
pub struct QLJSWebDemoDiagnostic {
    pub message: *const u8,
    pub code: [u8; 6], // null-terminated
    pub severity: QLJSSeverity,
    // Offsets count UTF-16 code units.
    pub begin_offset: i32,
    pub end_offset: i32,
}

impl Default for QLJSWebDemoDiagnostic {
    fn default() -> Self {
        QLJSWebDemoDiagnostic {
            message: std::ptr::null(),
            code: [0; 6],
            severity: QLJSSeverity::Error,
            begin_offset: 0,
            end_offset: 0,
        }
    }
}
