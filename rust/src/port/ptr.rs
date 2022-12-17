// Backport of Rust's is_aligned_to.
pub fn is_aligned_to<T>(p: *mut T, alignment: usize) -> bool {
    let alignment_mask = alignment - 1;
    ((p as usize) & alignment_mask) == 0
}
