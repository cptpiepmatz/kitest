use std::{
    borrow::Cow,
    fmt::Display,
    io::{self, Stdout, Write},
};

use kitest::{
    formatter::{
        FmtGroupOutcomes, FmtGroupStart, FmtTestStart, GroupedTestFormatter, TestFormatter,
    },
    runner::SimpleRunner,
    test::{Test, TestFnHandle, TestMeta},
};

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
enum Flag {
    A,
    B,
}

impl Display for Flag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Flag::A => f.write_str("A"),
            Flag::B => f.write_str("B"),
        }
    }
}

const TESTS: &[Test<Flag>] = &[
    Test::new(
        TestFnHandle::from_static_obj(&|| ()),
        TestMeta {
            name: Cow::Borrowed("a"),
            ignore: (false, None),
            should_panic: (false, None),
            extra: Flag::A,
        },
    ),
    Test::new(
        TestFnHandle::from_static_obj(&|| ()),
        TestMeta {
            name: Cow::Borrowed("b"),
            ignore: (false, None),
            should_panic: (false, None),
            extra: Flag::B,
        },
    ),
    Test::new(
        TestFnHandle::from_static_obj(&|| ()),
        TestMeta {
            name: Cow::Borrowed("c"),
            ignore: (false, None),
            should_panic: (false, None),
            extra: Flag::A,
        },
    ),
    Test::new(
        TestFnHandle::from_static_obj(&|| ()),
        TestMeta {
            name: Cow::Borrowed("d"),
            ignore: (false, None),
            should_panic: (false, None),
            extra: Flag::A,
        },
    ),
    Test::new(
        TestFnHandle::from_static_obj(&|| ()),
        TestMeta {
            name: Cow::Borrowed("e"),
            ignore: (false, None),
            should_panic: (false, None),
            extra: Flag::B,
        },
    ),
];

struct FlagFormatter(Stdout);

struct TestName<'t>(&'t str);

impl<'t, Extra> From<FmtTestStart<'t, Extra>> for TestName<'t> {
    fn from(value: FmtTestStart<'t, Extra>) -> Self {
        Self(value.meta.name.as_ref())
    }
}

impl<'t> TestFormatter<'t, Flag> for FlagFormatter {
    type Error = io::Error;

    type TestStart = TestName<'t>;
    fn fmt_test_start(&mut self, TestName(name): Self::TestStart) -> io::Result<()> {
        writeln!(self.0, "testing test {name}")
    }

    type RunInit = ();
    type RunStart = ();
    type TestIgnored = ();
    type TestOutcome = ();
    type RunOutcomes = ();
}

struct Group(Flag);

impl From<FmtGroupStart<'_, Flag>> for Group {
    fn from(value: FmtGroupStart<'_, Flag>) -> Self {
        Self(*value.key)
    }
}

impl From<FmtGroupOutcomes<'_, '_, '_, Flag>> for Group {
    fn from(value: FmtGroupOutcomes<'_, '_, '_, Flag>) -> Self {
        Self(*value.key)
    }
}

impl GroupedTestFormatter<'_, Flag, Flag> for FlagFormatter {
    type GroupStart = Group;
    fn fmt_group_start(&mut self, Group(flag): Self::GroupStart) -> std::io::Result<()> {
        writeln!(self.0, "testing group {flag}")
    }

    type GroupOutcomes = Group;
    fn fmt_group_outcomes(&mut self, Group(flag): Self::GroupOutcomes) -> std::io::Result<()> {
        writeln!(self.0, "tested group {flag}")
    }

    type GroupedRunStart = ();
    type GroupedRunOutcomes = ();
}

fn main() {
    kitest::harness(TESTS)
        .with_grouper(|meta: &TestMeta<Flag>| meta.extra)
        .with_runner(SimpleRunner::default())
        .with_formatter(FlagFormatter(io::stdout()))
        .run();
}
