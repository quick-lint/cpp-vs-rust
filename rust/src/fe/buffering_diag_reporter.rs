use crate::container::linked_vector::*;
use crate::fe::diag_reporter::*;
use crate::fe::diagnostic_types::*;
use crate::port::allocator::*;
use crate::port::maybe_uninit::*;
use crate::qljs_const_assert;

pub struct BufferingDiagReporter<'alloc, 'code> {
    diagnostics: std::cell::UnsafeCell<LinkedVector<'alloc, StoredDiag<'code>>>,
}

impl<'alloc, 'code> BufferingDiagReporter<'alloc, 'code> {
    pub fn new(allocator: &'alloc dyn Allocator) -> BufferingDiagReporter<'alloc, 'code> {
        BufferingDiagReporter {
            diagnostics: std::cell::UnsafeCell::new(LinkedVector::new(allocator)),
        }
    }

    pub fn copy_into(&self, other: &dyn DiagReporter) {
        unsafe { &mut *self.diagnostics.get() }.for_each(|diag: &StoredDiag| {
            // TODO(strager): Make report_impl accept a const pointer to reduce casting.
            other.report_impl(diag.type_, &diag.diag.data as *const _ as *mut u8);
        });
    }

    pub fn move_into(&mut self, other: &dyn DiagReporter) {
        self.copy_into(other);
    }

    pub fn is_empty(&self) -> bool {
        unsafe { &mut *self.diagnostics.get() }.is_empty()
    }

    pub fn clear(&mut self) {
        unsafe { &mut *self.diagnostics.get() }.clear()
    }
}

impl<'alloc, 'code> DiagReporter for BufferingDiagReporter<'alloc, 'code> {
    fn report_impl(&self, type_: DiagType, diag: *const u8) {
        let mut diag_data = [std::mem::MaybeUninit::uninit(); MAX_SIZE_OF_DIAGNOSTIC_TYPE];
        let diag_byte_size: usize = unsafe { *DIAG_SIZES.get_unchecked(type_ as usize) as usize };
        unsafe {
            write_slice(
                &mut diag_data[0..diag_byte_size],
                std::slice::from_raw_parts(diag, diag_byte_size),
            );
        }
        unsafe { &mut *self.diagnostics.get() }.push(StoredDiag {
            type_: type_,
            diag: StoredDiagData {
                data: diag_data,
                phantom: std::marker::PhantomData,
            },
        });
    }
}

struct StoredDiag<'code> {
    type_: DiagType,
    diag: StoredDiagData<'code>,
}

#[repr(align(8))]
struct StoredDiagData<'code> {
    data: [std::mem::MaybeUninit<u8>; MAX_SIZE_OF_DIAGNOSTIC_TYPE],
    phantom: std::marker::PhantomData<&'code u8>,
}

qljs_const_assert!(
    std::mem::align_of::<StoredDiagData>() >= std::mem::align_of::<AnyDiag>(),
    "StoredDiag should be aligned such that it can be cast to any diagnostic struct",
);
