// Backport of Rust's is_aligned_to.
pub fn is_aligned_to<T>(p: *mut T, alignment: usize) -> bool {
    let alignment_mask = alignment - 1;
    ((p as usize) & alignment_mask) == 0
}

// Backport of Rust's byte_offset_from.
pub fn byte_offset_from<T, U>(lhs: *const T, rhs: *const U) -> isize {
    unsafe { (lhs as *const u8).offset_from(rhs as *const u8) }
}

// Backport of Rust's byte_add.
pub fn byte_add<T>(p: *const T, offset: usize) -> *const T {
    unsafe { (p as *const u8).add(offset) as *const T }
}
