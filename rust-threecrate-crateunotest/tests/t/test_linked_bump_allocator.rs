use cpp_vs_rust::container::linked_bump_allocator::*;
use cpp_vs_rust::port::ptr::*;

#[test]
fn separate_allocations_are_contiguous_without_padding() {
    // HACK(strager): We would use a generic function, but if we do that, then we can't call
    // std::mem::align_of.
    macro_rules! test_with_type {
        ($type:ty, $value:expr) => {{
            const ALIGNMENT: usize = std::mem::align_of::<$type>();
            let alloc = LinkedBumpAllocator::<ALIGNMENT>::new("test");
            let a = alloc.new_object::<$type>($value);
            let b = alloc.new_object::<$type>($value);
            let c = alloc.new_object::<$type>($value);
            let d = alloc.new_object::<$type>($value);

            let delta: isize = byte_offset_from(a, b);
            assert!(
                delta == (ALIGNMENT as isize) || delta == -(ALIGNMENT as isize),
                "addresses should either go up or down with no padding"
            );
            assert_eq!(
                byte_offset_from(b, c),
                delta,
                "delta should be consistent between allocations"
            );
            assert_eq!(
                byte_offset_from(c, d),
                delta,
                "delta should be consistent between allocations"
            );
        }};
    }

    test_with_type!(u8, 100);
    test_with_type!(u32, 200);
    test_with_type!(usize, 300);
    let temp: u32 = 42;
    test_with_type!(*const u32, &temp);
}

#[test]
fn less_aligned_object_keeps_next_allocation_aligned() {
    let alloc = LinkedBumpAllocator::<4>::new("test");
    let _small: *mut u8 = alloc.new_object(42u8);
    let after: *mut u32 = alloc.new_object(42u32);
    assert_valid_object(after);
}

#[test]
fn less_aligned_bytes_keeps_next_allocation_aligned() {
    let alloc = LinkedBumpAllocator::<4>::new("test");
    let _small: *mut u8 = alloc.allocate(1, /*align=*/ 1);
    let after: *mut u32 = alloc.new_object(42u32);
    assert_valid_object(after);
}

#[test]
fn array_allocation_is_contiguous() {
    let alloc = LinkedBumpAllocator::<1>::new("test");
    let chars: &mut [std::mem::MaybeUninit<u8>] = alloc.allocate_uninitialized_array::<u8>(42);
    assert_valid_memory(chars.as_mut_ptr(), 42, std::mem::align_of::<u8>());
}

#[test]
fn less_aligned_array_keeps_next_allocation_aligned() {
    let alloc = LinkedBumpAllocator::<4>::new("test");
    let _chars: &mut [std::mem::MaybeUninit<u8>] = alloc.allocate_uninitialized_array::<u8>(3);
    let after: *mut u32 = alloc.new_object(42u32);
    assert_valid_object(after);
}

#[test]
fn less_aligned_pre_grown_and_grown_array_keeps_next_allocation_aligned() {
    let alloc = LinkedBumpAllocator::<4>::new("test");

    let mut chars: &mut [std::mem::MaybeUninit<u8>] = alloc.allocate_uninitialized_array::<u8>(3);
    let grew: bool = alloc.try_grow_array_in_place(&mut chars, 6);
    assert!(grew);

    let after: *mut u32 = alloc.new_object(42u32);
    assert_valid_object(after);
}

#[test]
fn less_aligned_grown_array_keeps_next_allocation_aligned() {
    let alloc = LinkedBumpAllocator::<4>::new("test");

    let mut chars: &mut [std::mem::MaybeUninit<u8>] = alloc.allocate_uninitialized_array::<u8>(4);
    let grew: bool = alloc.try_grow_array_in_place(&mut chars, 7);
    assert!(grew);

    let after: *mut u32 = alloc.new_object(42u32);
    assert_valid_object(after);
}

#[test]
fn allocate_bigger_than_remaining_space_in_chunk_allocates_in_new_chunk() {
    const CHUNK_SIZE: usize = 4096 - std::mem::size_of::<*mut u8>() * 2; // Implementation detail.
    struct BigObject([u8; CHUNK_SIZE - 2]);

    let alloc = LinkedBumpAllocator::<4>::new("test");
    assert!(
        CHUNK_SIZE >= std::mem::size_of::<BigObject>(),
        "A BigObject should fit in the chunk before allocating padding"
    );
    let _padding = alloc.new_object(42u32);

    assert!(
        alloc.remaining_bytes_in_current_chunk() < std::mem::size_of::<BigObject>(),
        "A BigObject should not fit in the chunk after allocating padding"
    );
    let big = alloc.new_object(BigObject([42; CHUNK_SIZE - 2]));

    assert_valid_object(big);
}

#[test]
fn filling_first_chunk_allocates_second_chunk() {
    let alloc = LinkedBumpAllocator::<1>::new("test");

    let first_chunk_size = alloc.remaining_bytes_in_current_chunk();
    for _ in 0..first_chunk_size {
        let _byte = alloc.new_object(42u8);
    }

    let new_chunk_object = alloc.new_object(42u8);
    // TODO(strager): How do we verify that new_chunk_object is in its own chunk?
    assert_valid_object(new_chunk_object);
}

#[test]
fn rewinding_within_chunk_reuses_memory() {
    let alloc = LinkedBumpAllocator::<1>::new("test");

    let _byte_0 = alloc.new_object(0u8);
    let _byte_1 = alloc.new_object(1u8);

    let rewind: LinkedBumpAllocatorRewindState = alloc.prepare_for_rewind();
    let byte_2a: *mut u8 = alloc.new_object(2u8);
    let byte_3a: *mut u8 = alloc.new_object(3u8);
    unsafe {
        alloc.rewind(rewind);
    }

    let byte_2b: *mut u8 = alloc.new_object(2u8);
    assert_eq!(byte_2b, byte_2a);
    assert_valid_object(byte_2b);
    let byte_3b: *mut u8 = alloc.new_object(3u8);
    assert_eq!(byte_3b, byte_3a);
    assert_valid_object(byte_3b);
}

#[test]
fn rewinding_across_chunk_reuses_memory_of_first_chunk() {
    let alloc = LinkedBumpAllocator::<1>::new("test");

    // First chunk:
    let first_chunk_size = alloc.remaining_bytes_in_current_chunk();
    for _ in 0..(first_chunk_size / 2) {
        let _byte = alloc.new_object(1u8);
    }
    let rewind: LinkedBumpAllocatorRewindState = alloc.prepare_for_rewind();
    let mut reusable_allocations: Vec<*mut u8> = vec![];
    for _ in (first_chunk_size / 2)..first_chunk_size {
        reusable_allocations.push(alloc.new_object(2u8));
    }

    // Second chunk:
    let second_chunk_size = alloc.remaining_bytes_in_current_chunk();
    for _ in 0..(second_chunk_size / 2) {
        let _byte = alloc.new_object(3u8);
    }

    unsafe {
        alloc.rewind(rewind);
    }
    // First chunk:
    for reusable_allocation in reusable_allocations {
        let new_allocation: *mut u8 = alloc.new_object(4u8);
        assert_eq!(new_allocation, reusable_allocation);
        assert_valid_object(new_allocation);
    }

    // Second chunk:
    let second_chunk_byte: *mut u8 = alloc.new_object(5u8);
    assert_valid_object(second_chunk_byte);
}

#[test]
fn rewinding_across_chunk_uses_unallocated_memory_of_first_chunk() {
    let alloc = LinkedBumpAllocator::<1>::new("test");

    // First chunk:
    let first_chunk_size = alloc.remaining_bytes_in_current_chunk();
    for _ in 0..(first_chunk_size / 2) {
        let _byte = alloc.new_object(1u8);
    }
    let rewind: LinkedBumpAllocatorRewindState = alloc.prepare_for_rewind();

    // Second chunk:
    let _big_allocation = alloc.allocate_uninitialized_array::<char>(first_chunk_size + 64);

    // Third chunk:
    let third_chunk_size = first_chunk_size;
    for _ in 0..(third_chunk_size / 2) {
        let _byte = alloc.new_object(3u8);
    }

    unsafe {
        alloc.rewind(rewind);
    }
    // First chunk:
    let new_allocation: *mut u8 = alloc.new_object(4u8);
    assert_valid_object(new_allocation);
    // TODO(strager): How do we verify that new_allocation is in the same chunk as
    // the original allocations?
}

#[test]
fn last_allocation_can_grow_in_place() {
    let alloc = LinkedBumpAllocator::<1>::new("test");
    let mut array: &mut [std::mem::MaybeUninit<u8>] = alloc.allocate_uninitialized_array::<u8>(10);
    let old_array_ptr: isize = array.as_ptr() as isize;
    let ok: bool = alloc.try_grow_array_in_place(&mut array, 20);
    assert!(ok);
    assert_valid_memory(array.as_mut_ptr(), 20, /*alignment=*/ 1);

    let next = alloc.allocate_uninitialized_array::<u8>(1);
    let diff = (next.as_ptr() as isize) - old_array_ptr;
    assert!(
        diff == 1 || diff == 20,
        "future allocations should not overlap resized array"
    );
}

#[test]
fn last_allocation_cannot_grow_beyond_current_chunk() {
    let alloc = LinkedBumpAllocator::<1>::new("test");

    let _first_byte = alloc.new_object(1u8); // Allocate the first chunk.
    let first_chunk_size = alloc.remaining_bytes_in_current_chunk();
    for _ in 0..(first_chunk_size - 15) {
        let _byte = alloc.new_object(2u8);
    }
    let mut array = alloc.allocate_uninitialized_array::<u8>(10);
    assert_eq!(alloc.remaining_bytes_in_current_chunk(), 5);

    let ok: bool = alloc.try_grow_array_in_place(&mut array, 20);
    assert!(!ok);
    assert_valid_memory(array.as_mut_ptr(), 10, /*alignment=*/ 1);

    let next = alloc.allocate_uninitialized_array::<u8>(1);
    let diff = (next.as_ptr() as isize) - (array.as_ptr() as isize);
    assert!(
        diff == 1 || diff == 10,
        "future allocations should pretend realloc request never happened"
    );
}

#[test]
fn non_last_allocation_cannot_grow() {
    let alloc = LinkedBumpAllocator::<1>::new("test");
    let mut array = alloc.allocate_uninitialized_array::<u8>(10);
    let last: *mut u8 = alloc.new_object(0u8);
    let ok: bool = alloc.try_grow_array_in_place(&mut array, 20);
    assert!(!ok);
    assert_valid_memory(array.as_mut_ptr(), 10, /*alignment=*/ 1);

    let next = alloc.allocate_uninitialized_array::<u8>(1);
    assert_ne!(
        next.as_mut_ptr() as *mut u8,
        last,
        "future allocations should not overlap resized array"
    );
}

#[cfg(feature = "qljs_debug")]
#[test]
fn cannot_allocate_when_disabled() {
    let result: Result<(), Box<dyn std::any::Any + Send + 'static>> =
        std::panic::catch_unwind(|| {
            let alloc = LinkedBumpAllocator::<1>::new("test");
            alloc.disable();
            // The following line should panic:
            let _ = alloc.new_object(42u8);
        });
    assert!(
        result.is_err(),
        "allocating should crash if allocation is disabled"
    );
}

#[cfg(feature = "qljs_debug")]
#[test]
fn can_allocate_after_disabling_then_reenabling() {
    let alloc = LinkedBumpAllocator::<1>::new("test");
    alloc.disable();
    alloc.enable();
    let c = alloc.new_object(42u8);
    assert_valid_object(c);
}

fn assert_is_aligned<T>(p: *mut T, alignment: usize) {
    assert!(
        is_aligned_to(p, alignment),
        "pointer {:#x} should be aligned to {:#x} bytes",
        p as usize,
        alignment
    );
}

// len is a number of T-s, not a number of bytes.
fn assert_valid_memory<T>(begin: *mut T, len: usize, alignment: usize) {
    unsafe {
        let end = begin.add(len);

        assert_is_aligned(begin, alignment);
        assert_is_aligned(end, alignment);

        assert!(begin <= end);

        // Try to get the OS or malloc to detect heap corruption by writing over all
        // of the given memory.
        std::ptr::write_bytes(begin, b'X', len);
    }
}

fn assert_valid_object<T>(object: *mut T) {
    assert_valid_memory(object, 1, std::mem::align_of::<T>());
}
