use crate::port::maybe_uninit::*;

pub fn generate_array_n<T, const N: usize, Generator: FnMut(usize) -> T>(
    mut generate: Generator,
) -> [T; N] {
    let mut data: [std::mem::MaybeUninit<T>; N] = uninit_array::<T, N>();
    let mut i = 0;
    while i < N {
        data[i].write(generate(i));
        i += 1;
    }
    unsafe { array_assume_init(data) }
}
