use crate::qljs_assert_trap;
use crate::util::assert::*;

#[track_caller]
pub fn narrow_cast<Out, In: TryInto<Out>>(x: In) -> Out {
    match x.try_into() {
        Ok(x) => x,
        Err(_) => {
            let caller: &std::panic::Location = std::panic::Location::caller();
            report_assertion_failure(caller.file(), caller.line(), "number not in range");
            qljs_assert_trap!();
        }
    }
}
