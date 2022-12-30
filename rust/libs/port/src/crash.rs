#[macro_export]
macro_rules! qljs_crash_allowing_core_dump {
    () => {
        // TODO(port-later): Is panic the right choice here? How heavy-weight is it? Could we
        // do something simpler? Maybe something like core::arch::x86_64::ud2 is better.
        panic!();
    };
}
