use std::{
    borrow::Cow,
    io::{self, Stdout, Write},
};

use kitest::{
    filter::NoFilter,
    formatter::{FmtTestOutcome, TestFormatter},
    ignore::NoIgnore,
    test::{Test, TestFnHandle, TestMeta},
    panic_handler::NoPanicHandler,
    runner::SimpleRunner,
};

fn test_a() {}

fn test_b() {}

const TESTS: &[Test] = &[
    Test::new(
        TestFnHandle::from_static_obj(&|| test_a()),
        TestMeta {
            name: Cow::Borrowed("test_a"),
            ignore: (false, None),
            should_panic: (false, None),
            extra: (),
        },
    ),
    Test::new(
        TestFnHandle::from_static_obj(&|| test_b()),
        TestMeta {
            name: Cow::Borrowed("test_b"),
            ignore: (false, None),
            should_panic: (false, None),
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
    kitest::run_tests(
        TESTS,
        NoFilter,
        SimpleRunner,
        NoIgnore,
        NoPanicHandler,
        BasicFormatter(io::stdout()),
    );
}
