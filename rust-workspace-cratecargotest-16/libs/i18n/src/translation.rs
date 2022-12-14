use crate::locale::*;
use crate::translation_table::*;
use crate::translation_table_generated::*;
use cpp_vs_rust_util::c_string::*;
use cpp_vs_rust_util::qljs_assert;
use cpp_vs_rust_util::qljs_c_string;

#[macro_export]
macro_rules! qljs_translatable {
    ($string:expr $(,)?) => {{
        const ID: u16 = $crate::translation_table::translation_table_const_look_up($string);
        $crate::translation::TranslatableMessage(ID)
    }};
}

// Global instance.
pub static mut QLJS_MESSAGES: Translator = Translator::new_using_messages_from_source_code();

pub fn initialize_locale() {
    unsafe {
        libc::setlocale(libc::LC_ALL, qljs_c_string!("") as *const std::ffi::c_char);
    }
}

pub fn initialize_translations_from_locale(locale_name: &str) {
    initialize_locale();
    unsafe {
        if !QLJS_MESSAGES.use_messages_from_locale(locale_name) {
            QLJS_MESSAGES.use_messages_from_source_code();
        }
    }
}

#[derive(Clone)]
pub struct Translator {
    locale_index: i32,
}

impl Translator {
    // Creates a translator which uses messages from the source code (i.e. no-op).
    pub const fn new_using_messages_from_source_code() -> Translator {
        Translator {
            locale_index: TRANSLATION_TABLE_LOCALE_COUNT as i32,
        }
    }

    pub fn use_messages_from_source_code(&mut self) {
        self.locale_index = TRANSLATION_TABLE_LOCALE_COUNT as i32;
    }

    pub fn use_messages_from_locale(&mut self, locale_name: &str) -> bool {
        match find_locale(TRANSLATION_DATA_LOCALE_TABLE.as_bytes(), locale_name) {
            Some(locale_index) => {
                self.locale_index = locale_index;
                true
            }
            None => false,
        }
    }

    pub fn use_messages_from_locales(&mut self, locale_names: &[&str]) -> bool {
        for locale in locale_names {
            if *locale == "C" || *locale == "POSIX" {
                // Stop seaching. C/POSIX locale takes priority. See GNU gettext.
                break;
            }
            let found_messages: bool = self.use_messages_from_locale(locale);
            if found_messages {
                return true;
            }
        }
        false
    }

    pub fn translate(&self, message: TranslatableMessage) -> &'static str {
        // If the following assertion fails, it's likely that
        // translation-table-generated.h is out of date. Run
        // tools/update-translator-sources to rebuild that file.
        qljs_assert!(message.valid());

        let mapping_index: u16 = message.translation_table_mapping_index();
        let mapping: &TranslationTableMappingEntry =
            &TRANSLATION_DATA_MAPPING_TABLE[mapping_index as usize];
        let mut string_offset: u32 = mapping.0[self.locale_index as usize];
        if string_offset == 0 {
            // The string has no translation.
            string_offset = mapping.0[TRANSLATION_TABLE_LOCALE_COUNT as usize];
            qljs_assert!(string_offset != 0);
        }
        let string_and_other_stuff: &[u8] =
            &TRANSLATION_DATA_STRING_TABLE[string_offset as usize..];
        unsafe { read_utf8_c_string_from_slice(string_and_other_stuff) }
    }
}

// An un-translated message.
#[derive(Clone, Copy)]
pub struct TranslatableMessage(pub u16);

impl TranslatableMessage {
    pub const fn unallocated() -> TranslatableMessage {
        TranslatableMessage(TRANSLATION_TABLE_UNALLOCATED_MAPPING_INDEX)
    }

    pub const fn from(untranslated: &str) -> TranslatableMessage {
        TranslatableMessage(translation_table_const_look_up(untranslated))
    }

    pub const fn valid(&self) -> bool {
        self.0 != TRANSLATION_TABLE_UNALLOCATED_MAPPING_INDEX
    }

    pub const fn translation_table_mapping_index(&self) -> u16 {
        self.0
    }
}
