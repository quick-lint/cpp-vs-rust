#[cfg(feature = "qljs_debug")]
use crate::qljs_always_assert;

use crate::port::allocator::*;
use crate::qljs_assert;
use crate::qljs_const_assert;
use crate::qljs_slow_assert;
use crate::util::narrow_cast::*;

pub trait BumpAllocatorLike {
    fn allocate_uninitialized_array<'b, T>(&self, len: usize)
        -> &'b mut [std::mem::MaybeUninit<T>];

    fn try_grow_array_in_place<'b, T>(
        &self,
        array: &'b mut [T],
        new_len: usize,
    ) -> Option<&'b mut [T]>;

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

    // TODO(port): make_rewind_guard

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
        (size + ALIGNMENT - 1) & !(ALIGNMENT - 1)
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
    fn allocate_uninitialized_array<'b, T>(
        &self,
        len: usize,
    ) -> &'b mut [std::mem::MaybeUninit<T>] {
        qljs_const_assert!(
            std::mem::align_of::<T>() <= ALIGNMENT,
            "T is not allowed by this allocator; this allocator's alignment is insufficient for T",
        );
        let byte_size: usize = Self::align_up(len * std::mem::size_of::<T>());
        let data = self.allocate_bytes(byte_size) as *mut std::mem::MaybeUninit<T>;
        unsafe { std::slice::from_raw_parts_mut(data, len) }
    }

    // TODO(port-later): Should this accept MaybeUninit?
    // TODO(port-later): Should this return MaybeUninit?
    fn try_grow_array_in_place<'b, T>(
        &self,
        array: &'b mut [T],
        new_len: usize,
    ) -> Option<&'b mut [T]> {
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
            self.next_allocation = c.as_mut().unwrap().data_begin();
            self.chunk_end = c.as_mut().unwrap().data_end();
        } else {
            self.chunk = r_chunk;
            self.next_allocation = r.next_allocation;
            self.chunk_end = r.chunk_end;
        }
        self.did_deallocate_bytes(
            self.next_allocation,
            narrow_cast(self.chunk_end.offset_from(self.next_allocation)),
        );
    }

    fn try_grow_array_in_place<'b, T>(
        &mut self,
        array: &'b mut [T],
        new_len: usize,
    ) -> Option<&'b mut [T]> {
        unsafe {
            self.assert_not_disabled();
            qljs_assert!(new_len > array.len());
            let old_byte_size = Self::align_up(array.len() * std::mem::size_of::<T>());
            let old_array_end: *mut u8 = (array.as_mut_ptr() as *mut u8).add(old_byte_size);
            let array_is_last_allocation = old_array_end == self.next_allocation;
            if !array_is_last_allocation {
                // We can't grow because something else was already allocated.
                return None;
            }

            let extra_bytes = Self::align_up((new_len - array.len()) * std::mem::size_of::<T>());
            if extra_bytes > self.remaining_bytes_in_current_chunk() {
                return None;
            }
            self.did_allocate_bytes(self.next_allocation, extra_bytes);
            self.next_allocation = self.next_allocation.add(extra_bytes);
            Some(std::slice::from_raw_parts_mut(array.as_mut_ptr(), new_len))
        }
    }

    pub fn remaining_bytes_in_current_chunk(&self) -> usize {
        unsafe { narrow_cast(self.chunk_end.offset_from(self.next_allocation)) }
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
        let chunk: &mut ChunkHeader<ALIGNMENT> = unsafe { self.chunk.as_mut().unwrap() };
        self.next_allocation = chunk.data_begin();
        self.chunk_end = chunk.data_end();
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
}

// TODO(port-later): Do we need repr(C)? We pack ChunkHeader and the data in a single allocation,
// so I thought I'd add repr(C) just to be safe.
#[repr(C)]
struct ChunkHeader<const ALIGNMENT: usize> {
    previous: *mut ChunkHeader<ALIGNMENT>, // Linked list.
    len: usize,                            // Size of the data portion in bytes.
}

impl<const ALIGNMENT: usize> ChunkHeader<ALIGNMENT> {
    fn data_begin(&mut self) -> *mut u8 {
        // FIXME(port): Data is not guaranteed to be aligned!
        unsafe { (self as *mut Self).offset(1) as *mut u8 }
    }

    fn data_end<'a>(&'a mut self) -> *mut u8 {
        unsafe { self.data_begin().add(self.len) }
    }

    fn allocation_layout(len: usize) -> std::alloc::Layout {
        std::alloc::Layout::from_size_align(
            std::mem::size_of::<Self>() + len,
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
