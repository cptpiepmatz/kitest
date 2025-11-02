use std::ops::ControlFlow;

use crate::outcome::TestOutcome;

pub type TestGroupOutcomes<'t> = Vec<(&'t str, TestOutcome)>;

pub trait TestGroupRunner<'t, Extra, GroupKey, GroupCtx> {
    fn run_group<F>(
        &self,
        f: F,
        key: &GroupKey,
        ctx: Option<&GroupCtx>,
    ) -> ControlFlow<TestGroupOutcomes<'t>, TestGroupOutcomes<'t>>
    where
        F: FnOnce() -> TestGroupOutcomes<'t>;
}

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
