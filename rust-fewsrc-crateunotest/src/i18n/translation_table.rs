use crate::container::sorted_search::*;
use crate::i18n::translation_table_generated::*;

pub const TRANSLATION_TABLE_UNALLOCATED_MAPPING_INDEX: u16 = 0;

pub struct TranslationTableMappingEntry(
    // string offsets
    pub [u32; (TRANSLATION_TABLE_LOCALE_COUNT + 1) as usize],
);

pub const fn translation_table_const_look_up(untranslated: &str) -> u16 {
    let index: Option<usize> = sorted_search(&UNTRANSLATED_STRINGS, untranslated);
    match index {
        Some(index) => (index + 1) as u16,
        None => {
            panic!("translation_table_generated.rs is out of date. Run tools/update-translator-sources to rebuild this file.");
        }
    }
}
