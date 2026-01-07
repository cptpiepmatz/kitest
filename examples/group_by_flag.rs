use std::{borrow::Cow, fmt::Display, process::Termination};

use kitest::{prelude::*, runner::SimpleRunner};

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
enum Flag {
    A,
    B,
}

impl Display for Flag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Flag::A => f.write_str("a"),
            Flag::B => f.write_str("b"),
        }
    }
}

const TESTS: &[Test<Flag>] = &[
    Test::new(
        TestFnHandle::from_static_obj(&|| ()),
        TestMeta {
            name: Cow::Borrowed("a"),
            ignore: IgnoreStatus::Run,
            should_panic: PanicExpectation::ShouldNotPanic,
            origin: origin!(),
            extra: Flag::A,
        },
    ),
    Test::new(
        TestFnHandle::from_static_obj(&|| ()),
        TestMeta {
            name: Cow::Borrowed("b"),
            ignore: IgnoreStatus::Run,
            should_panic: PanicExpectation::ShouldNotPanic,
            origin: origin!(),
            extra: Flag::B,
        },
    ),
    Test::new(
        TestFnHandle::from_static_obj(&|| ()),
        TestMeta {
            name: Cow::Borrowed("c"),
            ignore: IgnoreStatus::Run,
            should_panic: PanicExpectation::ShouldNotPanic,
            origin: origin!(),
            extra: Flag::A,
        },
    ),
    Test::new(
        TestFnHandle::from_static_obj(&|| ()),
        TestMeta {
            name: Cow::Borrowed("d"),
            ignore: IgnoreStatus::Run,
            should_panic: PanicExpectation::ShouldNotPanic,
            origin: origin!(),
            extra: Flag::A,
        },
    ),
    Test::new(
        TestFnHandle::from_static_obj(&|| ()),
        TestMeta {
            name: Cow::Borrowed("e"),
            ignore: IgnoreStatus::Run,
            should_panic: PanicExpectation::ShouldNotPanic,
            origin: origin!(),
            extra: Flag::B,
        },
    ),
];

fn main() -> impl Termination {
    kitest::harness(TESTS)
        .with_grouper(|meta: &TestMeta<Flag>| meta.extra)
        .with_runner(SimpleRunner::default())
        // .with_formatter(FlagFormatter(io::stdout()))
        .run()
}
