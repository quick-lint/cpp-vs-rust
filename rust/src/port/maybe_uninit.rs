// Backport of Rust's std::mem::MaybeUninit::write_slice.
pub fn write_slice<'a, T: Copy>(dest: &'a mut [std::mem::MaybeUninit<T>], src: &[T]) {
    let src_slice: &[std::mem::MaybeUninit<T>] = unsafe { std::mem::transmute(src) };
    dest.copy_from_slice(src_slice);
}
