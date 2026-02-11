//! Test execution and scheduling for kitest.
//!
//! A runner coordinates running tests and producing [`TestOutcome`] values.
//! It may run tests on a single thread or use multiple worker threads.
//!
//! The harness passes the runner an iterator of test execution functions.
//! These functions already include panic handling (they return a [`TestStatus`]),
//! so the runner can focus on scheduling, timing, and output capture.
//!
//! Runners are also responsible for collecting per test output (stdout and stderr)
//! using the output capture mechanisms provided by the crate, and for measuring
//! test durations.
//!
//! Implement [`TestRunner`] to define how kitest schedules tests and turns statuses
//! into outcomes.

use std::{num::NonZeroUsize, thread::Scope};

use crate::{
    outcome::{TestOutcome, TestStatus},
    test::TestMeta,
};

mod default;
pub use default::*;

mod simple;
pub use simple::*;

mod smart;
pub use smart::*;

/// A strategy for running tests and producing [`TestOutcome`] values.
///
/// A runner organizes when and where tests execute.
/// It may use multiple threads, but it does not have to.
/// The produced iterator does not have to keep the same order as the incoming test iterator.
pub trait TestRunner<Extra> {
    /// Run the given tests and return their outcomes.
    ///
    /// The input iterator yields `(f, meta)` pairs where `f` is the test execution function.
    /// `f` already includes the panic handler and returns a [`TestStatus`].
    ///
    /// The runner receives a [`Scope`] so it can spawn threads while still borrowing
    /// the test metadata with lifetime `'t`.
    ///
    /// The returned iterator yields `(meta, outcome)` pairs.
    /// The order is not fixed and may differ from the input order.
    ///
    /// A runner typically:
    /// - executes `f` to obtain a status
    /// - measures duration
    /// - captures output
    /// - builds a [`TestOutcome`]
    ///
    /// The runner may choose how to interpret the returned status when building
    /// the outcome.
    fn run<'t, 's, I, F>(
        &self,
        tests: I,
        scope: &'s Scope<'s, 't>,
    ) -> impl Iterator<Item = (&'t TestMeta<Extra>, TestOutcome)>
    where
        I: ExactSizeIterator<Item = (F, &'t TestMeta<Extra>)>,
        F: (Fn() -> TestStatus) + Send + 's,
        Extra: 't;

    /// Return the number of workers this runner would like to use for `tests_count` tests.
    ///
    /// This is used to inform formatters (for example for progress output).
    fn worker_count(&self, tests_count: usize) -> NonZeroUsize;
}
