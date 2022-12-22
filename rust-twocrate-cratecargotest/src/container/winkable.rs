// A type is winkable if the underlying memory for instances of the type can
// safely be deallocated without dropping the instance. (Memory leaks are considered "safe".)
//
// Example winkable types:
// * anything implementing Copy
// * container types like Vec<U>, if U is winkable
pub trait Winkable {}

impl Winkable for i8 {}
impl Winkable for i16 {}
impl Winkable for i32 {}
impl Winkable for i64 {}

impl Winkable for u8 {}
impl Winkable for u16 {}
impl Winkable for u32 {}
impl Winkable for u64 {}

impl Winkable for () {}
