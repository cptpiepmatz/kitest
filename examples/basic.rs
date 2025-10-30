use std::{
    borrow::Cow,
    io::{self, Stdout, Write},
};

use kitest::{
    filter::NoFilter,
    formatter::{FmtTestOutcome, TestFormatter},
    ignore::{IgnoreStatus, NoIgnore},
    panic_handler::{NoPanicHandler, PanicExpectation},
    runner::SimpleRunner,
    test::{Test, TestFnHandle, TestMeta},
};

fn test_a() {}

fn test_b() {}

const TESTS: &[Test] = &[
    Test::new(
        TestFnHandle::from_static_obj(&|| test_a()),
        TestMeta {
            name: Cow::Borrowed("test_a"),
            ignore: IgnoreStatus::Run,
            should_panic: PanicExpectation::ShouldNotPanic,
            extra: (),
        },
    ),
    Test::new(
        TestFnHandle::from_static_obj(&|| test_b()),
        TestMeta {
            name: Cow::Borrowed("test_b"),
            ignore: IgnoreStatus::Run,
            should_panic: PanicExpectation::ShouldNotPanic,
            extra: (),
        },
    ),
];

struct BasicFormatter(Stdout);

struct BasicTestOutcome<'t> {
    name: &'t str,
}

impl<'t> From<FmtTestOutcome<'t, '_, ()>> for BasicTestOutcome<'t> {
    fn from(value: FmtTestOutcome<'t, '_, ()>) -> Self {
        BasicTestOutcome {
            name: value.meta.name.as_ref(),
        }
    }
}

impl<'t> TestFormatter<'t, ()> for BasicFormatter {
    type Error = io::Error;

    type RunInit = ();
    fn fmt_run_init(&mut self, _: Self::RunInit) -> io::Result<()> {
        writeln!(self.0, "started testing")
    }

    type TestOutcome = BasicTestOutcome<'t>;
    fn fmt_test_outcome(&mut self, data: Self::TestOutcome) -> io::Result<()> {
        writeln!(self.0, "test {:?} done", data.name)
    }

    type RunStart = ();
    type TestIgnored = ();
    type TestStart = ();
    type RunOutcomes = ();
}

fn main() {
    kitest::harness(TESTS)
        .with_filter(NoFilter)
        .with_runner(SimpleRunner)
        .with_ignore(NoIgnore)
        .with_panic_handler(NoPanicHandler)
        .with_formatter(BasicFormatter(io::stdout()))
        .run();
}
