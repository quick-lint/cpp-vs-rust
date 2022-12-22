use cpp_vs_rust_port::constexpr::*;

#[test]
fn str_cmp_matches_standard() {
    for (lhs, rhs) in [
        ("", ""),
        ("a", ""),
        ("a", "b"),
        ("", "b"),
        ("a", "a"),
        ("abcdefg", "abcdefg"),
        ("abcdefh", "abcdefg"),
        ("abcdefa", "abcdefg"),
        ("abcdefg", "abcdefh"),
        ("abcdefg", "abcdefa"),
    ] {
        assert_eq!(
            const_str_cmp(lhs, rhs),
            lhs.cmp(rhs),
            "lhs={:?} rhs={:?}",
            lhs,
            rhs
        );
    }
}
