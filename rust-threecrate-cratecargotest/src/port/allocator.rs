// Backport of Rust's std::alloc::Allocator.
pub trait Allocator {
    fn allocate(&self, layout: std::alloc::Layout) -> Result<std::ptr::NonNull<[u8]>, AllocError>;
    unsafe fn deallocate(&self, ptr: std::ptr::NonNull<u8>, layout: std::alloc::Layout);
}

pub struct AllocError;

struct GlobalAllocator;
impl Allocator for GlobalAllocator {
    fn allocate(&self, layout: std::alloc::Layout) -> Result<std::ptr::NonNull<[u8]>, AllocError> {
        unsafe {
            if layout.size() == 0 {
                // std::alloc::alloc is not guaranteed to support a zero size. Create a fake
                // non-null pointer which is appropriately aligned.
                let result: *mut u8 = layout.align() as *mut u8;
                return Ok(std::ptr::NonNull::new_unchecked(
                    std::ptr::slice_from_raw_parts_mut(result, 0),
                ));
            }
            let result: *mut u8 = std::alloc::alloc(layout);
            if result.is_null() {
                return Err(AllocError);
            }
            Ok(std::ptr::NonNull::new_unchecked(
                std::ptr::slice_from_raw_parts_mut(result, layout.size()),
            ))
        }
    }

    unsafe fn deallocate(&self, ptr: std::ptr::NonNull<u8>, layout: std::alloc::Layout) {
        if layout.size() == 0 {
            // std::alloc::dealloc is not guaranteed to support a zero size.
            // TODO(strager): Assert that ptr came from self.allocate.
            return;
        }
        std::alloc::dealloc(ptr.as_ptr(), layout);
    }
}

static GLOBAL_ALLOCATOR_SINGLETON: GlobalAllocator = GlobalAllocator;

pub fn global_allocator() -> &'static impl Allocator {
    &GLOBAL_ALLOCATOR_SINGLETON
}
