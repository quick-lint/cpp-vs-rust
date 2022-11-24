#[macro_export]
macro_rules! qljs_always_assert {
    ($cond:expr $(,)?) => {
        // TODO(port): Force.
        assert!($cond);
    };
}

#[macro_export]
macro_rules! qljs_slow_assert {
    ($cond:expr $(,)?) => {
        // TODO(port): Conditional on qljs_debug feature.
        assert!($cond);
    };
}
