pub mod capture;
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

/// Prelude containing everything you need to build a [`Test`](test::Test).
pub mod prelude {
    #[doc(no_inline)]
    pub use super::{
        ignore::IgnoreStatus,
        panic::PanicExpectation,
        test::{Test, TestFn, TestFnHandle, TestMeta, TestResult},
    };

    #[doc(no_inline)]
    pub use std::borrow::Cow;
}

mod util;

#[cfg(any(test, doctest))]
mod test_support;

/// Custom highlighted code?
#[doc = include_str!("../doc/html/regular.ansi.html")]
pub const WOW: &str = "WOW";
