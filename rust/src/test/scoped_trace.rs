#[macro_export]
macro_rules! scoped_trace {
    ($expr:expr $(,)?) => {
        // TODO(port): SCOPED_TRACE from Google Test.
    };
}
