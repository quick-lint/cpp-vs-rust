pub const fn concat<
    const RESULT_SIZE: usize,
    T: Copy,
    const LHS_SIZE: usize,
    const RHS_SIZE: usize,
>(
    lhs: &[T; LHS_SIZE],
    rhs: &[T; RHS_SIZE],
) -> [T; RESULT_SIZE] {
    if RESULT_SIZE == 0 {
        unimplemented!();
    }
    let mut result: [T; RESULT_SIZE] = if LHS_SIZE == 0 {
        [rhs[0]; RESULT_SIZE]
    } else {
        [lhs[0]; RESULT_SIZE]
    };
    let mut i: usize = 0;
    while i < LHS_SIZE {
        result[i] = lhs[i];
        i += 1;
    }
    while i < RHS_SIZE {
        result[i] = rhs[i - LHS_SIZE];
        i += 1;
    }
    result
}
