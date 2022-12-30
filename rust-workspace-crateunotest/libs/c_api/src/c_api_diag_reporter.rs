use crate::c_api::*;
use crate::web_demo_location::*;
use cpp_vs_rust_container::linked_bump_allocator::*;
use cpp_vs_rust_container::monotonic_allocator::*;
use cpp_vs_rust_fe::diag_reporter::*;
use cpp_vs_rust_fe::diagnostic::*;
use cpp_vs_rust_fe::diagnostic_formatter::*;
use cpp_vs_rust_fe::diagnostic_types::*;
use cpp_vs_rust_fe::source_code_span::*;
use cpp_vs_rust_i18n::translation::*;
use cpp_vs_rust_port::maybe_uninit::*;
use cpp_vs_rust_util::narrow_cast::*;
use cpp_vs_rust_util::padded_string::*;
use cpp_vs_rust_util::qljs_assert;
use cpp_vs_rust_util::qljs_const_assert;

// NOTE(port): The C++ code had a generic Diagnostic parameter. KISS by inlining it to
// QLJSWebDemoDiagnostic.
// NOTE(port): The C++ code had a generic Locator parameter. KISS by inlining it to
// WebDemoLocator.
pub struct CAPIDiagReporter<'code> {
    translator: Translator,
    diagnostics: std::cell::UnsafeCell<Vec<QLJSWebDemoDiagnostic>>,
    _input: *const u8,
    input_phantom: std::marker::PhantomData<&'code [u8]>,
    locator: Option<WebDemoLocator<'code>>,
    string_allocator: MonotonicAllocator,
}

impl<'code> CAPIDiagReporter<'code> {
    pub fn new() -> Self {
        CAPIDiagReporter {
            translator: Translator::new_using_messages_from_source_code(),
            diagnostics: std::cell::UnsafeCell::new(vec![]),
            _input: std::ptr::null(),
            input_phantom: std::marker::PhantomData,
            locator: None,
            string_allocator: MonotonicAllocator::new("c_api_diag_reporter::string_allocator_"),
        }
    }

    pub fn set_input(&mut self, input: PaddedStringView<'code>) {
        self._input = input.c_str();
        self.locator = Some(WebDemoLocator::new(input));
    }

    // Does not reset translator.
    pub fn reset(&mut self) {
        self.diagnostics.get_mut().clear();
        // TODO(strager): Release allocated string memory.
    }

    pub fn set_translator(&mut self, t: Translator) {
        self.translator = t;
    }

    pub fn get_diagnostics(&mut self) -> *const QLJSWebDemoDiagnostic {
        // Null-terminate the returned diagnostics.
        self.diagnostics
            .get_mut()
            .push(QLJSWebDemoDiagnostic::default());

        self.diagnostics.get_mut().as_ptr()
    }

    fn allocate_c_string<'this>(&'this self, string: &[u8]) -> &'this [u8] {
        let result: &'this mut [std::mem::MaybeUninit<u8>] = self
            .string_allocator
            .allocate_uninitialized_array::<u8>(string.len() + 1);
        write_slice(&mut result[0..string.len()], string);
        result[string.len()].write(b'\0');
        unsafe { slice_assume_init_ref(result) }
    }
}

impl<'code> DiagReporter for CAPIDiagReporter<'code> {
    fn report_impl(&self, type_: DiagType, diag: *const u8) {
        let mut formatter = CAPIDiagFormatter::new(self);
        formatter.format(get_diagnostic_info(type_), diag);
    }
}

struct CAPIDiagFormatter<'code, 'reporter> {
    reporter: &'reporter CAPIDiagReporter<'code>,
    current_message: Vec<u8>,
}

impl<'code, 'reporter> CAPIDiagFormatter<'code, 'reporter> {
    fn new(reporter: &'reporter CAPIDiagReporter<'code>) -> Self {
        CAPIDiagFormatter {
            reporter: reporter,
            current_message: vec![],
        }
    }
}

impl<'code, 'reporter> DiagnosticFormatter for CAPIDiagFormatter<'code, 'reporter> {
    fn write_before_message(
        &mut self,
        _code: &str,
        sev: DiagnosticSeverity,
        _origin: SourceCodeSpan<'_>,
    ) {
        if sev == DiagnosticSeverity::Note {
            // Don't write notes. Only write the main message.
            return;
        }
        qljs_assert!(self.current_message.is_empty());
    }

    fn write_message_part(&mut self, _code: &str, sev: DiagnosticSeverity, message: &[u8]) {
        if sev == DiagnosticSeverity::Note {
            // Don't write notes. Only write the main message.
            return;
        }
        self.current_message.extend_from_slice(message);
    }

    fn write_after_message(
        &mut self,
        code: &str,
        sev: DiagnosticSeverity,
        origin: SourceCodeSpan<'_>,
    ) {
        let diag_severity: QLJSSeverity = match sev {
            DiagnosticSeverity::Note => {
                // Don't write notes. Only write the main message.
                return;
            }
            DiagnosticSeverity::Error => QLJSSeverity::Error,
            DiagnosticSeverity::Warning => QLJSSeverity::Warning,
        };
        let mut diag: QLJSWebDemoDiagnostic = QLJSWebDemoDiagnostic::default();
        let r: WebDemoSourceRange = self.reporter.locator.as_ref().unwrap().range(origin);
        diag.begin_offset = narrow_cast::<i32, _>(r.begin);
        diag.end_offset = narrow_cast::<i32, _>(r.end);

        qljs_const_assert!(std::mem::size_of::<u8>() == std::mem::size_of::<std::ffi::c_char>());
        diag.code[0..code.len()].copy_from_slice(unsafe {
            std::mem::transmute::<&[u8], &[std::ffi::c_char]>(code.as_bytes())
        });
        diag.code[code.len()] = b'\0' as std::ffi::c_char;

        diag.message = self
            .reporter
            .allocate_c_string(self.current_message.as_slice())
            .as_ptr();
        diag.severity = diag_severity;

        unsafe {
            (*self.reporter.diagnostics.get()).push(diag);
        }
    }

    fn translator(&self) -> Translator {
        self.reporter.translator.clone()
    }
}
