#[cfg(feature = "qljs_debug")]
use cpp_vs_rust_util::qljs_always_assert;

use cpp_vs_rust_port::allocator::*;
use cpp_vs_rust_util::align::*;
use cpp_vs_rust_util::narrow_cast::*;
use cpp_vs_rust_util::qljs_assert;
use cpp_vs_rust_util::qljs_const_assert;
use cpp_vs_rust_util::qljs_slow_assert;

pub trait BumpAllocatorLike {
    fn allocate_uninitialized_array<'alloc, T>(
        &'alloc self,
        len: usize,
    ) -> &'alloc mut [std::mem::MaybeUninit<T>];

    fn try_grow_array_in_place<T>(&self, array: &mut &mut [T], new_len: usize) -> bool;

    unsafe fn deallocate(&self, p: *mut u8, bytes: usize, align: usize);
}

const fn default_chunk_size<const ALIGNMENT: usize>() -> usize {
    4096 - std::mem::size_of::<ChunkHeader<ALIGNMENT>>()
}

pub struct LinkedBumpAllocator<const ALIGNMENT: usize> {
    state: std::cell::UnsafeCell<LinkedBumpAllocatorState<ALIGNMENT>>,
}

struct LinkedBumpAllocatorState<const ALIGNMENT: usize> {
    chunk: *mut ChunkHeader<ALIGNMENT>,
    next_allocation: *mut u8,
    chunk_end: *mut u8,
    #[cfg(feature = "qljs_debug")]
    disabled_count: i32,
}

#[derive(Clone)]
pub struct LinkedBumpAllocatorRewindState {
    chunk: *mut u8, // ChunkHeader<ALIGNMENT>
    next_allocation: *mut u8,
    chunk_end: *mut u8,
}

impl<const ALIGNMENT: usize> LinkedBumpAllocator<ALIGNMENT> {
    pub fn new(_debug_owner: &'static str) -> LinkedBumpAllocator<ALIGNMENT> {
        LinkedBumpAllocator {
            state: std::cell::UnsafeCell::new(LinkedBumpAllocatorState {
                chunk: std::ptr::null_mut(),
                next_allocation: std::ptr::null_mut(),
                chunk_end: std::ptr::null_mut(),
                #[cfg(feature = "qljs_debug")]
                disabled_count: 0,
            }),
        }
    }

    pub unsafe fn release(&self) {
        unsafe { &mut *self.state.get() }.release();
    }

    pub fn prepare_for_rewind(&self) -> LinkedBumpAllocatorRewindState {
        unsafe { &mut *self.state.get() }.prepare_for_rewind()
    }

    pub unsafe fn rewind(&self, r: LinkedBumpAllocatorRewindState) {
        (*self.state.get()).rewind(r);
    }

    pub fn new_object<T: Sized>(&self, value: T) -> *mut T {
        qljs_const_assert!(
            std::mem::align_of::<T>() <= ALIGNMENT,
            "T is not allowed by this allocator; this allocator's alignment is insufficient for T",
        );
        let byte_size: usize = Self::align_up(std::mem::size_of::<T>());
        unsafe {
            let result_raw: *mut T = self.allocate_bytes(byte_size) as *mut T;
            std::ptr::write(result_raw, value);
            result_raw
        }
    }

    pub fn remaining_bytes_in_current_chunk(&self) -> usize {
        unsafe { &*self.state.get() }.remaining_bytes_in_current_chunk()
    }

    fn align_up(size: usize) -> usize {
        size.align_up(ALIGNMENT)
    }

    /// After calling disable, be sure to call enable before allocating more memory.
    #[cfg(feature = "qljs_debug")]
    pub fn disable(&self) {
        unsafe { &mut *self.state.get() }.disabled_count += 1;
    }

    /// Call only after calling disable.
    #[cfg(feature = "qljs_debug")]
    pub fn enable(&self) {
        unsafe { &mut *self.state.get() }.disabled_count -= 1;
    }

    fn allocate_bytes(&self, size: usize) -> *mut u8 {
        unsafe { &mut *self.state.get() }.allocate_bytes(size)
    }

    unsafe fn deallocate_bytes(&self, _p: *mut u8, _size: usize) {
        // TODO(strager): Mark memory as unallocated for Valgrind and ASAN.
    }

    pub fn allocate(&self, bytes: usize, align: usize) -> *mut u8 {
        qljs_assert!(align <= ALIGNMENT);
        self.allocate_bytes(Self::align_up(bytes))
    }
}

impl<const ALIGNMENT: usize> BumpAllocatorLike for LinkedBumpAllocator<ALIGNMENT> {
    fn allocate_uninitialized_array<'alloc, T>(
        &'alloc self,
        len: usize,
    ) -> &'alloc mut [std::mem::MaybeUninit<T>] {
        qljs_const_assert!(
            std::mem::align_of::<T>() <= ALIGNMENT,
            "T is not allowed by this allocator; this allocator's alignment is insufficient for T",
        );
        let byte_size: usize = Self::align_up(len * std::mem::size_of::<T>());
        let data = self.allocate_bytes(byte_size) as *mut std::mem::MaybeUninit<T>;
        unsafe { std::slice::from_raw_parts_mut(data, len) }
    }

    // TODO(port-later): Should this accept/return MaybeUninit?
    fn try_grow_array_in_place<T>(&self, array: &mut &mut [T], new_len: usize) -> bool {
        unsafe { &mut *self.state.get() }.try_grow_array_in_place(array, new_len)
    }

    unsafe fn deallocate(&self, p: *mut u8, bytes: usize, align: usize) {
        qljs_assert!(align <= ALIGNMENT);
        self.deallocate_bytes(p, bytes);
    }
}

impl<const ALIGNMENT: usize> LinkedBumpAllocatorState<ALIGNMENT> {
    fn release(&mut self) {
        let mut c = self.chunk;
        while !c.is_null() {
            unsafe {
                let previous = c.as_ref().unwrap().previous;
                ChunkHeader::<ALIGNMENT>::delete_chunk(c);
                c = previous;
            }
        }
        self.chunk = std::ptr::null_mut();
        self.next_allocation = std::ptr::null_mut();
        self.chunk_end = std::ptr::null_mut();
    }

    fn prepare_for_rewind(&mut self) -> LinkedBumpAllocatorRewindState {
        LinkedBumpAllocatorRewindState {
            chunk: self.chunk as *mut u8,
            next_allocation: self.next_allocation,
            chunk_end: self.chunk_end,
        }
    }

    unsafe fn rewind(&mut self, r: LinkedBumpAllocatorRewindState) {
        let r_chunk: *mut ChunkHeader<ALIGNMENT> = r.chunk as *mut ChunkHeader<ALIGNMENT>;
        let allocated_new_chunk = self.chunk != r_chunk;
        if allocated_new_chunk {
            // If we rewound to exactly where we were before, we might rewind near the
            // end of a chunk. Allocations would soon need a new chunk.
            //
            // Avoid straddling near the end of a chunk by using a new chunk (which
            // was already allocated).
            //
            // TODO(strager): Should we use the *oldest* chunk or the *newest* chunk?
            // Here we pick the *oldest* chunk.
            let mut c: *mut ChunkHeader<ALIGNMENT> = self.chunk;
            qljs_assert!(!c.is_null());
            while c.as_ref().unwrap().previous != r_chunk {
                let previous = c.as_ref().unwrap().previous;
                ChunkHeader::<ALIGNMENT>::delete_chunk(c);
                c = previous;
                qljs_assert!(!c.is_null());
            }
            self.chunk = c;
            self.next_allocation = Self::data_begin(c);
            self.chunk_end = Self::data_end(c);
        } else {
            self.chunk = r_chunk;
            self.next_allocation = r.next_allocation;
            self.chunk_end = r.chunk_end;
        }
        self.did_deallocate_bytes(
            self.next_allocation,
            self.remaining_bytes_in_current_chunk(),
        );
    }

    fn try_grow_array_in_place<T>(&mut self, array: &mut &mut [T], new_len: usize) -> bool {
        unsafe {
            self.assert_not_disabled();
            let old_array: &mut [T] = *array;
            qljs_assert!(new_len > old_array.len());
            let old_byte_size = Self::align_up(old_array.len() * std::mem::size_of::<T>());
            let old_array_end: *mut u8 = (old_array.as_mut_ptr() as *mut u8).add(old_byte_size);
            let array_is_last_allocation = old_array_end == self.next_allocation;
            if !array_is_last_allocation {
                // We can't grow because something else was already allocated.
                return false;
            }

            let extra_bytes =
                Self::align_up((new_len - old_array.len()) * std::mem::size_of::<T>());
            if extra_bytes > self.remaining_bytes_in_current_chunk() {
                return false;
            }
            // NOTE(strager): new_array_begin should be the same as old_array.as_ptr(). However, we
            // need to create a new pointer based on self.new_allocation. Otherwise, Miri thinks
            // that the new slice is created out of bounds.
            let new_array_begin: *mut T = self.next_allocation.sub(old_byte_size) as *mut T;
            qljs_assert!(new_array_begin == old_array.as_mut_ptr());

            self.did_allocate_bytes(self.next_allocation, extra_bytes);
            self.next_allocation = self.next_allocation.add(extra_bytes);
            *array = std::slice::from_raw_parts_mut(new_array_begin, new_len);
            true
        }
    }

    pub fn remaining_bytes_in_current_chunk(&self) -> usize {
        // NOTE(strager): offset_from is UB if next_allocator or chunk_end is null.
        if self.next_allocation.is_null() {
            0
        } else {
            unsafe { narrow_cast(self.chunk_end.offset_from(self.next_allocation)) }
        }
    }

    fn allocate_bytes(&mut self, size: usize) -> *mut u8 {
        unsafe {
            self.assert_not_disabled();
            qljs_slow_assert!(size % ALIGNMENT == 0);
            if self.remaining_bytes_in_current_chunk() < size {
                self.append_chunk(std::cmp::max(size, default_chunk_size::<ALIGNMENT>()));
                qljs_assert!(self.remaining_bytes_in_current_chunk() >= size);
            }
            let result = self.next_allocation;
            self.next_allocation = self.next_allocation.add(size);
            self.did_allocate_bytes(result, size);
            result
        }
    }

    fn did_allocate_bytes(&self, _p: *mut u8, _size: usize) {
        // TODO(strager): Mark memory as usable for Valgrind.
        // TODO(port-later): Mark memory as usable for ASAN.
    }

    fn did_deallocate_bytes(&self, _p: *mut u8, _size: usize) {
        // TODO(strager): Mark memory as unusable for Valgrind.
        // TODO(port-later): Mark memory as unusable for Valgrind.
    }

    fn append_chunk(&mut self, len: usize) {
        self.chunk = ChunkHeader::new_chunk(len, self.chunk);
        self.next_allocation = Self::data_begin(self.chunk);
        self.chunk_end = Self::data_end(self.chunk);
    }

    fn assert_not_disabled(&self) {
        #[cfg(feature = "qljs_debug")]
        qljs_always_assert!(!self.is_disabled());
    }

    #[cfg(feature = "qljs_debug")]
    fn is_disabled(&self) -> bool {
        self.disabled_count > 0
    }

    fn align_up(size: usize) -> usize {
        LinkedBumpAllocator::<ALIGNMENT>::align_up(size)
    }

    fn data_begin(chunk: *mut ChunkHeader<ALIGNMENT>) -> *mut u8 {
        unsafe { chunk.offset(1).align_up(ALIGNMENT) as *mut u8 }
    }

    fn data_end(chunk: *mut ChunkHeader<ALIGNMENT>) -> *mut u8 {
        unsafe { Self::data_begin(chunk).add((*chunk).len) }
    }
}

// TODO(port-later): Do we need repr(C)? We pack ChunkHeader and the data in a single allocation,
// so I thought I'd add repr(C) just to be safe.
#[repr(C)]
struct ChunkHeader<const ALIGNMENT: usize> {
    previous: *mut ChunkHeader<ALIGNMENT>, // Linked list.
    len: usize,                            // Size of the data portion in bytes.
}

impl<const ALIGNMENT: usize> ChunkHeader<ALIGNMENT> {
    fn allocation_layout(len: usize) -> std::alloc::Layout {
        std::alloc::Layout::from_size_align(
            std::mem::size_of::<Self>().align_up(ALIGNMENT) + len,
            std::cmp::max(ALIGNMENT, std::mem::align_of::<Self>()),
        )
        .unwrap()
    }

    fn new_chunk(len: usize, previous: *mut Self) -> *mut Self {
        unsafe {
            let chunk = std::alloc::alloc(Self::allocation_layout(len)) as *mut Self;
            std::ptr::write(
                chunk,
                Self {
                    previous: previous,
                    len: len,
                },
            );
            chunk
        }
    }

    unsafe fn delete_chunk(c: *mut ChunkHeader<ALIGNMENT>) {
        unsafe {
            let len = c.as_ref().unwrap().len;
            std::ptr::drop_in_place(c);
            std::alloc::dealloc(c as *mut u8, Self::allocation_layout(len));
        }
    }
}

impl<const ALIGNMENT: usize> Drop for LinkedBumpAllocator<ALIGNMENT> {
    fn drop(&mut self) {
        unsafe {
            self.release();
        }
    }
}

impl<const ALIGNMENT: usize> Allocator for LinkedBumpAllocator<ALIGNMENT> {
    fn allocate(&self, layout: std::alloc::Layout) -> Result<std::ptr::NonNull<[u8]>, AllocError> {
        unsafe {
            // TODO(port-later): Does an allocation size of 0 work?
            let result: *mut u8 = self.allocate(layout.size(), layout.align());
            Ok(std::ptr::NonNull::new_unchecked(
                std::ptr::slice_from_raw_parts_mut(result, layout.size()),
            ))
        }
    }

    unsafe fn deallocate(&self, ptr: std::ptr::NonNull<u8>, layout: std::alloc::Layout) {
        self.deallocate_bytes(ptr.as_ptr(), layout.size());
    }
}
