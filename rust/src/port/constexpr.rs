// Port of some standard routines for constant evaluation.

pub const fn const_str_eq(x: &str, y: &str) -> bool {
    const_slice_eq_u8(x.as_bytes(), y.as_bytes())
}

pub const fn const_slice_eq_u8(x: &[u8], y: &[u8]) -> bool {
    if x.len() != y.len() {
        return false;
    }
    const_slice_eq_u8_impl(&x, &y, 0)
}

const fn const_slice_eq_u8_impl(x: &[u8], y: &[u8], i: usize) -> bool {
    if i >= x.len() {
        return true;
    }
    if x[i] != y[i] {
        return false;
    }
    const_slice_eq_u8_impl(x, y, i + 1)
}

pub const fn const_str_cmp(x: &str, y: &str) -> std::cmp::Ordering {
    const_slice_cmp_u8(x.as_bytes(), y.as_bytes())
}

pub const fn const_slice_cmp_u8(x: &[u8], y: &[u8]) -> std::cmp::Ordering {
    const_slice_cmp_u8_impl(&x, &y, 0)
}

const fn const_slice_cmp_u8_impl(x: &[u8], y: &[u8], i: usize) -> std::cmp::Ordering {
    if i == x.len() && i == y.len() {
        return std::cmp::Ordering::Equal;
    }
    if i >= x.len() {
        return std::cmp::Ordering::Less;
    }
    if i >= y.len() {
        return std::cmp::Ordering::Greater;
    }
    if x[i] < y[i] {
        return std::cmp::Ordering::Less;
    }
    if x[i] > y[i] {
        return std::cmp::Ordering::Greater;
    }
    const_slice_cmp_u8_impl(x, y, i + 1)
}

pub const fn maximum_u8(xs: &[u8]) -> u8 {
    let mut max: u8 = xs[0];
    let mut i: usize = 1;
    while i < xs.len() {
        if xs[i] > max {
            max = xs[i];
        }
        i += 1;
    }
    max
}
