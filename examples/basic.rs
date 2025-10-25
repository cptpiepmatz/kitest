use std::{
    borrow::Cow,
    io::{self, Stdout, Write},
};

use kitest::{
    filter::NoFilter,
    formatter::{FmtTestOutcome, TestFormatter},
    ignore::NoIgnore,
    meta::{TestFnHandle, TestMeta},
    panic_handler::NoPanicHandler,
    runner::SimpleRunner,
};

fn test_a() {}

fn test_b() {}

const TESTS: &[TestMeta] = &[
    TestMeta {
        function: TestFnHandle::from_static_obj(&|| test_a()),
        name: Cow::Borrowed("test_a"),
        ignore: (false, None),
        should_panic: (false, None),
        extra: (),
    },
    TestMeta {
        function: TestFnHandle::from_static_obj(&|| test_b()),
        name: Cow::Borrowed("test_b"),
        ignore: (false, None),
        should_panic: (false, None),
        extra: (),
    },
];

struct BasicFormatter(Stdout);

struct BasicTestOutcome<'m> {
    name: &'m str,
}

impl<'m> From<FmtTestOutcome<'m, '_, ()>> for BasicTestOutcome<'m> {
    fn from(value: FmtTestOutcome<'m, '_, ()>) -> Self {
        BasicTestOutcome {
            name: value.meta.name.as_ref(),
        }
    }
}

impl<'m> TestFormatter<'m, ()> for BasicFormatter {
    type Error = io::Error;

    type RunInit = ();
    fn fmt_run_init(&mut self, _: Self::RunInit) -> io::Result<()> {
        writeln!(self.0, "started testing")
    }

    type TestOutcome = BasicTestOutcome<'m>;
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
