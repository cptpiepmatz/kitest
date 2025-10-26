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
