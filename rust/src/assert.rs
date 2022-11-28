#[macro_export]
macro_rules! qljs_always_assert {
    ($cond:expr $(,)?) => {
        // TODO(port): Use our own infrastructure.
        assert!($cond);
    };
}

#[macro_export]
macro_rules! qljs_assert {
    ($cond:expr $(,)?) => {
        #[cfg(debug_assertions)]
        $crate::qljs_always_assert!($cond);
    };
}

#[macro_export]
macro_rules! qljs_slow_assert {
    ($cond:expr $(,)?) => {
        #[cfg(feature = "qljs_debug")]
        $crate::qljs_always_assert!($cond);
    };
}
