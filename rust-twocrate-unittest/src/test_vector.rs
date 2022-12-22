use crate::container::linked_bump_allocator::*;
use crate::container::vector::*;
use crate::container::winkable::*;

const I32_ALIGNMENT: usize = std::mem::align_of::<i32>();

#[test]
fn is_empty() {
    let alloc = LinkedBumpAllocator::<I32_ALIGNMENT>::new("test");
    let v: BumpVector<i32, _> = BumpVector::new("test", &alloc);
    assert!(v.is_empty());
    assert_eq!(v.len(), 0);
    assert_eq!(v.capacity(), 0);
}

#[test]
fn append_into_reserved_memory() {
    let alloc = LinkedBumpAllocator::<I32_ALIGNMENT>::new("test");
    let mut v: BumpVector<i32, _> = BumpVector::new("test", &alloc);
    v.reserve_to(2);
    assert_eq!(v.capacity(), 2);
    assert_eq!(v.len(), 0);

    v.push(100);
    assert_eq!(v.capacity(), 2);
    assert_eq!(v.len(), 1);
    assert_eq!(to_vec(&v), vec![100]);

    v.push(200);
    assert_eq!(v.capacity(), 2);
    assert_eq!(v.len(), 2);
    assert_eq!(to_vec(&v), vec![100, 200]);
}

#[test]
fn append_into_new_memory() {
    let alloc = LinkedBumpAllocator::<I32_ALIGNMENT>::new("test");
    let mut v: BumpVector<i32, _> = BumpVector::new("test", &alloc);
    assert_eq!(v.capacity(), 0);
    assert_eq!(v.len(), 0);

    v.push(100);
    assert!(v.capacity() > 0);
    assert_eq!(v.len(), 1);
    assert_eq!(to_vec(&v), vec![100]);

    v.push(200);
    assert!(v.capacity() > 0);
    assert_eq!(v.len(), 2);
    assert_eq!(to_vec(&v), vec![100, 200]);
}

#[test]
fn growing_allocation_in_place() {
    let alloc = LinkedBumpAllocator::<I32_ALIGNMENT>::new("test");
    let mut v: BumpVector<i32, _> = BumpVector::new("test", &alloc);
    v.reserve_to(2);

    v.push(100);
    v.push(200);
    assert_eq!(v.capacity(), 2);
    assert_eq!(to_vec(&v), vec![100, 200]);

    v.push(300);
    assert!(v.capacity() > 2);
    v.push(400);
    assert_eq!(to_vec(&v), vec![100, 200, 300, 400]);
}

#[test]
fn growing_allocation_by_copy() {
    let alloc = LinkedBumpAllocator::<I32_ALIGNMENT>::new("test");
    let mut v: BumpVector<i32, _> = BumpVector::new("test", &alloc);
    v.reserve_to(2);

    v.push(100);
    v.push(200);
    assert_eq!(v.capacity(), 2);
    assert_eq!(to_vec(&v), vec![100, 200]);
    let old_v_data_pointer = v.as_slice().as_ptr() as usize;

    // Prevent allocation from growing in-place.
    let middle_number: *mut i32 = alloc.new_object(42i32);

    v.push(300);
    assert!(v.capacity() > 2);
    v.push(400);
    assert_eq!(to_vec(&v), vec![100, 200, 300, 400]);

    assert_ne!(
        old_v_data_pointer,
        v.as_slice().as_ptr() as usize,
        "growing vector should use new data pointer"
    );
    assert_eq!(
        unsafe { *middle_number },
        42,
        "growing vector shouldn't change unrelated allocation"
    );
}

#[test]
fn resize_allows_same_size() {
    let alloc = LinkedBumpAllocator::<I32_ALIGNMENT>::new("test");
    let mut v: BumpVector<i32, _> = BumpVector::new("test", &alloc);
    v.push(100);
    v.push(200);
    let old_v_data_pointer = v.as_slice().as_ptr() as usize;
    let old_capacity: usize = v.capacity();

    v.resize(2);

    assert_eq!(v.len(), 2, "resizing vector should not change size");
    assert_eq!(
        v.capacity(),
        old_capacity,
        "resizing vector should not change capacity"
    );
    assert_eq!(to_vec(&v), vec![100, 200]);
    assert_eq!(
        old_v_data_pointer,
        v.as_slice().as_ptr() as usize,
        "resizing vector should not change data pointer"
    );
}

#[test]
fn resize_allows_shrinking() {
    let alloc = LinkedBumpAllocator::<I32_ALIGNMENT>::new("test");
    let mut v: BumpVector<i32, _> = BumpVector::new("test", &alloc);
    v.push(100);
    v.push(200);
    v.push(300);
    let old_v_data_pointer = v.as_slice().as_ptr() as usize;
    let old_capacity: usize = v.capacity();

    v.resize(2);

    assert_eq!(v.len(), 2, "shrinking vector should change size");
    assert_eq!(
        v.capacity(),
        old_capacity,
        "shrinking vector should not change capacity"
    );
    assert_eq!(
        to_vec(&v),
        vec![100, 200],
        "shrinking vector should preserve some elements"
    );
    assert_eq!(
        old_v_data_pointer,
        v.as_slice().as_ptr() as usize,
        "shrinking vector should not change data pointer"
    );
}

#[test]
fn resize_allows_growing_within_capacity() {
    let alloc = LinkedBumpAllocator::<I32_ALIGNMENT>::new("test");
    let mut v: BumpVector<i32, _> = BumpVector::new("test", &alloc);
    v.push(100);
    v.push(200);
    let old_v_data_pointer = v.as_slice().as_ptr() as usize;
    let old_capacity: usize = v.capacity();

    assert!(old_capacity >= 3);
    v.resize(3);

    assert_eq!(v.len(), 3, "growing vector should change size");
    assert_eq!(
        v.capacity(),
        old_capacity,
        "growing vector should not change capacity"
    );
    assert_eq!(
        to_vec(&v),
        vec![100, 200, 0],
        "growing vector should default-construct new elements"
    );
    assert_eq!(
        old_v_data_pointer,
        v.as_slice().as_ptr() as usize,
        "growing vector within capacity should not change data pointer"
    );
}

#[test]
fn resize_allows_growing_outside_capacity() {
    let alloc = LinkedBumpAllocator::<I32_ALIGNMENT>::new("test");
    let mut v: BumpVector<i32, _> = BumpVector::new("test", &alloc);
    v.push(100);
    v.push(200);

    assert!(v.capacity() < 10);
    v.resize(10);

    assert_eq!(v.len(), 10, "growing vector should change size");
    assert_eq!(v.capacity(), 10, "growing vector should change capacity");
    assert_eq!(
        to_vec(&v),
        vec![100, 200, 0, 0, 0, 0, 0, 0, 0, 0],
        "growing vector should default-construct new elements"
    );
}

#[test]
fn pop_shrinks_vector() {
    let alloc = LinkedBumpAllocator::<I32_ALIGNMENT>::new("test");
    let mut v: BumpVector<i32, _> = BumpVector::new("test", &alloc);
    v.push(100);
    v.push(200);
    v.push(300);
    v.pop();

    assert_eq!(to_vec(&v), vec![100, 200]);
    assert!(v.capacity() >= 3);
}

#[test]
fn pop_then_push_reuses_memory() {
    let alloc = LinkedBumpAllocator::<I32_ALIGNMENT>::new("test");
    let mut v: BumpVector<i32, _> = BumpVector::new("test", &alloc);
    v.push(100);
    v.push(200);
    v.push(300);
    v.pop();
    let old_v_data_pointer = v.as_slice().as_ptr() as usize;
    v.push(400);
    let v_data_pointer = v.as_slice().as_ptr() as usize;

    assert_eq!(to_vec(&v), vec![100, 200, 400]);
    assert_eq!(v_data_pointer, old_v_data_pointer);
    assert!(v.capacity() >= 3);
}

#[test]
fn moving_preserves_pointers() {
    let alloc = LinkedBumpAllocator::<I32_ALIGNMENT>::new("test");
    let mut v: BumpVector<i32, _> = BumpVector::new("test", &alloc);
    v.push(100);
    v.push(200);

    let old_v_data_pointer = v.as_slice().as_ptr() as usize;
    let old_v_capacity: usize = v.capacity();
    let old_v_len: usize = v.len();

    let v2: BumpVector<i32, _> = v; // Move.

    assert_eq!(v2.as_slice().as_ptr() as usize, old_v_data_pointer);
    assert_eq!(v2.capacity(), old_v_capacity);
    assert_eq!(v2.len(), old_v_len);
}

fn to_vec<T: Clone + Winkable, BumpAllocator: BumpAllocatorLike>(
    v: &BumpVector<T, BumpAllocator>,
) -> Vec<T> {
    v.as_slice().to_vec()
}
