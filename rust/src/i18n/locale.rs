use crate::qljs_assert;

// Returns the index of the matching locale.
//
// If locales is "en_US\0fr_FR\0de_DE\0", and locale_name is "fr_FR", then the
// result will be 1.
pub fn find_locale(locales: &[u8], locale_name: &str) -> Option<i32> {
    let mut found_entry: Option<i32> = None;
    iterate_locale_name_combinations(locale_name, |current_locale_name: &str| {
        let mut i: i32 = 0;
        for l in locales.split(|c| *c == 0) {
            if l.is_empty() {
                break;
            }
            if l == current_locale_name.as_bytes() {
                found_entry = Some(i);
                return false;
            }
            i += 1;
        }
        true
    });
    found_entry
}

// TODO(port): Make this [u8; 3] to reduce casting?
const LOCALE_PART_SEPARATORS: [char; 3] = ['_', '.', '@'];

struct LocaleParts<'a> {
    // language, territory, codeset, modifier
    parts: [&'a str; 4],
}

// Indexes into LocaleParts::parts.
const LANGUAGE_INDEX: usize = 0;
const TERRITORY_INDEX: usize = 1;
const CODESET_INDEX: usize = 2;
const MODIFIER_INDEX: usize = 3;

impl<'a> LocaleParts<'a> {
    fn language(&self) -> &'a str {
        self.parts[LANGUAGE_INDEX]
    }
}

fn parse_locale<'a>(locale_name: &'a str) -> LocaleParts<'a> {
    struct FoundSeparator {
        length: usize,
        which_separator: usize,
    }
    const INVALID_WHICH_SEPARATOR: usize = (-1 as isize) as usize;

    let find_next_separator = |c: &str, separators: &[char]| -> FoundSeparator {
        match c.find(separators) {
            None => FoundSeparator {
                length: c.len(),
                which_separator: INVALID_WHICH_SEPARATOR,
            },
            Some(length) => {
                let found_separator = unsafe { *c.as_bytes().get_unchecked(length) } as char;
                match separators.iter().position(|c| *c == found_separator) {
                    None => {
                        unreachable!();
                    }
                    Some(which_separator) => FoundSeparator {
                        length: length,
                        which_separator: which_separator,
                    },
                }
            }
        }
    };

    let mut parts: LocaleParts = LocaleParts { parts: [""; 4] };

    let mut current_separators: &'static [char] = &LOCALE_PART_SEPARATORS;
    let mut current_part: &mut [&str] = &mut parts.parts[..];
    let mut c: &str = locale_name;
    loop {
        let part: FoundSeparator = find_next_separator(c, current_separators);
        current_part[0] = &c[..part.length];
        c = &c[part.length..];
        if c.is_empty() {
            break;
        }

        qljs_assert!(part.which_separator != INVALID_WHICH_SEPARATOR);
        current_separators = &current_separators[(part.which_separator + 1)..];
        current_part = &mut current_part[(part.which_separator + 1)..];
        c = &c[1..];
    }

    parts
}

pub fn locale_name_combinations(locale_name: &str) -> Vec<String> {
    let mut locale_names: Vec<String> = vec![];
    iterate_locale_name_combinations(locale_name, |current_locale| {
        locale_names.push(String::from(current_locale));
        true
    });
    locale_names
}

fn iterate_locale_name_combinations<Func: FnMut(&str) -> bool>(
    locale_name: &str,
    mut callback: Func,
) {
    let parts: LocaleParts = parse_locale(locale_name);

    let mut locale: Vec<u8> = vec![];
    let max_locale_size: usize = locale_name.len();
    locale.reserve(max_locale_size);
    locale.extend_from_slice(parts.language().as_bytes());

    let mut present_parts_mask: u8 = 0;
    for part_index in 1..4 {
        let part: &str = parts.parts[part_index];
        present_parts_mask |= (if part.is_empty() { 0 } else { 1 }) << part_index;
    }

    const TERRITORY: u8 = 1 << TERRITORY_INDEX;
    const CODESET: u8 = 1 << CODESET_INDEX;
    const MODIFIER: u8 = 1 << MODIFIER_INDEX;
    #[rustfmt::skip]
  let masks: [u8; 8] = [
      TERRITORY | CODESET | MODIFIER,
      TERRITORY           | MODIFIER,
                  CODESET | MODIFIER,
                            MODIFIER,
      TERRITORY | CODESET,
      TERRITORY,
                  CODESET,
      0,
  ];
    for mask in masks {
        if (present_parts_mask & mask) != mask {
            continue;
        }

        locale.resize(parts.language().as_bytes().len(), 0xff);
        for part_index in 1..4 {
            if (mask & (1 << part_index)) != 0 {
                let part: &str = parts.parts[part_index];
                locale.push(LOCALE_PART_SEPARATORS[part_index - 1] as u8);
                locale.extend_from_slice(part.as_bytes());
            }
        }
        qljs_assert!(locale.len() <= max_locale_size);

        let keep_going = callback(unsafe { std::str::from_utf8_unchecked(locale.as_slice()) });
        if !keep_going {
            break;
        }
    }
}
