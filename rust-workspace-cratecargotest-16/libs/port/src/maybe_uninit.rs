// Backport of Rust's std::mem::MaybeUninit::write_slice.
pub fn write_slice<'a, T: Copy>(dest: &'a mut [std::mem::MaybeUninit<T>], src: &[T]) {
    let src_slice: &[std::mem::MaybeUninit<T>] = unsafe { std::mem::transmute(src) };
    dest.copy_from_slice(src_slice);
}

// Backport of Rust's std::mem::MaybeUninit::slice_assume_init_ref.
pub unsafe fn slice_assume_init_ref<T>(slice: &[std::mem::MaybeUninit<T>]) -> &[T] {
    std::mem::transmute(slice)
}

// Backport of Rust's std::mem::MaybeUninit::array_assume_init.
pub unsafe fn array_assume_init<T, const N: usize>(array: [std::mem::MaybeUninit<T>; N]) -> [T; N] {
    let ptr = &array as *const [std::mem::MaybeUninit<T>; N];
    (ptr as *const [T; N]).read()
}

// Backport of Rust's std::mem::MaybeUninit::uninit_array.
pub fn uninit_array<T, const N: usize>() -> [std::mem::MaybeUninit<T>; N] {
    unsafe { std::mem::MaybeUninit::<[std::mem::MaybeUninit<T>; N]>::uninit().assume_init() }
}
