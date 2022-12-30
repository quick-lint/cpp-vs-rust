use crate::c_api::c_api::*;
use crate::i18n::translation_table_generated::*;
use crate::util::c_string::*;

#[test]
fn empty_document_has_no_diagnostics() {
    unsafe {
        let p: *mut QLJSWebDemoDocument = qljs_web_demo_create_document();
        let diagnostics: *const QLJSWebDemoDiagnostic = qljs_web_demo_lint(p);
        assert_eq!((*diagnostics.add(0)).message, std::ptr::null());
        qljs_web_demo_destroy_document(p);
    }
}

#[test]
fn lint_error_after_text_insertion() {
    unsafe {
        let p: *mut QLJSWebDemoDocument = qljs_web_demo_create_document();

        let document_text: &[u8] = b"'unfinished";
        qljs_web_demo_set_text(
            p,
            document_text.as_ptr() as *const std::ffi::c_void,
            document_text.len(),
        );
        let diagnostics: *const QLJSWebDemoDiagnostic = qljs_web_demo_lint(p);
        assert_ne!((*diagnostics.add(0)).message, std::ptr::null());
        assert_eq!((*diagnostics.add(1)).message, std::ptr::null());
        assert_eq!(
            read_utf8_c_string_from_c_slice(&(*diagnostics.add(1)).code),
            ""
        );

        assert_eq!(
            read_utf8_c_string((*diagnostics.add(0)).message),
            "unclosed string literal"
        );
        assert_eq!(
            read_utf8_c_string_from_c_slice(&(*diagnostics.add(0)).code),
            "E0040"
        );
        assert_eq!((*diagnostics.add(0)).begin_offset as usize, b"".len());
        assert_eq!(
            (*diagnostics.add(0)).end_offset as usize,
            b"'unfinished".len()
        );

        qljs_web_demo_destroy_document(p);
    }
}

#[test]
fn lint_new_error_after_second_text_insertion() {
    unsafe {
        let p: *mut QLJSWebDemoDocument = qljs_web_demo_create_document();

        let document_text: &[u8] = b"let x";
        qljs_web_demo_set_text(
            p,
            document_text.as_ptr() as *const std::ffi::c_void,
            document_text.len(),
        );
        let diagnostics: *const QLJSWebDemoDiagnostic = qljs_web_demo_lint(p);
        assert_eq!((*diagnostics.add(0)).message, std::ptr::null());

        let document_text_2: &[u8] = b"let x = 'unfinished";
        qljs_web_demo_set_text(
            p,
            document_text_2.as_ptr() as *const std::ffi::c_void,
            document_text_2.len(),
        );
        let diagnostics: *const QLJSWebDemoDiagnostic = qljs_web_demo_lint(p);
        assert_ne!((*diagnostics.add(0)).message, std::ptr::null());
        assert_eq!((*diagnostics.add(1)).message, std::ptr::null());
        assert_eq!(
            read_utf8_c_string_from_c_slice(&(*diagnostics.add(1)).code),
            ""
        );

        assert_eq!(
            read_utf8_c_string((*diagnostics.add(0)).message),
            "unclosed string literal"
        );
        assert_eq!(
            read_utf8_c_string_from_c_slice(&(*diagnostics.add(0)).code),
            "E0040"
        );
        assert_eq!(
            (*diagnostics.add(0)).begin_offset as usize,
            b"let x = ".len()
        );
        assert_eq!(
            (*diagnostics.add(0)).end_offset as usize,
            b"let x = 'unfinished".len()
        );

        qljs_web_demo_destroy_document(p);
    }
}

#[test]
fn setting_locale_changes_messages_forever() {
    unsafe {
        let p: *mut QLJSWebDemoDocument = qljs_web_demo_create_document();

        qljs_web_demo_set_locale(p, b"en_US@snarky\0".as_ptr() as *const std::ffi::c_char);

        let document_text_1: &[u8] = b"'unfinished";
        qljs_web_demo_set_text(
            p,
            document_text_1.as_ptr() as *const std::ffi::c_void,
            document_text_1.len(),
        );
        let diagnostics: *const QLJSWebDemoDiagnostic = qljs_web_demo_lint(p);
        assert_eq!(
            read_utf8_c_string((*diagnostics.add(0)).message),
            "\"unclosed string literal"
        );

        let document_text_2: &[u8] = b"`unfinished";
        qljs_web_demo_set_text(
            p,
            document_text_2.as_ptr() as *const std::ffi::c_void,
            document_text_2.len(),
        );
        let diagnostics: *const QLJSWebDemoDiagnostic = qljs_web_demo_lint(p);
        assert_eq!(
            read_utf8_c_string((*diagnostics.add(0)).message),
            "`unclosed template"
        );

        qljs_web_demo_destroy_document(p);
    }
}

#[test]
fn locale_list() {
    unsafe {
        let mut locale_strings: Vec<String> = vec![];
        let locales: *const *const std::ffi::c_char = qljs_list_locales();
        let mut l: *const *const std::ffi::c_char = locales;
        while !(*l).is_null() {
            locale_strings.push(String::from(read_utf8_c_string(*l as *const u8)));
            l = l.add(1);
        }
        locale_strings.sort();

        let mut expected_locale_strings: Vec<String> = vec![];
        let mut l: &[u8] = TRANSLATION_DATA_LOCALE_TABLE.as_bytes();
        while l[0] != b'\0' {
            expected_locale_strings.push(String::from(read_utf8_c_string_from_slice(l)));
            l = &l[(l.iter().position(|c: &u8| *c == b'\0').unwrap() + 1)..];
        }
        expected_locale_strings.push(String::new());
        expected_locale_strings.sort();

        assert_eq!(locale_strings, expected_locale_strings);
    }
}
