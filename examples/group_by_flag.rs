use std::{borrow::Cow, fmt::Display, io::{self, Stdout, Write}};

use kitest::{
    filter::DefaultFilter, formatter::{FmtGroupOutcomes, FmtGroupStart, FmtTestStart, GroupedTestFormatter, NoFormatter, TestFormatter}, group::{SimpleGroupRunner, TestGroupHashMap}, ignore::DefaultIgnore, meta::{TestFnHandle, TestMeta}, panic_handler::DefaultPanicHandler, runner::{DefaultRunner, SimpleRunner}
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

const TESTS: &[TestMeta<Flag>] = &[
    TestMeta {
        function: TestFnHandle::from_static_obj(&|| ()),
        name: Cow::Borrowed("a"),
        ignore: (false, None),
        should_panic: (false, None),
        extra: Flag::A,
    },
    TestMeta {
        function: TestFnHandle::from_static_obj(&|| ()),
        name: Cow::Borrowed("b"),
        ignore: (false, None),
        should_panic: (false, None),
        extra: Flag::B,
    },
    TestMeta {
        function: TestFnHandle::from_static_obj(&|| ()),
        name: Cow::Borrowed("c"),
        ignore: (false, None),
        should_panic: (false, None),
        extra: Flag::A,
    },
    TestMeta {
        function: TestFnHandle::from_static_obj(&|| ()),
        name: Cow::Borrowed("d"),
        ignore: (false, None),
        should_panic: (false, None),
        extra: Flag::A,
    },
    TestMeta {
        function: TestFnHandle::from_static_obj(&|| ()),
        name: Cow::Borrowed("e"),
        ignore: (false, None),
        should_panic: (false, None),
        extra: Flag::B,
    },
];

struct FlagFormatter(Stdout);

struct TestName(Cow<'static, str>);

impl<Extra> From<FmtTestStart<'_, Extra>> for TestName {
    fn from(value: FmtTestStart<'_, Extra>) -> Self {
        Self(value.meta.name.clone())
    }
}

impl TestFormatter<Flag> for FlagFormatter {
    type TestStart = TestName;
    fn fmt_test_start(&mut self, TestName(name): Self::TestStart) -> std::io::Result<()> {
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

impl From<FmtGroupOutcomes<'_, '_, Flag>> for Group {
    fn from(value: FmtGroupOutcomes<'_, '_, Flag>) -> Self {
        Self(*value.key)
    }
}

impl GroupedTestFormatter<Flag, Flag> for FlagFormatter {
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
