//! Test ignoring for kitest.
//!
//! Ignoring decides, per test, whether a test should be executed or skipped.
//! An ignored test is still part of the test run: it shows up as ignored in the
//! results and formatters still observe it.
//!
//! This is different to filtering. Filtering selects the set of tests at the
//! start of the run and removes non matching tests entirely. Ignoring happens
//! right before each test would run, so the decision can be made during the run.
//!
//! Implement [`TestIgnore`] to define an ignore strategy for kitest.

use std::borrow::Cow;

use crate::test::TestMeta;

mod no;
pub use no::*;

mod default;
pub use default::*;

/// The ignore decision for a single test.
///
/// The harness calls the ignore strategy right before a test would execute.
/// The decision can either allow the test to run, ignore it, or ignore it with
/// a reason that a formatter may display.
#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub enum IgnoreStatus {
    /// Run the test.
    #[default]
    Run,

    /// Ignore the test without a reason.
    Ignore,

    /// Ignore the test and provide a human readable reason.
    IgnoreWithReason(Cow<'static, str>),
}

impl From<bool> for IgnoreStatus {
    fn from(value: bool) -> Self {
        match value {
            true => Self::Ignore,
            false => Self::Run,
        }
    }
}

impl From<&'static str> for IgnoreStatus {
    fn from(value: &'static str) -> Self {
        Self::IgnoreWithReason(value.into())
    }
}

impl From<String> for IgnoreStatus {
    fn from(value: String) -> Self {
        Self::IgnoreWithReason(value.into())
    }
}

/// A strategy for deciding whether a test should be ignored.
///
/// This decision is made right before each test would run, which means it does
/// not need to be computed for all tests up front. This also allows the ignore
/// decision to depend on information that is only known once the run is already
/// in progress.
pub trait TestIgnore<Extra> {
    /// Decide whether the given test should be ignored.
    ///
    /// Returning [`IgnoreStatus::Run`] runs the test. Returning an ignore status
    /// skips execution and marks the test as ignored in the report.
    fn ignore(&self, meta: &TestMeta<Extra>) -> IgnoreStatus;
}

impl<Extra, F> TestIgnore<Extra> for F
where
    F: Fn(&TestMeta<Extra>) -> IgnoreStatus,
{
    fn ignore(&self, meta: &TestMeta<Extra>) -> IgnoreStatus {
        self(meta)
    }
}
