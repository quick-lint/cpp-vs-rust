// Backport of std::assert_matches::*.

#[macro_export]
macro_rules! assert_matches {
    ($actual:expr, $($expected_pattern:pat_param)|+ $(if $guard:expr)? $(,)?) => {
        match $actual {
            $($expected_pattern)|+ $(if $guard)? => { /* Assertion passed. */ },
            ref actual => {
                panic!("assertion failed: {} ({:?}) does not match {}",
                    stringify!($actual),
                    actual,
                    stringify!($($expected_pattern)|+ $(if $guard)?),
                );
            }
        }
    }
}
