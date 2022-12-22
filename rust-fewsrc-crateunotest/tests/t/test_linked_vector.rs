use cpp_vs_rust::container::linked_vector::*;
use cpp_vs_rust::port::allocator::*;
use cpp_vs_rust::util::narrow_cast::*;

#[test]
fn empty() {
    let v = LinkedVector::<i32>::new(global_allocator());
    assert!(v.is_empty());
    assert_eq!(to_vec(&v), vec![]);
}

#[test]
fn push_one() {
    let mut v = LinkedVector::<i32>::new(global_allocator());
    v.push(42);
    assert!(!v.is_empty());
    assert_eq!(to_vec(&v), vec![42]);
}

#[test]
fn push_seven() {
    let mut v = LinkedVector::<i32>::new(global_allocator());
    v.push(100);
    v.push(200);
    v.push(300);
    v.push(400);
    v.push(500);
    v.push(600);
    v.push(700);
    assert!(!v.is_empty());
    assert_eq!(to_vec(&v), vec![100, 200, 300, 400, 500, 600, 700]);
}

#[test]
fn push_full_chunk() {
    let mut v = LinkedVector::<i32>::new(global_allocator());
    let mut expected_items: Vec<i32> = vec![];
    for i in 0..narrow_cast(v.items_per_chunk()) {
        v.push(i);
        expected_items.push(i);
    }
    assert_eq!(to_vec(&v), expected_items);
}

#[test]
fn push_full_chunk_and_one() {
    let mut v = LinkedVector::<i32>::new(global_allocator());
    let mut expected_items: Vec<i32> = vec![];
    for i in 0..(narrow_cast::<i32, usize>(v.items_per_chunk()) + 1) {
        v.push(i);
        expected_items.push(i);
    }
    assert_eq!(to_vec(&v), expected_items);
}

#[test]
fn push_one_then_pop() {
    let mut v = LinkedVector::<i32>::new(global_allocator());
    v.push(42);
    v.pop();
    assert!(v.is_empty());
    assert_eq!(to_vec(&v), vec![]);
}

#[test]
fn push_two_then_pop() {
    let mut v = LinkedVector::<i32>::new(global_allocator());
    v.push(42);
    v.push(69);
    v.pop();
    assert!(!(v.is_empty()));
    assert_eq!(to_vec(&v), vec![42]);
    assert_eq!(*v.back(), 42);
}

#[test]
fn push_full_chunk_then_pop() {
    let mut v = LinkedVector::<i32>::new(global_allocator());
    let mut expected_items: Vec<i32> = vec![];
    for i in 0..narrow_cast(v.items_per_chunk()) {
        v.push(i);
        expected_items.push(i);
    }
    v.pop();
    expected_items.pop();
    assert_eq!(to_vec(&v), expected_items);
    assert_eq!(*v.back(), *expected_items.last().unwrap());
}

#[test]
fn push_full_chunk_plus_one_then_pop() {
    let mut v = LinkedVector::<i32>::new(global_allocator());
    let mut expected_items: Vec<i32> = vec![];
    for i in 0..(narrow_cast::<i32, usize>(v.items_per_chunk()) + 1) {
        v.push(i);
        expected_items.push(i);
    }
    v.pop();
    expected_items.pop();
    assert_eq!(to_vec(&v), expected_items);
    assert_eq!(*v.back(), *expected_items.last().unwrap());
}

#[test]
fn push_full_chunk_plus_one_then_pop_most() {
    let mut v = LinkedVector::<i32>::new(global_allocator());
    let mut expected_items: Vec<i32> = vec![];
    for i in 0..(narrow_cast::<i32, usize>(v.items_per_chunk()) + 1) {
        v.push(i);
        expected_items.push(i);
    }
    for _ in 0..(narrow_cast::<i32, usize>(v.items_per_chunk()) * 2 / 3) {
        v.pop();
        expected_items.pop();
    }
    assert_eq!(to_vec(&v), expected_items);
    assert_eq!(*v.back(), *expected_items.last().unwrap());
}

fn to_vec<T: Clone>(v: &LinkedVector<T>) -> Vec<T> {
    let mut result: Vec<T> = vec![];
    v.for_each(|x| {
        result.push(x.clone());
    });
    result
}
