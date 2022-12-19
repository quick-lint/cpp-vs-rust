use crate::port::ptr::*;
use crate::qljs_assert;

pub trait Alignable {
    fn align_up(self, alignment: usize) -> Self;
}

impl Alignable for usize {
    fn align_up(self, alignment: usize) -> usize {
        qljs_assert!(alignment.is_power_of_two());
        (self + alignment - 1) & !(alignment - 1)
    }
}

impl<T> Alignable for *const T {
    fn align_up(self, alignment: usize) -> *const T {
        let original: usize = self as usize;
        let aligned: usize = original.align_up(alignment) as usize;
        qljs_assert!(aligned >= original);
        byte_add(self, aligned - original)
    }
}

impl<T> Alignable for *mut T {
    fn align_up(self, alignment: usize) -> *mut T {
        self.cast_const().align_up(alignment).cast_mut()
    }
}
