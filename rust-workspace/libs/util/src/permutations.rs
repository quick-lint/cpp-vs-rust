pub fn iterate_permutations<T: Clone, Func: FnMut(&[T])>(items: &[T], mut callback: Func) {
    if !items.is_empty() {
        iterate_permutations_impl(&mut items.into(), 0, &mut callback);
    }
}

fn iterate_permutations_impl<T, Func: FnMut(&[T])>(
    items: &mut Vec<T>,
    n: usize,
    callback: &mut Func,
) {
    // Slow but simple implementation.
    if n == items.len() {
        callback(items.as_slice());
        return;
    }
    for i in n..items.len() {
        items.swap(i, n);
        iterate_permutations_impl(items, n + 1, callback);
        items.swap(i, n);
    }
}
