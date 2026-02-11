//! Panic handling for kitest.
//!
//! Tests can panic. In the built in Rust test harness, a panic usually means the
//! test failed, so we need a way to catch panics and turn them into a structured
//! status.
//!
//! A panic handler is responsible for actually executing the test function and
//! deciding the resulting [`TestStatus`]. The runner organizes when and where
//! tests are executed, but the panic handler is the piece that runs the test.
//!
//! The test metadata includes a [`PanicExpectation`]. A panic handler may respect
//! that expectation (for example "should panic"), but it does not have to.
//!
//! Implement [`TestPanicHandler`] to define how kitest executes tests and turns
//! panics into statuses.

use std::{borrow::Cow, panic::UnwindSafe};

use crate::{
    outcome::TestStatus,
    test::{TestMeta, TestResult},
};

mod no;
pub use no::*;

mod default;
pub use default::*;

/// The panic expectation for a test.
///
/// This value is stored in [`TestMeta`] and can be used by a panic handler to
/// decide whether a panic should be treated as success or failure.
#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub enum PanicExpectation {
    /// The test should not panic.
    #[default]
    ShouldNotPanic,

    /// The test should panic.
    ShouldPanic,

    /// The test should panic and include an expected message.
    ShouldPanicWithExpected(Cow<'static, str>),
}

impl From<bool> for PanicExpectation {
    fn from(value: bool) -> Self {
        match value {
            true => Self::ShouldPanic,
            false => Self::ShouldNotPanic,
        }
    }
}

impl From<&'static str> for PanicExpectation {
    fn from(value: &'static str) -> Self {
        Self::ShouldPanicWithExpected(value.into())
    }
}

impl From<String> for PanicExpectation {
    fn from(value: String) -> Self {
        Self::ShouldPanicWithExpected(value.into())
    }
}

/// A strategy for executing a test function and translating panics into a [`TestStatus`].
///
/// The runner calls into the panic handler to execute tests. The runner may use
/// the returned status to build a richer outcome, but the panic handler is the
/// one that decides the primary test status.
///
/// Panic handlers can be called in any way by the runner (for example from worker
/// threads), so the handler is passed by shared reference.
pub trait TestPanicHandler<Extra> {
    /// Execute the given test function and return a [`TestStatus`].
    ///
    /// The panic handler should at least execute `f`, since that is the actual
    /// test. The provided metadata can be used to read the test's
    /// [`PanicExpectation`] and to include context in the produced status.
    fn handle<F: FnOnce() -> TestResult + UnwindSafe>(
        &self,
        f: F,
        meta: &TestMeta<Extra>,
    ) -> TestStatus;
}
