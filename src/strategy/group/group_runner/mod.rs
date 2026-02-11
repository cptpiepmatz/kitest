use std::ops::ControlFlow;

use crate::outcome::TestOutcome;

mod simple;
pub use simple::*;

/// The outcomes of a single test group.
///
/// This is a list of test names and their corresponding [`TestOutcome`]
/// values, produced after running one group.
pub type TestGroupOutcomes<'t> = Vec<(&'t str, TestOutcome)>;

/// A strategy for running a single test group.
///
/// A group runner controls how groups are executed in a grouped test harness.
/// Its main responsibility is to perform work before and after a group runs,
/// for example setting up or cleaning up shared state for that group.
///
/// The harness calls [`run_group`](TestGroupRunner::run_group) once per group.
pub trait TestGroupRunner<'t, Extra, GroupKey, GroupCtx> {
    /// Run a single group.
    ///
    /// - `f` executes all tests in the group and returns their outcomes
    /// - `key` is the group key returned by the [`TestGrouper`](super::TestGrouper)
    /// - `ctx` is optional group context provided by the grouper for this key
    ///
    /// The implementation should call `f` to actually execute the tests in the group.
    /// Around that call it may perform setup and teardown work.
    ///
    /// The returned [`ControlFlow`] decides whether execution should continue with the next group:
    /// - `ControlFlow::Continue(outcomes)` runs the next group
    /// - `ControlFlow::Break(outcomes)` stops after this group
    fn run_group<F>(
        &self,
        f: F,
        key: &GroupKey,
        ctx: Option<&GroupCtx>,
    ) -> ControlFlow<TestGroupOutcomes<'t>, TestGroupOutcomes<'t>>
    where
        F: FnOnce() -> TestGroupOutcomes<'t>;
}
