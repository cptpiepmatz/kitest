#![allow(clippy::type_complexity)]

use std::io;

use crate::{
    filter::DefaultFilter,
    formatter::{
        label::{FromGroupKey, GroupLabel},
        pretty::PrettyFormatter,
    },
    ignore::DefaultIgnore,
    panic::DefaultPanicHandler,
    runner::DefaultRunner,
    test::Test,
};

mod test;
pub use test::TestHarness;

mod grouped_test;
pub use grouped_test::GroupedTestHarness;

pub fn harness<'t, Extra>(
    tests: &'t [Test<Extra>],
) -> TestHarness<
    't,
    Extra,
    DefaultFilter,
    DefaultIgnore,
    DefaultPanicHandler,
    DefaultRunner,
    PrettyFormatter<io::Stdout, GroupLabel<FromGroupKey>>,
> {
    TestHarness {
        tests,
        filter: DefaultFilter::default(),
        ignore: DefaultIgnore::Default,
        panic_handler: DefaultPanicHandler,
        runner: DefaultRunner::default(),
        formatter: PrettyFormatter::default(),
    }
}

trait FmtErrors<E> {
    fn push_on_error<T>(&mut self, res: Result<T, E>);
}

impl<E> FmtErrors<E> for Vec<E> {
    fn push_on_error<T>(&mut self, res: Result<T, E>) {
        if let Err(err) = res {
            self.push(err);
        }
    }
}
