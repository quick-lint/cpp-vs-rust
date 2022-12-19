use crate::container::linked_bump_allocator::*;
use crate::container::winkable::*;
use crate::qljs_assert;
use crate::util::narrow_cast::*;

// TODO(port-later): Use InstrumentedVector if vector instrumentation is enabled.
pub type BumpVector<'alloc, T, BumpAllocator> =
    UninstrumentedVector<RawBumpVector<'alloc, T, BumpAllocator>>;

// Wraps a vector class so it has the same interface as
// InstrumentedVector<Vector> (but without the instrumentation overhead).
//
// NOTE(port): We're not porting InstrumentedVector right now.
pub struct UninstrumentedVector<Vector>(Vector);

pub trait VectorLike {
    type T;
    type Allocator;

    fn new(allocator: Self::Allocator) -> Self;
    fn is_empty(&self) -> bool;
    fn len(&self) -> usize;
    fn capacity(&self) -> usize;
    fn as_slice(&self) -> &[Self::T];
    fn as_mut_slice(&mut self) -> &mut [Self::T];
    fn reserve_to(&mut self, new_capacity: usize);
    fn push(&mut self, value: Self::T);
    fn pop(&mut self);
    fn resize(&mut self, new_size: usize)
    where
        Self::T: Default;
    // Similar to std::basic_string::append.
    fn extend_from_slice(&mut self, data: &[Self::T])
    where
        Self::T: Clone;
    // Similar to std::basic_string::append.
    fn append_count(&mut self, count: usize, value: Self::T)
    where
        Self::T: Clone;
    // Like clear(), but doesn't touch the allocated memory. Objects remain alive
    // and valid.
    fn release(&mut self) -> *mut [Self::T];
}

impl<Vector: VectorLike> UninstrumentedVector<Vector> {
    pub fn new(
        _debug_owner: &'static str,
        allocator: Vector::Allocator,
    ) -> UninstrumentedVector<Vector> {
        UninstrumentedVector(Vector::new(allocator))
    }

    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
    #[inline(always)]
    pub fn len(&self) -> usize {
        self.0.len()
    }
    #[inline(always)]
    pub fn capacity(&self) -> usize {
        self.0.capacity()
    }
    #[inline(always)]
    pub fn as_slice(&self) -> &[Vector::T] {
        self.0.as_slice()
    }
    #[inline(always)]
    pub fn as_mut_slice(&mut self) -> &mut [Vector::T] {
        self.0.as_mut_slice()
    }
    #[inline(always)]
    pub fn reserve_to(&mut self, new_capacity: usize) {
        self.0.reserve_to(new_capacity);
    }
    #[inline(always)]
    pub fn push(&mut self, value: Vector::T) {
        self.0.push(value);
    }
    #[inline(always)]
    pub fn pop(&mut self) {
        self.0.pop();
    }
    #[inline(always)]
    pub fn extend_from_slice(&mut self, data: &[Vector::T])
    where
        Vector::T: Clone,
    {
        self.0.extend_from_slice(data);
    }
    #[inline(always)]
    pub fn append_count(&mut self, count: usize, value: Vector::T)
    where
        Vector::T: Clone,
    {
        self.0.append_count(count, value);
    }
    #[inline(always)]
    pub fn resize(&mut self, new_size: usize)
    where
        Vector::T: Default,
    {
        self.0.resize(new_size);
    }
    #[inline(always)]
    pub fn release(&mut self) -> *mut [Vector::T] {
        self.0.release()
    }

    // TODO(port-later): Expose more RawBumpVector functions.
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

    fn is_empty(&self) -> bool {
        self.data == self.data_end
    }

    fn len(&self) -> usize {
        // NOTE(strager): offset_from is UB if data or data_end is null.
        if self.data.is_null() {
            0
        } else {
            unsafe { narrow_cast(self.data_end.offset_from(self.data)) }
        }
    }

    fn capacity(&self) -> usize {
        // NOTE(strager): offset_from is UB if data or capacity_end is null.
        if self.data.is_null() {
            0
        } else {
            unsafe { narrow_cast(self.capacity_end.offset_from(self.data)) }
        }
    }

    fn as_slice(&self) -> &[T] {
        // NOTE(strager): from_raw_parts is UB if self.data is null, even if the len is 0.
        if self.data.is_null() {
            &[]
        } else {
            unsafe { std::slice::from_raw_parts(self.data as *const T, self.len()) }
        }
    }

    fn as_mut_slice(&mut self) -> &mut [T] {
        // NOTE(strager): from_raw_parts is UB if self.data is null, even if the len is 0.
        if self.data.is_null() {
            &mut []
        } else {
            unsafe { std::slice::from_raw_parts_mut(self.data as *mut T, self.len()) }
        }
    }

    fn reserve_to(&mut self, new_capacity: usize) {
        if self.capacity() < new_capacity {
            self.reserve_grow(new_capacity);
        }
    }

    fn push(&mut self, value: T) {
        if self.capacity_end == self.data_end {
            self.reserve_grow_by_at_least(1);
        }
        unsafe {
            (*self.data_end).write(value);
            self.data_end = self.data_end.offset(1);
        }
    }

    fn extend_from_slice(&mut self, data: &[T])
    where
        T: Clone,
    {
        // TODO(strager): Make this more efficient.
        for x in data {
            self.push(x.clone());
        }
    }

    // Similar to std::basic_string::append.
    fn append_count(&mut self, count: usize, value: T)
    where
        T: Clone,
    {
        // TODO(strager): Make this more efficient.
        for _ in 0..count {
            self.push(value.clone());
        }
    }

    fn pop(&mut self) {
        qljs_assert!(!self.is_empty());
        // NOTE(port): The C++ code didn't destruct, so we don't drop here.
        self.data_end = unsafe { self.data_end.offset(-1) };
    }

    fn resize(&mut self, new_len: usize)
    where
        T: Default,
    {
        unsafe {
            let old_len = self.len();
            if new_len == old_len {
                // Do nothing.
            } else if new_len < old_len {
                let new_end: *mut std::mem::MaybeUninit<T> = self.data.add(new_len);
                for i in new_len..old_len {
                    (*self.data.add(i)).assume_init_drop();
                }
                self.data_end = new_end;
            } else {
                let old_capacity = self.capacity();
                if new_len > old_capacity {
                    self.reserve_grow_by_at_least(new_len - old_capacity);
                }
                let new_end: *mut std::mem::MaybeUninit<T> = self.data.add(new_len);
                for i in old_len..new_len {
                    (*self.data.add(i)).write(T::default());
                }
                self.data_end = new_end;
            }
        }
    }

    fn release(&mut self) -> *mut [T] {
        let result: *mut [T] = self.as_mut_slice();
        self.data = std::ptr::null_mut();
        self.data_end = std::ptr::null_mut();
        self.capacity_end = std::ptr::null_mut();
        result
    }
}

impl<'alloc, T: Winkable, BumpAllocator: BumpAllocatorLike>
    RawBumpVector<'alloc, T, BumpAllocator>
{
    pub fn get_allocator(&self) -> &'alloc BumpAllocator {
        self.allocator
    }

    pub fn front_mut(&mut self) -> &mut T {
        qljs_assert!(!self.is_empty());
        unsafe { (*self.data).assume_init_mut() }
    }

    pub fn back_mut(&mut self) -> &mut T {
        qljs_assert!(!self.is_empty());
        unsafe { (*self.data_end.offset(-1)).assume_init_mut() }
    }

    pub fn front(&self) -> &T {
        qljs_assert!(!self.is_empty());
        unsafe { (*self.data).assume_init_ref() }
    }

    pub fn back(&self) -> &T {
        qljs_assert!(!self.is_empty());
        unsafe { (*self.data_end.offset(-1)).assume_init_ref() }
    }

    pub fn reserve_grow(&mut self, new_capacity: usize) {
        unsafe {
            qljs_assert!(new_capacity > self.capacity());
            if self.data.is_null() {
                self.data = self
                    .allocator
                    .allocate_uninitialized_array::<T>(new_capacity)
                    .as_mut_ptr();
                self.data_end = self.data;
                self.capacity_end = self.data.add(new_capacity);
            } else {
                let old_len = self.len();
                let old_capacity = self.capacity();
                let mut array: &mut [std::mem::MaybeUninit<T>] =
                    std::slice::from_raw_parts_mut(self.data, old_capacity);
                let grew: bool = self
                    .allocator
                    .try_grow_array_in_place(&mut array, new_capacity);
                if grew {
                    // NOTE(strager): These are no-ops, but Miri needs to understand that future
                    // uses of self.data and self.data_end are valid and refer to the "new" array.
                    qljs_assert!(self.data == array.as_mut_ptr());
                    self.data = array.as_mut_ptr();
                    qljs_assert!(self.data_end == self.data.add(old_len));
                    self.data_end = self.data.add(old_len);

                    self.capacity_end = self.data.add(new_capacity);
                } else {
                    let new_data: &mut [std::mem::MaybeUninit<T>] = self
                        .allocator
                        .allocate_uninitialized_array::<T>(new_capacity);
                    for i in 0..old_len {
                        new_data[i].write(array[i].assume_init_read());
                    }
                    self.clear();
                    let new_data_ptr: *mut std::mem::MaybeUninit<T> = new_data.as_mut_ptr();
                    self.data = new_data_ptr;
                    self.data_end = new_data_ptr.add(old_len);
                    self.capacity_end = new_data_ptr.add(new_capacity);
                }
            }
        }
    }

    pub fn clear(&mut self) {
        if !self.data.is_null() {
            let len = self.len();
            unsafe {
                for i in 0..len {
                    (*self.data.add(i)).assume_init_drop();
                }
                self.allocator.deallocate(
                    self.data as *mut u8,
                    len * std::mem::size_of::<T>(),
                    std::mem::align_of::<T>(),
                );
            }
            self.release();
        }
    }

    fn reserve_grow_by_at_least(&mut self, minimum_new_entries: usize) {
        let old_capacity: usize = self.capacity();
        const MINIMUM_CAPACITY: usize = 4;
        let new_len: usize = std::cmp::max(
            std::cmp::max(MINIMUM_CAPACITY, old_capacity + minimum_new_entries),
            old_capacity * 2,
        );
        self.reserve_grow(new_len);
    }
}

impl<'alloc, T: Winkable, BumpAllocator: BumpAllocatorLike> std::ops::Index<usize>
    for RawBumpVector<'alloc, T, BumpAllocator>
{
    type Output = T;

    fn index<'a>(&'a self, index: usize) -> &'a T {
        qljs_assert!(index < self.len());
        unsafe { (*self.data.add(index)).assume_init_ref() }
    }
}

impl<'alloc, T: Winkable, BumpAllocator: BumpAllocatorLike> std::ops::IndexMut<usize>
    for RawBumpVector<'alloc, T, BumpAllocator>
{
    fn index_mut<'a>(&'a mut self, index: usize) -> &'a mut T {
        qljs_assert!(index < self.len());
        unsafe { (*self.data.add(index)).assume_init_mut() }
    }
}

impl<'alloc, T: Winkable, BumpAllocator: BumpAllocatorLike> Drop
    for RawBumpVector<'alloc, T, BumpAllocator>
{
    fn drop(&mut self) {
        self.clear();
    }
}
