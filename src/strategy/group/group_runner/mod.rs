use std::ops::ControlFlow;

use crate::outcome::TestOutcome;

mod simple;
pub use simple::*;

mod default;
pub use default::*;

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
