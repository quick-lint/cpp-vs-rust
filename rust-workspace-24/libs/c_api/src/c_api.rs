use crate::c_api_diag_reporter::*;
use cpp_vs_rust_fe::linter::*;
use cpp_vs_rust_i18n::translation::*;
use cpp_vs_rust_i18n::translation_table_generated::*;
use cpp_vs_rust_util::c_string::*;
use cpp_vs_rust_util::padded_string::*;
use cpp_vs_rust_util::qljs_assert;

#[allow(non_camel_case_types)]
pub type c_size_t = usize;

// A bit set (i.e. flags) which tell qljs_web_demo_lint how to interpret a
// QLJSWebDemoDocument's text.
//
// To associate options with a document, call
// qljs_web_demo_set_language_options.
pub type QLJSLanguageOptions = std::ffi::c_int;

// If set, parse JSX syntax. JSX is a JavaScript language extension.
//
// If unset, report a diagnostic if JSX syntax is encounted (e.g. E0177 or
// E0306).
//
// Ignored if qljs_language_options_config_json_bit is set.
pub const QLJS_LANGUAGE_OPTIONS_JSX_BIT: QLJSLanguageOptions = 1 << 0;

// If set, parse TypeScript instead of JavaScript.
//
// If unset, parse JavaScript, and report a diagnostic if TypeScript-specific
// syntax is encountered (e.g. E0222 or E0281).
//
// Ignored if qljs_language_options_config_json_bit is set.
pub const QLJS_LANGUAGE_OPTIONS_TYPESCRIPT_BIT: QLJSLanguageOptions = 1 << 1;

// If set, parse a quick-lint-js.config file instead of JavaScript.
//
// If unset, parse JavaScript or TypeScript.
pub const QLJS_LANGUAGE_OPTIONS_CONFIG_JSON_BIT: QLJSLanguageOptions = 1 << 2;

#[repr(C)]
pub enum QLJSSeverity {
    Error = 1,
    Warning = 2,
}

// A QLJSWebDemoDocument is a text document.
//
// A QLJSWebDemoDocument contains the following state:
//
// * Text, changed using qljs_web_demo_set_text
// * Language options, changed using qljs_web_demo_set_language_options
// * Configuration document, changed using qljs_web_demo_set_config
// * Locale, changed using qljs_web_demo_set_locale
// * Output diagnostics, changed using qljs_web_demo_lint
//
// QLJSWebDemoDocument objects are allocated dynamically. To create a
// QLJSWebDemoDocument, call qljs_web_demo_create_document. When you are
// finished using a QLJSWebDemoDocument, call qljs_web_demo_destroy_document
// to free resources.
//
// NOTE[QLJSWebDemoDocument threads]: In general, qljs_web_demo_* functions
// can be called from multiple threads without synchronization. However, for a
// given QLJSWebDemoDocument, functions accepting that QLJSWebDemoDocument
// *cannot* be called from multiple threads without synchronization.
//
// In other words, you can create documents A, B, and C, and use document A on
// thread 1, document B on thread 2, and document C on thread 3, with no
// synchronization. However, if instead you want to call
// qljs_web_demo_set_text(A, ...) on thread 1, then call qljs_web_demo_lint(A)
// on thread 2, then these calls must be synchronized by you.
//
// A mutex is sufficient synchronization.
pub struct QLJSWebDemoDocument {
    text: PaddedString,
    diag_reporter: CAPIDiagReporter</* HACK(strager) */ 'static>,
    linter_options: LinterOptions,
    is_config_json: bool,
    config_document: *mut QLJSWebDemoDocument,
    need_update_config: bool,
}

#[repr(C)]
pub struct QLJSWebDemoDiagnostic {
    pub message: *const u8,
    pub code: [std::ffi::c_char; 6], // null-terminated
    pub severity: QLJSSeverity,
    // Offsets count UTF-16 code units.
    pub begin_offset: std::ffi::c_int,
    pub end_offset: std::ffi::c_int,
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

// Create a new document.
//
// The new document ('d') has the following state:
//
// * No text, as if by qljs_web_demo_set_text(d, "", 0)
// * No language options set, as if by qljs_web_demo_set_language_options(d, 0)
// * No configuration document, as if by qljs_web_demo_set_config(d, NULL)
// * A default locale, as if by qljs_web_demo_set_locale(d, default_locale)
//   * TODO(strager): What is default_locale?
// * Unspecified output diagnostics
//
// Thread safety: Thread-safe. Not async-signal-safe.
//
// Postcondition: The returned value is not null.
#[no_mangle]
pub unsafe extern "C" fn qljs_web_demo_create_document() -> *mut QLJSWebDemoDocument {
    let p: Box<QLJSWebDemoDocument> = Box::new(QLJSWebDemoDocument {
        text: PaddedString::new(),
        diag_reporter: CAPIDiagReporter::new(),
        linter_options: LinterOptions::default(),
        is_config_json: false,
        config_document: std::ptr::null_mut(),
        need_update_config: true,
    });
    Box::leak(p) as *mut _
}

// Free resources which were allocated for the given document.
//
// After calling qljs_web_demo_destroy_document, the document pointer should
// never be used.
//
// Thread safety: See NOTE[QLJSWebDemoDocument threads].
//
// Precondition: qljs_web_demo_create_document() previously returned document.
// Precondition: qljs_web_demo_destroy_document(document) was not previously
//               called.
#[no_mangle]
pub unsafe extern "C" fn qljs_web_demo_destroy_document(p: *mut QLJSWebDemoDocument) {
    let p: Box<QLJSWebDemoDocument> = Box::from_raw(p);
    std::mem::drop(p);
}

// Make qljs_web_demo_lint use this text.
//
// qljs_web_demo_set_text makes an internal copy of the given array. To change
// the document's text, you cannot just modify the array pointed to by
// text_utf_8; you must call qljs_web_demo_set_text again.
//
// If qljs_web_demo_set_config(js_document, document) was previously called,
// then in order for the new config to take effect for js_document,
// qljs_web_demo_lint(js_document) must be called. (You couldn't notice without
// calling qljs_web_demo_lint anyway...)
//
// Thread safety: See NOTE[QLJSWebDemoDocument threads].
//
// Precondition: qljs_web_demo_create_document() returned document, and
//               qljs_web_demo_destroy_document(document) has not been called.
// Precondition: text_utf_8 points to an array of at least text_byte_count
//               bytes.
// Precondition: text_utf_8 is not null, even if text_byte_count is 0.
#[no_mangle]
pub unsafe extern "C" fn qljs_web_demo_set_text(
    p: *mut QLJSWebDemoDocument,
    text_utf_8: *const std::ffi::c_void,
    text_byte_count: c_size_t,
) {
    (*p).text = PaddedString::from_slice(std::slice::from_raw_parts(
        text_utf_8 as *const u8,
        text_byte_count,
    ));
}

// When running qljs_web_demo_lint(js_document), treat config_document's text as
// if it was js_document's associated quick-lint-js.config file.
//
// config_document's language options are ignored.
//
// config_document is optional. If null, reverts to the default config.
//
// Thread safety: See NOTE[QLJSWebDemoDocument threads].
//
// Precondition: qljs_web_demo_create_document() returned js_document, and
//               qljs_web_demo_destroy_document(js_document) has not been
//               called.
// Precondition: config_document is null, or: qljs_web_demo_create_document()
//               returned config_document, and
//               qljs_web_demo_destroy_document(config_document) has not been
//               called.
#[no_mangle]
pub unsafe extern "C" fn qljs_web_demo_set_config(
    js_document: *mut QLJSWebDemoDocument,
    config_document: *mut QLJSWebDemoDocument,
) {
    (*js_document).need_update_config = true;
    (*js_document).config_document = config_document;
}

// Change how qljs_web_demo_lint(document) parses and interprets document's
// text.
//
// options is a bit set. See qljs_language_options for details.
//
// Thread safety: See NOTE[QLJSWebDemoDocument threads].
//
// Precondition: qljs_web_demo_create_document() returned document, and
//               qljs_web_demo_destroy_document(document) has not been called.
// Precondition: options is a bitwise-or of zero or more qljs_language_options
//               members. (options==0 is permitted.)
#[no_mangle]
pub unsafe extern "C" fn qljs_web_demo_set_language_options(
    p: *mut QLJSWebDemoDocument,
    options: QLJSLanguageOptions,
) {
    (*p).linter_options.jsx = (options & QLJS_LANGUAGE_OPTIONS_JSX_BIT) != 0;
    (*p).linter_options.typescript = (options & QLJS_LANGUAGE_OPTIONS_TYPESCRIPT_BIT) != 0;
    (*p).is_config_json = (options & QLJS_LANGUAGE_OPTIONS_CONFIG_JSON_BIT) != 0;
}

// Change the human language which qljs_web_demo_lint(document) uses for its
// diagnostics.
//
// locale can compare equal to a string returned by qljs_list_locales, or it can
// be any other string.
//
// If locale matches no supported locales, then this sets document's locale to
// the default locale (which corresponds to professional US English).
//
// Thread safety: See NOTE[QLJSWebDemoDocument threads].
//
// Precondition: qljs_web_demo_create_document() returned document, and
//               qljs_web_demo_destroy_document(document) has not been called.
// Precondition: locale points to a C string.
// Precondition: locale is not null.
#[no_mangle]
pub unsafe extern "C" fn qljs_web_demo_set_locale(
    p: *mut QLJSWebDemoDocument,
    locale: *const std::ffi::c_char,
) {
    let mut t: Translator = Translator::new_using_messages_from_source_code();
    t.use_messages_from_locale(read_utf8_c_string(locale as *const u8));
    (*p).diag_reporter.set_translator(t);
}

// Parse and lint document's text [1], according to its language options [2] and
// config [3], and return a list of diagnostics according to document's
// locale [4].
//
// The returned pointer refers to an array of qljs_web_demo_diagnostic objects.
// The array is terminated by an item where:
// * qljs_web_demo_diagnostic::message is null, and
// * qljs_web_demo_diagnostic::code is an empty string.
//
// The returned pointer is valid until either the next call to
// qljs_web_demo_lint(document) or a call to
// qljs_web_demo_destroy_document(document), whichever comes first.
//
// [1] qljs_web_demo_set_text
// [2] qljs_web_demo_set_language_options
// [3] qljs_web_demo_set_config
// [4] qljs_web_demo_set_locale
//
// Thread safety: See NOTE[QLJSWebDemoDocument threads].
//
// Precondition: qljs_web_demo_create_document() returned document, and
//               qljs_web_demo_destroy_document(document) has not been called.
// Precondition: qljs_web_demo_destroy_document(config_document) has not been
//               called, where config_document is the QLJSWebDemoDocument
//               associated with this document via qljs_web_demo_set_config.
// Postcondition: The returned value is not null.
#[no_mangle]
pub unsafe extern "C" fn qljs_web_demo_lint(
    p: *mut QLJSWebDemoDocument,
) -> *const QLJSWebDemoDiagnostic {
    (*p).diag_reporter.reset();
    (*p).diag_reporter.set_input((*p).text.view());
    if !(*p).is_config_json {
        parse_and_lint((*p).text.view(), &(*p).diag_reporter, (*p).linter_options);
    }
    (*p).diag_reporter.get_diagnostics()
}

// Returns a null-terminated array of null-terminated strings.
//
// Every call to qljs_list_locales will return the same pointer (for a given
// process).
//
// Thread safety: Thread-safe. Not async-signal-safe.
//
// Postcondition: The returned value is not null.
// Postcondition: The returned array contains at least one non-empty string.
#[no_mangle]
pub unsafe extern "C" fn qljs_list_locales() -> *const *const std::ffi::c_char {
    static mut LOCALES: *const *const std::ffi::c_char = std::ptr::null();
    static INIT_LOCALES: std::sync::Once = std::sync::Once::new();
    INIT_LOCALES.call_once(|| {
        const LOCALE_COUNT: usize = (TRANSLATION_TABLE_LOCALE_COUNT + 1) as usize;
        let mut locales: Box<[*const std::ffi::c_char]> =
            Box::new([std::ptr::null(); LOCALE_COUNT + 1]);

        let mut i: usize = 0;
        let mut l: &[u8] = TRANSLATION_DATA_LOCALE_TABLE.as_bytes();
        while l[0] != b'\0' {
            locales[i] = l.as_ptr() as *const std::ffi::c_char;
            i += 1;
            l = &l[(l.iter().position(|c: &u8| *c == b'\0').unwrap() + 1)..];
        }
        locales[i] = l.as_ptr() as *const std::ffi::c_char; // Default locale (empty string).
        i += 1;
        qljs_assert!(i == LOCALE_COUNT);
        // Terminator was added when we initialized locales.

        LOCALES = Box::leak(locales).as_ptr();
    });
    LOCALES
}
