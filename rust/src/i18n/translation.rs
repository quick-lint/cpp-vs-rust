use crate::i18n::locale::*;
use crate::i18n::translation_table::*;
use crate::i18n::translation_table_generated::*;
use crate::qljs_assert;

#[macro_export]
macro_rules! qljs_translatable {
    ($string:expr $(,)?) => {{
        const ID: u16 = $crate::i18n::translation_table::translation_table_const_look_up($string);
        $crate::i18n::translation::TranslatableMessage(ID)
    }};
}

// Global instance.
static mut QLJS_MESSAGES: Translator = Translator::new_using_messages_from_source_code();

fn initialize_locale() {
    // TODO(port): Call C's setlocale:
    // std::setlocale(LC_ALL, "")
}

fn initialize_translations_from_locale(locale_name: &str) {
    initialize_locale();
    unsafe {
        if !QLJS_MESSAGES.use_messages_from_locale(locale_name) {
            QLJS_MESSAGES.use_messages_from_source_code();
        }
    }
}

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
        unsafe { read_utf8_c_string(string_and_other_stuff) }
    }
}

// Returns a str for data up until (but not including) a null terminator.
unsafe fn read_utf8_c_string(bytes: &[u8]) -> &str {
    std::str::from_utf8_unchecked(&bytes[0..bytes.iter().position(|c| *c == 0).unwrap_unchecked()])
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
        return self.0 != TRANSLATION_TABLE_UNALLOCATED_MAPPING_INDEX;
    }

    pub const fn translation_table_mapping_index(&self) -> u16 {
        self.0
    }
}
