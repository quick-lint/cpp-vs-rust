use crate::container::linked_bump_allocator::*;
use crate::container::winkable::*;
use crate::util::narrow_cast::*;

// TODO(port): Use InstrumentedVector if vector instrumentation is enabled.
pub type BumpVector<'alloc, T, BumpAllocator> =
    UninstrumentedVector<RawBumpVector<'alloc, T, BumpAllocator>>;

// Wraps a vector class so it has the same interface as
// InstrumentedVector<Vector> (but without the instrumentation overhead).
//
// NOTE(port): We're not porting InstrumentedVector right now.
// TODO(port): Make this interface more like Rust's Vec.
pub struct UninstrumentedVector<Vector>(Vector);

pub trait VectorLike {
    type T;
    type Allocator;

    fn new(allocator: Self::Allocator) -> Self;
    fn empty(&self) -> bool;
    fn size(&self) -> usize;
    fn capacity(&self) -> usize;
    fn as_slice(&self) -> &[Self::T];
    fn reserve(&mut self, size: usize);
    fn push_back(&mut self, value: Self::T);
    fn pop_back(&mut self);
    fn resize(&mut self, new_size: usize)
    where
        Self::T: Default;
}

impl<Vector: VectorLike> UninstrumentedVector<Vector> {
    pub fn new(
        _debug_owner: &'static str,
        allocator: Vector::Allocator,
    ) -> UninstrumentedVector<Vector> {
        UninstrumentedVector(Vector::new(allocator))
    }

    pub fn empty(&self) -> bool {
        self.0.empty()
    }
    pub fn size(&self) -> usize {
        self.0.size()
    }
    pub fn capacity(&self) -> usize {
        self.0.capacity()
    }
    pub fn as_slice(&self) -> &[Vector::T] {
        self.0.as_slice()
    }
    pub fn reserve(&mut self, size: usize) {
        self.0.reserve(size);
    }
    pub fn push_back(&mut self, value: Vector::T) {
        self.0.push_back(value);
    }
    pub fn pop_back(&mut self) {
        self.0.pop_back();
    }
    pub fn resize(&mut self, new_size: usize)
    where
        Vector::T: Default,
    {
        self.0.resize(new_size);
    }

    // TODO(port): Expose more RawBumpVector functions.
}

pub struct RawBumpVector<'alloc, T: Winkable, BumpAllocator: BumpAllocatorLike> {
    data: *mut std::mem::MaybeUninit<T>,
    data_end: *mut std::mem::MaybeUninit<T>,
    capacity_end: *mut std::mem::MaybeUninit<T>,
    allocator: &'alloc BumpAllocator,
}

impl<'alloc, T: Winkable, BumpAllocator: BumpAllocatorLike> VectorLike
    for RawBumpVector<'alloc, T, BumpAllocator>
{
    type T = T;
    type Allocator = &'alloc BumpAllocator;

    fn new(allocator: &'alloc BumpAllocator) -> RawBumpVector<'alloc, T, BumpAllocator> {
        RawBumpVector {
            data: std::ptr::null_mut(),
            data_end: std::ptr::null_mut(),
            capacity_end: std::ptr::null_mut(),
            allocator: allocator,
        }
    }

    fn empty(&self) -> bool {
        self.data == self.data_end
    }

    // TODO(port): Rename to 'len'.
    fn size(&self) -> usize {
        unsafe { narrow_cast(self.data_end.offset_from(self.data)) }
    }

    fn capacity(&self) -> usize {
        unsafe { narrow_cast(self.capacity_end.offset_from(self.data)) }
    }

    fn as_slice(&self) -> &[T] {
        unsafe { std::slice::from_raw_parts(self.data as *const T, self.size()) }
    }

    fn reserve(&mut self, new_capacity: usize) {
        if self.capacity() < new_capacity {
            self.reserve_grow(new_capacity);
        }
    }

    fn push_back(&mut self, value: T) {
        if self.capacity_end == self.data_end {
            self.reserve_grow_by_at_least(1);
        }
        unsafe {
            (&mut *self.data_end).write(value);
            self.data_end = self.data_end.offset(1);
        }
    }

    fn pop_back(&mut self) {
        assert!(!self.empty());
        // NOTE(port): The C++ code didn't destruct, so we don't drop here.
        self.data_end = unsafe { self.data_end.offset(-1) };
    }

    fn resize(&mut self, new_size: usize)
    where
        T: Default,
    {
        unsafe {
            let old_size = self.size();
            if new_size == old_size {
                // Do nothing.
            } else if new_size < old_size {
                let new_end: *mut std::mem::MaybeUninit<T> = self.data.offset(new_size as isize);
                for i in new_size..old_size {
                    (&mut *self.data.offset(i as isize)).assume_init_drop();
                }
                self.data_end = new_end;
            } else {
                let old_capacity = self.capacity();
                if new_size > old_capacity {
                    self.reserve_grow_by_at_least(new_size - old_capacity);
                }
                let new_end: *mut std::mem::MaybeUninit<T> = self.data.offset(new_size as isize);
                for i in old_size..new_size {
                    (&mut *self.data.offset(i as isize)).write(T::default());
                }
                self.data_end = new_end;
            }
        }
    }
}

impl<'alloc, T: Winkable, BumpAllocator: BumpAllocatorLike>
    RawBumpVector<'alloc, T, BumpAllocator>
{
    pub fn get_allocator(&self) -> &'alloc BumpAllocator {
        self.allocator
    }

    pub fn front_mut(&mut self) -> &mut T {
        assert!(!self.empty());
        unsafe { (&mut *self.data).assume_init_mut() }
    }

    pub fn back_mut(&mut self) -> &mut T {
        assert!(!self.empty());
        unsafe { (&mut *self.data_end.offset(-1)).assume_init_mut() }
    }

    pub fn front(&self) -> &T {
        assert!(!self.empty());
        unsafe { (&*self.data).assume_init_ref() }
    }

    pub fn back(&self) -> &T {
        assert!(!self.empty());
        unsafe { (&*self.data_end.offset(-1)).assume_init_ref() }
    }

    pub fn reserve_grow(&mut self, new_capacity: usize) {
        unsafe {
            assert!(new_capacity > self.capacity());
            if self.data.is_null() {
                self.data = self
                    .allocator
                    .allocate_uninitialized_array::<T>(new_capacity)
                    .as_mut_ptr();
                self.data_end = self.data;
                self.capacity_end = self.data.offset(new_capacity as isize);
            } else {
                let old_size = self.size();
                let old_capacity = self.capacity();
                let old_array: &mut [std::mem::MaybeUninit<T>] =
                    std::slice::from_raw_parts_mut(self.data, old_capacity);
                let new_array: Option<&mut [std::mem::MaybeUninit<T>]> = self
                    .allocator
                    .try_grow_array_in_place(old_array, new_capacity);
                match new_array {
                    Some(_new_array) => {
                        self.capacity_end = self.data.offset(new_capacity as isize);
                    }
                    None => {
                        let new_data: &mut [std::mem::MaybeUninit<T>] = self
                            .allocator
                            .allocate_uninitialized_array::<T>(new_capacity);
                        for i in 0..old_size {
                            new_data[i].write(old_array[i].assume_init_read());
                        }
                        self.clear();
                        let new_data_ptr: *mut std::mem::MaybeUninit<T> = new_data.as_mut_ptr();
                        self.data = new_data_ptr;
                        self.data_end = new_data_ptr.offset(old_size as isize);
                        self.capacity_end = new_data_ptr.offset(new_capacity as isize);
                    }
                }
            }
        }
    }

    // Similar to std::basic_string::append.
    pub fn append(&mut self, data: &[T])
    where
        T: Clone,
    {
        // TODO(strager): Make this more efficient.
        for x in data {
            self.push_back(x.clone());
        }
    }

    // Similar to std::basic_string::append.
    pub fn append_count(&mut self, count: usize, value: T)
    where
        T: Clone,
    {
        // TODO(strager): Make this more efficient.
        for _ in 0..count {
            self.push_back(value.clone());
        }
    }

    // Like clear(), but doesn't touch the allocated memory. Objects remain alive
    // and valid.
    pub fn release(&mut self) {
        self.data = std::ptr::null_mut();
        self.data_end = std::ptr::null_mut();
        self.capacity_end = std::ptr::null_mut();
    }

    pub fn clear(&mut self) {
        if !self.data.is_null() {
            let size = self.size();
            unsafe {
                for i in 0..size {
                    (&mut *self.data.offset(i as isize)).assume_init_drop();
                }
                self.allocator.deallocate(
                    self.data as *mut u8,
                    size * std::mem::size_of::<T>(),
                    std::mem::align_of::<T>(),
                );
            }
            self.release();
        }
    }

    fn reserve_grow_by_at_least(&mut self, minimum_new_entries: usize) {
        let old_capacity: usize = self.capacity();
        const MINIMUM_CAPACITY: usize = 4;
        let new_size: usize = std::cmp::max(
            std::cmp::max(MINIMUM_CAPACITY, old_capacity + minimum_new_entries),
            old_capacity * 2,
        );
        self.reserve_grow(new_size);
    }
}

impl<'alloc, T: Winkable, BumpAllocator: BumpAllocatorLike> std::ops::Index<usize>
    for RawBumpVector<'alloc, T, BumpAllocator>
{
    type Output = T;

    fn index<'a>(&'a self, index: usize) -> &'a T {
        assert!(index < self.size());
        unsafe { (&*self.data.offset(index as isize)).assume_init_ref() }
    }
}

impl<'alloc, T: Winkable, BumpAllocator: BumpAllocatorLike> std::ops::IndexMut<usize>
    for RawBumpVector<'alloc, T, BumpAllocator>
{
    fn index_mut<'a>(&'a mut self, index: usize) -> &'a mut T {
        assert!(index < self.size());
        unsafe { (&mut *self.data.offset(index as isize)).assume_init_mut() }
    }
}

impl<'alloc, T: Winkable, BumpAllocator: BumpAllocatorLike> Drop
    for RawBumpVector<'alloc, T, BumpAllocator>
{
    fn drop(&mut self) {
        self.clear();
    }
}
