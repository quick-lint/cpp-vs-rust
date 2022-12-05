use crate::port::allocator::*;
use crate::qljs_assert;
use crate::qljs_slow_assert;

// A linked list of arrays. Optimized for appending then iterating.
//
// Guarantees:
//
// * Items are ordered by insertion (like std::vector and std::deque when only
//   calling emplace_back).
// * Items are never copied or moved when adding or removing different items
//   (like std::deque). Pointer stability.
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

    // TODO(port): Rename to 'push'.
    pub fn emplace_back(&mut self, value: T) -> &mut T {
        unsafe {
            let mut c: *mut ChunkHeader<T> = self.tail;
            if c.is_null() || (&*c).item_count == ChunkHeader::<T>::capacity() {
                c = self.append_new_chunk_slow();
            }
            let item: &mut std::mem::MaybeUninit<T> = (&mut *c).slot((&mut *c).item_count);
            item.write(value);
            (&mut *c).item_count += 1;
            item.assume_init_mut()
        }
    }

    pub fn pop_back(&mut self) {
        unsafe {
            qljs_assert!(!self.empty());
            let c: &mut ChunkHeader<T> = &mut *self.tail;
            let item: &mut std::mem::MaybeUninit<T> = c.slot(c.item_count - 1);
            item.assume_init_drop();
            c.item_count -= 1;
            if c.item_count == 0 {
                self.remove_tail_chunk_slow();
            }
        }
    }

    pub fn clear(&mut self) {
        unsafe {
            let mut c: *mut ChunkHeader<T> = self.head;
            while !c.is_null() {
                let next: *mut ChunkHeader<T> = (&*c).next;
                for i in 0..(&*c).item_count {
                    (&mut *c).slot(i).assume_init_drop();
                }
                self.drop_and_deallocate_chunk(c);
                c = next;
            }
            self.head = std::ptr::null_mut();
            self.tail = std::ptr::null_mut();
        }
    }

    pub fn empty(&self) -> bool {
        self.head.is_null()
    }

    pub fn back(&self) -> &T {
        unsafe {
            qljs_assert!(!self.empty());
            (&*self.tail).items().last().unwrap_unchecked()
        }
    }

    pub fn for_each<Func: FnMut(&T)>(&self, mut func: Func) {
        unsafe {
            let mut c: *mut ChunkHeader<T> = self.head;
            while !c.is_null() {
                for item in (&*c).items() {
                    func(item);
                }
                c = (&*c).next;
            }
        }
    }

    // TODO(port): noinline
    fn append_new_chunk_slow(&mut self) -> *mut ChunkHeader<T> {
        unsafe {
            let c: &mut ChunkHeader<T> = {
                let layout = ChunkHeader::<T>::layout();
                match self.allocator.allocate(layout) {
                    Err(_) => {
                        std::alloc::handle_alloc_error(layout);
                    }
                    Ok(raw) => {
                        let raw = raw.as_ptr() as *mut ChunkHeader<T>;
                        std::ptr::write(raw, ChunkHeader::<T>::new());
                        &mut *raw
                    }
                }
            };

            if self.head.is_null() {
                self.head = c;
            } else {
                (&mut *self.tail).next = c;
            }
            c.prev = self.tail;
            self.tail = c;
            c
        }
    }

    // TODO(port): noinline
    fn remove_tail_chunk_slow(&mut self) {
        unsafe {
            let old_tail: *mut ChunkHeader<T> = self.tail;
            qljs_assert!(!old_tail.is_null());
            qljs_assert!((&*old_tail).item_count == 0);

            let new_tail: *mut ChunkHeader<T> = (&*old_tail).prev;
            qljs_assert!(new_tail.is_null() == (self.head == self.tail));
            self.drop_and_deallocate_chunk(old_tail);
            if new_tail.is_null() {
                // We deallocated the only chunk.
                self.head = std::ptr::null_mut();
                self.tail = std::ptr::null_mut();
            } else {
                (&mut *new_tail).next = std::ptr::null_mut();
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
            std::mem::size_of::<Self>() + std::mem::size_of::<T>() * Self::capacity(),
            std::cmp::max(std::mem::align_of::<T>(), std::mem::align_of::<Self>()),
        )
        .unwrap()
    }

    fn items(&self) -> &[T] {
        unsafe { std::slice::from_raw_parts(self.data_begin() as *const T, self.item_count) }
    }

    fn slot(&mut self, index: usize) -> &mut std::mem::MaybeUninit<T> {
        qljs_slow_assert!(index < Self::capacity());
        unsafe { &mut *self.data_begin_mut().offset(index as isize) }
    }

    fn data_begin(&self) -> *const std::mem::MaybeUninit<T> {
        // FIXME(port): Data is not guaranteed to be aligned!
        unsafe { (self as *const Self).offset(1) as *const std::mem::MaybeUninit<T> }
    }

    fn data_begin_mut(&mut self) -> *mut std::mem::MaybeUninit<T> {
        // FIXME(port): Data is not guaranteed to be aligned!
        unsafe { (self as *mut Self).offset(1) as *mut std::mem::MaybeUninit<T> }
    }

    fn capacity() -> usize {
        items_per_chunk::<T>()
    }
}
