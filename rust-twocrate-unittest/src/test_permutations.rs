use crate::util::permutations::*;

#[test]
fn permute_nothing() {
    assert_eq!(permutations_vec(&[] as &[i32]), vec![] as Vec<Vec<i32>>);
}

#[test]
fn permute_one_gives_one_permutation() {
    assert_eq!(permutations_vec(&[100i32]), vec![vec![100i32]]);
}

#[test]
fn permute_two_gives_two_permutation() {
    assert_eq!(
        permutations_vec(&[100i32, 200i32]),
        vec![vec![100i32, 200i32], vec![200i32, 100i32]]
    );
}

#[test]
fn permute_three_gives_six_permutation() {
    assert_eq!(
        permutations_vec(&[100, 200, 300] as &[i32]),
        vec![
            vec![100, 200, 300],
            vec![100, 300, 200],
            vec![200, 100, 300],
            vec![200, 300, 100],
            vec![300, 200, 100],
            vec![300, 100, 200],
        ]
    );
}

fn permutations_vec<T: Clone>(items: &[T]) -> Vec<Vec<T>> {
    let mut out: Vec<Vec<T>> = vec![];
    iterate_permutations(items, |permutation: &[T]| {
        out.push(permutation.into());
    });
    out
}
