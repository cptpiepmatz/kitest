use std::{borrow::Cow, io::{self, Stdout, Write}};

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

struct BasicTestOutcome {
    name: Cow<'static, str>,
}

impl From<FmtTestOutcome<'_, '_, ()>> for BasicTestOutcome {
    fn from(value: FmtTestOutcome<'_, '_, ()>) -> Self {
        BasicTestOutcome { name: value.meta.name.clone() }
    }
}

impl TestFormatter<()> for BasicFormatter {
    type RunInit = ();
    fn fmt_run_init(&mut self, _: Self::RunInit) -> std::io::Result<()> {
        writeln!(self.0, "started testing")
    }

    type TestOutcome = BasicTestOutcome;
    fn fmt_test_outcome(&mut self, data: Self::TestOutcome) -> std::io::Result<()> {
        writeln!(self.0, "test {:?} done", data.name)
    }

    type RunStart = ();
    type TestIgnored = ();
    type TestStart = ();
    type RunOutcomes = ();
}

fn main() {
    kitest::run_tests(
        TESTS.iter(),
        NoFilter,
        SimpleRunner,
        NoIgnore,
        NoPanicHandler,
        BasicFormatter(io::stdout()),
    );
}
