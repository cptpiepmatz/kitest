use std::ops::ControlFlow;

use crate::group::{TestGroupOutcomes, TestGroupRunner};

#[derive(Debug, Default)]
pub struct DefaultGroupRunner {
    keep_going: bool,
}

impl DefaultGroupRunner {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_keep_going(self, keep_going: bool) -> Self {
        Self {keep_going, ..self}
    }
}

impl<'t, Extra, GroupKey, GroupCtx> TestGroupRunner<'t, Extra, GroupKey, GroupCtx> for DefaultGroupRunner {
    fn run_group<F>(
        &self,
        f: F,
        _: &GroupKey,
        _: Option<&GroupCtx>,
    ) -> ControlFlow<TestGroupOutcomes<'t>, TestGroupOutcomes<'t>>
    where
        F: FnOnce() -> TestGroupOutcomes<'t> {
            let outcomes = f();
            let any_failed = outcomes.iter().any(|(_, outcome)| outcome.failed());
        match (self.keep_going, any_failed) {
            (false, true) => ControlFlow::Break(outcomes),
            (true, _) | (false, false) => ControlFlow::Continue(outcomes),
        }
    }
}