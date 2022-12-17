// Clippy's suggested fix is ugly.
#![allow(clippy::explicit_counter_loop)]
// Often we write (x << 0) or (x | 0) for symmetry with other code.
#![allow(clippy::identity_op)]
// Often we write lifetimes explicitly for better readability.
#![allow(clippy::needless_lifetimes)]
#![allow(clippy::redundant_static_lifetimes)]
// Refactoring is easier if the shorthand syntax is avoided.
#![allow(clippy::redundant_field_names)]

pub mod assert;
pub mod container;
pub mod fe;
pub mod i18n;
pub mod port;
pub mod util;
