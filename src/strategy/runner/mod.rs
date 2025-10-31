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

pub trait TestRunner<Extra> {
    fn run<'t, 's, I, F>(
        &self,
        tests: I,
        scope: &'s Scope<'s, 't>,
    ) -> impl Iterator<Item = (&'t TestMeta<Extra>, TestOutcome)>
    where
        I: ExactSizeIterator<Item = (F, &'t TestMeta<Extra>)>,
        F: (Fn() -> TestStatus) + Send + 's,
        Extra: 't;

    fn worker_count(&self, tests_count: usize) -> NonZeroUsize;
}
