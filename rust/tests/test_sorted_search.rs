use cpp_vs_rust::container::*;

#[test]
fn empty_never_finds() {
    let data: [&str; 0] = [];
    let result: Option<usize> = sorted_search(&data, "hi");
    assert_eq!(result, None);
}

#[test]
fn single_item_array_with_match() {
    let data: [&str; 1] = ["hi"];
    let result: Option<usize> = sorted_search(&data, "hi");
    assert_eq!(result, Some(0));
}

#[test]
fn single_item_array_with_no_match() {
    let data: [&str; 1] = ["hi"];
    assert_eq!(sorted_search(&data, "aye"), None);
    assert_eq!(sorted_search(&data, "yo"), None);
}

#[test]
fn two_item_array() {
    let data: [&str; 2] = ["b", "d"];
    assert_eq!(sorted_search(&data, "a"), None);
    assert_eq!(sorted_search(&data, "b"), Some(0));
    assert_eq!(sorted_search(&data, "c"), None);
    assert_eq!(sorted_search(&data, "d"), Some(1));
    assert_eq!(sorted_search(&data, "e"), None);
}

#[test]
fn three_item_array() {
    let data: [&str; 3] = ["b", "d", "f"];
    assert_eq!(sorted_search(&data, "a"), None);
    assert_eq!(sorted_search(&data, "b"), Some(0));
    assert_eq!(sorted_search(&data, "c"), None);
    assert_eq!(sorted_search(&data, "d"), Some(1));
    assert_eq!(sorted_search(&data, "e"), None);
    assert_eq!(sorted_search(&data, "f"), Some(2));
    assert_eq!(sorted_search(&data, "g"), None);
}
