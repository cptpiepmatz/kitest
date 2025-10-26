use std::{
    borrow::Cow,
    fmt::Display,
    io::{self, Stdout, Write},
};

use kitest::{
    filter::DefaultFilter,
    formatter::{
        FmtGroupOutcomes, FmtGroupStart, FmtTestStart, GroupedTestFormatter, TestFormatter,
    },
    group::{SimpleGroupRunner, TestGroupHashMap},
    ignore::DefaultIgnore,
    meta::{Test, TestFnHandle, TestMeta},
    panic_handler::DefaultPanicHandler,
    runner::SimpleRunner,
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

struct TestName<'m>(&'m str);

impl<'m, Extra> From<FmtTestStart<'m, Extra>> for TestName<'m> {
    fn from(value: FmtTestStart<'m, Extra>) -> Self {
        Self(value.meta.name.as_ref())
    }
}

impl<'m> TestFormatter<'m, Flag> for FlagFormatter {
    type Error = io::Error;

    type TestStart = TestName<'m>;
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
    kitest::run_grouped_tests(
        TESTS,
        DefaultFilter::default(),
        |meta: &TestMeta<Flag>| meta.extra,
        TestGroupHashMap::<'_, _, _>::default(),
        SimpleGroupRunner::default(),
        SimpleRunner::default(),
        DefaultIgnore::default(),
        DefaultPanicHandler::default(),
        FlagFormatter(io::stdout()),
    );
}
