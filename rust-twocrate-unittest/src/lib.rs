pub mod c_api;
pub mod container;
pub mod fe;
pub mod i18n;
pub mod port;
pub mod test;
pub mod util;

#[cfg(test)]
mod test_assert;
#[cfg(test)]
mod test_buffering_diag_reporter;
#[cfg(test)]
mod test_c_api;
#[cfg(test)]
mod test_constexpr;
#[cfg(test)]
mod test_diagnostic;
#[cfg(test)]
mod test_diagnostic_formatter;
#[cfg(test)]
mod test_document;
#[cfg(test)]
mod test_lex;
#[cfg(test)]
mod test_linked_bump_allocator;
#[cfg(test)]
mod test_linked_vector;
#[cfg(test)]
mod test_locale;
#[cfg(test)]
mod test_offset_of;
#[cfg(test)]
mod test_padded_string;
#[cfg(test)]
mod test_permutations;
#[cfg(test)]
mod test_simd;
#[cfg(test)]
mod test_sorted_search;
#[cfg(test)]
mod test_translation;
#[cfg(test)]
mod test_translation_table_generated;
#[cfg(test)]
mod test_utf_8;
#[cfg(test)]
mod test_vector;
#[cfg(test)]
mod test_web_demo_location;
