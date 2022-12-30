use cpp_vs_rust_port::constexpr::*;

pub const fn sorted_search(haystack: &[&str], needle: &str) -> Option<usize> {
    use std::cmp::Ordering;

    let length: isize = haystack.len() as isize;
    let mut lo: isize = 0;
    let mut hi: isize = length - 1;
    while lo <= hi {
        let mid: isize = (lo + hi) / 2;
        match const_str_cmp(haystack[mid as usize], needle) {
            Ordering::Less => {
                lo = mid + 1;
            }
            Ordering::Greater => {
                hi = mid - 1;
            }
            Ordering::Equal => {
                return Some(mid as usize);
            }
        }
    }
    None
}
