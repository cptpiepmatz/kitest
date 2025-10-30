pub mod formatter;
pub mod outcome;
pub mod test;

mod strategy;
pub use strategy::*;

mod harness;
pub use harness::*;

mod report;
pub use report::*;

mod whatever;
pub use whatever::*;

/// Custom highlighted code?
#[doc = include_str!("../doc/html/regular.ansi.html")]
pub const WOW: &str = "WOW";

#[test]
fn foo() {}

#[test]
fn bar() {}

#[test]
#[ignore = "for a reason"]
fn ignored() {}
