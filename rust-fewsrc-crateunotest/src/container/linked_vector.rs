use crate::port::allocator::*;
use crate::qljs_assert;
use crate::qljs_slow_assert;
use crate::util::align::*;

// A linked list of arrays. Optimized for appending then iterating.
//
// Guarantees:
//
// * Items are ordered by insertion (like std::vec::Vec and std::collections::VecDeque when only
//   calling push).
// * Items are never copied or moved when adding or removing different items. Pointer stability.
//   * TODO(port-later): Is pointer stability important in the Rust port?
pub struct LinkedVector<'alloc, T> {
    head: *mut ChunkHeader<T>,
    tail: *mut ChunkHeader<T>,
    allocator: &'alloc dyn Allocator,
}

fn items_per_chunk<T>() -> usize {
    std::cmp::max(
        1,
        (DEFAULT_CHUNK_BYTE_SIZE - std::mem::size_of::<usize>() * 3) / std::mem::size_of::<T>(),
    )
}

impl<'alloc, T> LinkedVector<'alloc, T> {
    const ALIGNMENT: usize = std::mem::align_of::<T>();

    pub fn new(allocator: &'alloc dyn Allocator) -> LinkedVector<'alloc, T> {
        LinkedVector {
            head: std::ptr::null_mut(),
            tail: std::ptr::null_mut(),
            allocator: allocator,
        }
    }

    pub fn items_per_chunk(&self) -> usize {
        items_per_chunk::<T>()
    }

    pub fn push(&mut self, value: T) -> &mut T {
        unsafe {
            let mut c: *mut ChunkHeader<T> = self.tail;
            if c.is_null() || (*c).item_count == ChunkHeader::<T>::capacity() {
                c = self.append_new_chunk_slow();
            }
            let item: &mut std::mem::MaybeUninit<T> = Self::slot(c, (*c).item_count);
            item.write(value);
            (*c).item_count += 1;
            item.assume_init_mut()
        }
    }

    pub fn pop(&mut self) {
        unsafe {
            qljs_assert!(!self.is_empty());
            let c: *mut ChunkHeader<T> = self.tail;
            let item: &mut std::mem::MaybeUninit<T> = Self::slot(c, (*c).item_count - 1);
            item.assume_init_drop();
            (*c).item_count -= 1;
            if (*c).item_count == 0 {
                self.remove_tail_chunk_slow();
            }
        }
    }

    pub fn clear(&mut self) {
        unsafe {
            let mut c: *mut ChunkHeader<T> = self.head;
            while !c.is_null() {
                let next: *mut ChunkHeader<T> = (*c).next;
                for i in 0..(*c).item_count {
                    Self::slot(c, i).assume_init_drop();
                }
                self.drop_and_deallocate_chunk(c);
                c = next;
            }
            self.head = std::ptr::null_mut();
            self.tail = std::ptr::null_mut();
        }
    }

    pub fn is_empty(&self) -> bool {
        self.head.is_null()
    }

    pub fn back(&self) -> &T {
        unsafe {
            qljs_assert!(!self.is_empty());
            Self::items(self.tail).last().unwrap_unchecked()
        }
    }

    pub fn for_each<Func: FnMut(&T)>(&self, mut func: Func) {
        unsafe {
            let mut c: *mut ChunkHeader<T> = self.head;
            while !c.is_null() {
                for item in Self::items(c) {
                    func(item);
                }
                c = (*c).next;
            }
        }
    }

    #[inline(never)]
    fn append_new_chunk_slow(&mut self) -> *mut ChunkHeader<T> {
        unsafe {
            let c: *mut ChunkHeader<T> = {
                let layout = ChunkHeader::<T>::layout();
                match self.allocator.allocate(layout) {
                    Err(_) => {
                        std::alloc::handle_alloc_error(layout);
                    }
                    Ok(raw) => {
                        let raw = raw.as_ptr() as *mut ChunkHeader<T>;
                        std::ptr::write(raw, ChunkHeader::<T>::new());
                        raw
                    }
                }
            };

            if self.head.is_null() {
                self.head = c;
            } else {
                (*self.tail).next = c;
            }
            (*c).prev = self.tail;
            self.tail = c;
            c
        }
    }

    #[inline(never)]
    fn remove_tail_chunk_slow(&mut self) {
        unsafe {
            let old_tail: *mut ChunkHeader<T> = self.tail;
            qljs_assert!(!old_tail.is_null());
            qljs_assert!((*old_tail).item_count == 0);

            let new_tail: *mut ChunkHeader<T> = (*old_tail).prev;
            qljs_assert!(new_tail.is_null() == (self.head == self.tail));
            self.drop_and_deallocate_chunk(old_tail);
            if new_tail.is_null() {
                // We deallocated the only chunk.
                self.head = std::ptr::null_mut();
                self.tail = std::ptr::null_mut();
            } else {
                (*new_tail).next = std::ptr::null_mut();
                self.tail = new_tail;
            }
        }
    }

    unsafe fn drop_and_deallocate_chunk(&self, chunk: *mut ChunkHeader<T>) {
        std::ptr::drop_in_place(chunk);
        self.allocator.deallocate(
            std::ptr::NonNull::new_unchecked(chunk as *mut u8),
            ChunkHeader::<T>::layout(),
        );
    }

    unsafe fn items<'a>(chunk: *const ChunkHeader<T>) -> &'a [T] {
        std::slice::from_raw_parts(Self::data_begin(chunk) as *const T, (*chunk).item_count)
    }

    unsafe fn slot<'a>(
        chunk: *mut ChunkHeader<T>,
        index: usize,
    ) -> &'a mut std::mem::MaybeUninit<T> {
        qljs_slow_assert!(index < ChunkHeader::<T>::capacity());
        &mut *Self::data_begin_mut(chunk).add(index)
    }

    fn data_begin(chunk: *const ChunkHeader<T>) -> *const std::mem::MaybeUninit<T> {
        unsafe { chunk.offset(1).align_up(Self::ALIGNMENT) as *const std::mem::MaybeUninit<T> }
    }

    fn data_begin_mut(chunk: *mut ChunkHeader<T>) -> *mut std::mem::MaybeUninit<T> {
        unsafe { chunk.offset(1).align_up(Self::ALIGNMENT) as *mut std::mem::MaybeUninit<T> }
    }
}

impl<'alloc, T> Drop for LinkedVector<'alloc, T> {
    fn drop(&mut self) {
        self.clear();
    }
}

const DEFAULT_CHUNK_BYTE_SIZE: usize = 4096;

// NOTE(port): This design is different from the C++ design. C++ had an array whose size was
// dependant on T. Rust doesn't allow that, so we need to do allocator tricks (similar to
// LinkedBumpAllocator).
struct ChunkHeader<T> {
    prev: *mut ChunkHeader<T>,
    next: *mut ChunkHeader<T>,
    item_count: usize,
}

impl<T> ChunkHeader<T> {
    fn new() -> ChunkHeader<T> {
        ChunkHeader {
            prev: std::ptr::null_mut(),
            next: std::ptr::null_mut(),
            item_count: 0,
        }
    }

    fn layout() -> std::alloc::Layout {
        std::alloc::Layout::from_size_align(
            std::mem::size_of::<Self>().align_up(std::mem::align_of::<T>())
                + std::mem::size_of::<T>() * Self::capacity(),
            std::cmp::max(std::mem::align_of::<T>(), std::mem::align_of::<Self>()),
        )
        .unwrap()
    }

    fn capacity() -> usize {
        items_per_chunk::<T>()
    }
}
