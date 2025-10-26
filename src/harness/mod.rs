mod test;
use std::io;

pub use test::TestHarness;

mod grouped_test;
pub use grouped_test::GroupedTestHarness;

use crate::{
    filter::DefaultFilter, formatter::pretty::PrettyFormatter, ignore::DefaultIgnore,
    panic_handler::DefaultPanicHandler, runner::DefaultRunner, test::Test,
};

pub fn harness<'t, Extra>(
    tests: &'t [Test<Extra>],
) -> TestHarness<
    't,
    Extra,
    DefaultFilter,
    DefaultIgnore,
    DefaultPanicHandler,
    DefaultRunner,
    PrettyFormatter<io::Stdout>,
> {
    TestHarness {
        tests,
        filter: DefaultFilter::default(),
        ignore: DefaultIgnore::Default,
        panic_handler: DefaultPanicHandler::default(),
        runner: DefaultRunner::default(),
        formatter: PrettyFormatter::default(),
    }
}

trait FmtErrors<E> {
    fn push_on_error<T>(&mut self, data: (&'static str, Result<T, E>));
}

impl<E> FmtErrors<E> for Vec<(&'static str, E)> {
    fn push_on_error<T>(&mut self, (name, res): (&'static str, Result<T, E>)) {
        if let Err(err) = res {
            self.push((name, err));
        }
    }
}

macro_rules! named_fmt {
    ($fmt:ident.$method:ident($expr:expr)) => {
        (stringify!($method), $fmt.$method($expr))
    };
}

pub(self) use named_fmt;
