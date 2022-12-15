use crate::container::linked_vector::*;
use crate::fe::diag_reporter::*;
use crate::fe::diagnostic_types::*;
use crate::port::allocator::*;
use crate::port::maybe_uninit::*;

pub struct BufferingDiagReporter<'alloc> {
    allocator: &'alloc dyn Allocator,
    diagnostics: std::cell::UnsafeCell<LinkedVector<'alloc, AnyDiag>>,
}

impl<'alloc> BufferingDiagReporter<'alloc> {
    pub fn new(allocator: &'alloc dyn Allocator) -> BufferingDiagReporter<'alloc> {
        BufferingDiagReporter {
            allocator: allocator,
            diagnostics: std::cell::UnsafeCell::new(LinkedVector::new(allocator)),
        }
    }

    pub fn copy_into(&self, other: &dyn DiagReporter) {
        unsafe { &mut *self.diagnostics.get() }.for_each(|diag: &AnyDiag| {
            // TODO(strager): Make report_impl accept a const pointer to reduce casting.
            other.report_impl(diag.type_, &diag.diag as *const _ as *mut u8);
        });
    }

    pub fn move_into(&mut self, other: &dyn DiagReporter) {
        self.copy_into(other);
    }
}

impl<'alloc> DiagReporter for BufferingDiagReporter<'alloc> {
    fn report_impl(&self, type_: DiagType, diag: *const u8) {
        let mut diag_data = [std::mem::MaybeUninit::uninit(); MAX_SIZE_OF_DIAGNOSTIC_TYPE];
        let diag_byte_size: usize = unsafe { *DIAG_SIZES.get_unchecked(type_ as usize) as usize };
        unsafe {
            write_slice(
                &mut diag_data[0..diag_byte_size],
                std::slice::from_raw_parts(diag, diag_byte_size),
            );
        }
        unsafe { &mut *self.diagnostics.get() }.push(AnyDiag {
            type_: type_,
            diag: diag_data,
        });
    }
}

struct AnyDiag {
    type_: DiagType,
    diag: [std::mem::MaybeUninit<u8>; MAX_SIZE_OF_DIAGNOSTIC_TYPE],
}
