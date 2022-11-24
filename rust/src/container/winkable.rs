// A type is winkable if the underlying memory for instances of the type can
// safely be deallocated without calling the object's destructor. (Memory leaks
// are considered "safe".)
//
// Example winkable types:
// * anything trivially destructible
// * container types like std::vector<U>, if U is winkable
//
// TODO(port): Update description for Rustisms.
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
