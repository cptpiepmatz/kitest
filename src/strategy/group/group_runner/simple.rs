use std::ops::ControlFlow;

use crate::group::{TestGroupOutcomes, TestGroupRunner};

#[derive(Debug, Default)]
pub struct SimpleGroupRunner {
    keep_going: bool,
}

impl SimpleGroupRunner {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_keep_going(self, keep_going: bool) -> Self {
        Self { keep_going }
    }
}

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
        let outcomes = f();
        let any_bad = outcomes.iter().any(|(_, outcome)| outcome.is_bad());
        match (self.keep_going, any_bad) {
            (false, true) => ControlFlow::Break(outcomes),
            (true, _) | (false, false) => ControlFlow::Continue(outcomes),
        }
    }
}
