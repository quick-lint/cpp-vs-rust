use crate::container::linked_bump_allocator::*;

const ALIGN_OF_POINTER: usize = std::mem::align_of::<usize>();
pub type MonotonicAllocator = LinkedBumpAllocator<ALIGN_OF_POINTER>;
