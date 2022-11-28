use cpp_vs_rust::container::linked_vector::*;
use cpp_vs_rust::port::allocator::*;
use cpp_vs_rust::util::narrow_cast::*;

#[test]
fn empty() {
    let v = LinkedVector::<i32>::new(global_allocator());
    assert!(v.empty());
    assert_eq!(to_vec(&v), vec![]);
}

#[test]
fn emplace_back_one() {
    let mut v = LinkedVector::<i32>::new(global_allocator());
    v.emplace_back(42);
    assert!(!v.empty());
    assert_eq!(to_vec(&v), vec![42]);
}

#[test]
fn emplace_back_seven() {
    let mut v = LinkedVector::<i32>::new(global_allocator());
    v.emplace_back(100);
    v.emplace_back(200);
    v.emplace_back(300);
    v.emplace_back(400);
    v.emplace_back(500);
    v.emplace_back(600);
    v.emplace_back(700);
    assert!(!v.empty());
    assert_eq!(to_vec(&v), vec![100, 200, 300, 400, 500, 600, 700]);
}

#[test]
fn emplace_back_full_chunk() {
    let mut v = LinkedVector::<i32>::new(global_allocator());
    let mut expected_items: Vec<i32> = vec![];
    for i in 0..narrow_cast(v.items_per_chunk()) {
        v.emplace_back(i);
        expected_items.push(i);
    }
    assert_eq!(to_vec(&v), expected_items);
}

#[test]
fn emplace_back_full_chunk_and_one() {
    let mut v = LinkedVector::<i32>::new(global_allocator());
    let mut expected_items: Vec<i32> = vec![];
    for i in 0..(narrow_cast::<i32, usize>(v.items_per_chunk()) + 1) {
        v.emplace_back(i);
        expected_items.push(i);
    }
    assert_eq!(to_vec(&v), expected_items);
}

#[test]
fn emplace_back_one_then_pop_back() {
    let mut v = LinkedVector::<i32>::new(global_allocator());
    v.emplace_back(42);
    v.pop_back();
    assert!(v.empty());
    assert_eq!(to_vec(&v), vec![]);
}

#[test]
fn emplace_back_two_then_pop_back() {
    let mut v = LinkedVector::<i32>::new(global_allocator());
    v.emplace_back(42);
    v.emplace_back(69);
    v.pop_back();
    assert!(!(v.empty()));
    assert_eq!(to_vec(&v), vec![42]);
    assert_eq!(*v.back(), 42);
}

#[test]
fn emplace_back_full_chunk_then_pop_back() {
    let mut v = LinkedVector::<i32>::new(global_allocator());
    let mut expected_items: Vec<i32> = vec![];
    for i in 0..narrow_cast(v.items_per_chunk()) {
        v.emplace_back(i);
        expected_items.push(i);
    }
    v.pop_back();
    expected_items.pop();
    assert_eq!(to_vec(&v), expected_items);
    assert_eq!(*v.back(), *expected_items.last().unwrap());
}

#[test]
fn emplace_back_full_chunk_plus_one_then_pop_back() {
    let mut v = LinkedVector::<i32>::new(global_allocator());
    let mut expected_items: Vec<i32> = vec![];
    for i in 0..(narrow_cast::<i32, usize>(v.items_per_chunk()) + 1) {
        v.emplace_back(i);
        expected_items.push(i);
    }
    v.pop_back();
    expected_items.pop();
    assert_eq!(to_vec(&v), expected_items);
    assert_eq!(*v.back(), *expected_items.last().unwrap());
}

#[test]
fn emplace_back_full_chunk_plus_one_then_pop_back_most() {
    let mut v = LinkedVector::<i32>::new(global_allocator());
    let mut expected_items: Vec<i32> = vec![];
    for i in 0..(narrow_cast::<i32, usize>(v.items_per_chunk()) + 1) {
        v.emplace_back(i);
        expected_items.push(i);
    }
    for _ in 0..(narrow_cast::<i32, usize>(v.items_per_chunk()) * 2 / 3) {
        v.pop_back();
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
