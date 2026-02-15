use std::ops::ControlFlow;

use crate::group::{TestGroupOutcomes, TestGroupRunner};

/// A [`TestGroupRunner`] that simply runs each group and always continues.
///
/// This runner just executes the provided group function and returns
/// `ControlFlow::Continue` with the produced outcomes. It does not perform
/// any setup, teardown, or early stopping logic.
///
/// This is usually sufficient for basic grouped test harnesses and is
/// especially useful in tests where no special group handling is required.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct SimpleGroupRunner;

impl<'t, Extra, GroupKey, GroupCtx> TestGroupRunner<'t, Extra, GroupKey, GroupCtx>
    for SimpleGroupRunner
{
    fn run_group<F>(
        &self,
        f: F,
        _: &GroupKey,
        _: Option<&GroupCtx>,
    ) -> ControlFlow<TestGroupOutcomes<'t>, TestGroupOutcomes<'t>>
    where
        F: FnOnce() -> TestGroupOutcomes<'t>,
    {
        ControlFlow::Continue(f())
    }
}
