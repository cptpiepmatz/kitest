use std::ops::ControlFlow;

use crate::group::{TestGroupOutcomes, TestGroupRunner};

#[derive(Debug, Default)]
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