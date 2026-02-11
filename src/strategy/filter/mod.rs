//! Test filtering for kitest.
//!
//! A filter decides which tests from the input slice are included in the test run.
//! Tests that do not match the filter are removed from the run entirely, and they
//! cannot be pulled back in by later steps.
//!
//! This is different to ignoring: ignore is decided per test after filtering, and
//! ignored tests still "exist" in the run (they can show up as ignored and they
//! are still part of the formatter flow). Filtering removes tests before we even
//! decide whether a test is ignored.
//!
//! In the default harness, filtering is mainly meant for the common workflow of
//! picking a smaller set of tests to focus on (for example by name matching),
//! rather than running the whole suite.
//!
//! Implement [`TestFilter`] to define a filter strategy for kitest.

use crate::test::Test;

mod no;
pub use no::*;

mod default;
pub use default::*;

/// The result of applying a [`TestFilter`].
///
/// This contains an iterator over the tests that are included in the run,
/// as well as the number of tests that were filtered out.
///
/// The iterator is required to be an [`ExactSizeIterator`]. Knowing the
/// number of remaining tests upfront allows other parts of the system
/// to make better decisions, such as estimating a worker count in the
/// runner or showing progress information in a formatter.
#[derive(Debug)]
pub struct FilteredTests<'t, I, Extra>
where
    I: ExactSizeIterator<Item = &'t Test<Extra>>,
    Extra: 't,
{
    /// The tests that are included in the run.
    pub tests: I,

    /// The number of tests that were filtered out.
    ///
    /// This may be used by formatters to report how many tests did not
    /// match the filter.
    pub filtered_out: usize,
}

/// A strategy for selecting which tests are included in a test run.
///
/// A `TestFilter` is applied before any ignore logic or test execution.
/// Tests that are filtered out are completely removed from the run and
/// are never seen by later stages.
///
/// This trait is used by the test harness to reduce the set of tests
/// it needs to work on.
pub trait TestFilter<Extra> {
    /// Filter the given slice of tests.
    ///
    /// The returned [`FilteredTests`] contains an iterator over the tests
    /// that are included in the run, as well as the number of tests that
    /// were filtered out.
    ///
    /// The iterator must yield references into the original `tests` slice
    /// and must have an exact size.
    fn filter<'t>(
        &self,
        tests: &'t [Test<Extra>],
    ) -> FilteredTests<'t, impl ExactSizeIterator<Item = &'t Test<Extra>>, Extra>;
}
