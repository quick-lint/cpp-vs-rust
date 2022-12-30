use cpp_vs_rust_util::qljs_always_assert;

#[test]
#[cfg(panic = "unwind")]
fn failing_assert_crashes() {
    let result = std::panic::catch_unwind(|| {
        let everything_is_okay = false;
        qljs_always_assert!(everything_is_okay);
    });
    assert!(result.is_err());
    // TODO(port-later): Check the printed message.
}

#[test]
fn passing_assert_does_not_crash() {
    qljs_always_assert!(true);
}

#[test]
fn passing_assert_executes_side_effects() {
    #[allow(unused_assignments)]
    let mut executed = false;
    qljs_always_assert!({
        executed = true;
        true
    });
    assert!(executed);
}
