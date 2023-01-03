use cpp_vs_rust_i18n::locale::*;
use cpp_vs_rust_util::permutations::*;

#[test]
fn combinations_for_language() {
    assert_eq!(locale_name_combinations("en"), vec!["en"]);
}

#[test]
fn combinations_for_language_with_territory() {
    assert_eq!(locale_name_combinations("fr_FR"), vec!["fr_FR", "fr"]);
}

#[test]
fn combinations_for_language_with_codeset() {
    assert_eq!(locale_name_combinations("fr.utf8"), vec!["fr.utf8", "fr"]);
}

#[test]
fn combinations_for_language_with_modifier() {
    assert_eq!(locale_name_combinations("fr@bon"), vec!["fr@bon", "fr"]);
}

#[test]
fn combinations_for_language_with_territory_and_modifier() {
    assert_eq!(
        locale_name_combinations("fr_FR@bon"),
        vec!["fr_FR@bon", "fr@bon", "fr_FR", "fr"]
    );
}

#[test]
fn combinations_for_language_with_territory_and_codeset() {
    assert_eq!(
        locale_name_combinations("fr_FR.utf8"),
        vec!["fr_FR.utf8", "fr_FR", "fr.utf8", "fr"]
    );
}

#[test]
fn combinations_for_language_with_territory_and_codeset_and_modifier() {
    assert_eq!(
        locale_name_combinations("fr_FR.utf8@bon"),
        vec![
            "fr_FR.utf8@bon",
            "fr_FR@bon",
            "fr.utf8@bon",
            "fr@bon",
            "fr_FR.utf8",
            "fr_FR",
            "fr.utf8",
            "fr"
        ]
    );
}

#[test]
fn modifier_can_contain_underscores_and_at_signs() {
    assert_eq!(locale_name_combinations("fr@a_b@c"), vec!["fr@a_b@c", "fr"]);
}

#[test]
fn exact_match_locale() {
    iterate_permutations(&["fr_FR", "en@slang"], |locales: &[&str]| {
        let locales_string: Vec<u8> = make_locales_string(locales);
        let fr_index: Option<i32> = find_locale(locales_string.as_slice(), "fr_FR");
        assert_eq!(locales[fr_index.unwrap() as usize], "fr_FR");
        let en_index: Option<i32> = find_locale(locales_string.as_slice(), "en@slang");
        assert_eq!(locales[en_index.unwrap() as usize], "en@slang");
    });
}

#[test]
fn no_match() {
    iterate_permutations(&["fr_FR", "en@slang"], |locales: &[&str]| {
        let locales_string: Vec<u8> = make_locales_string(locales);
        assert_eq!(find_locale(locales_string.as_slice(), "de_DE@a"), None);
    });
}

#[test]
fn match_subset_of_locale_name() {
    iterate_permutations(&["fr", "en"], |locales: &[&str]| {
        let locales_string: Vec<u8> = make_locales_string(locales);
        let fr_index: Option<i32> = find_locale(locales_string.as_slice(), "fr_FR.utf8@bon");
        assert_eq!(locales[fr_index.unwrap() as usize], "fr");
    });
}

fn make_locales_string(locales: &[&str]) -> Vec<u8> {
    let mut locales_string: Vec<u8> = vec![];
    for locale in locales {
        locales_string.extend_from_slice(locale.as_bytes());
        locales_string.push(0);
    }
    locales_string.push(0);
    locales_string
}
