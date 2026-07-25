use std::{borrow::Cow, process::Termination};

use kitest::{formatter::debug::DebugFormatter, prelude::*};

fn test_a() {
    std::thread::sleep(std::time::Duration::from_millis(400));
}

fn test_b() {
    std::thread::sleep(std::time::Duration::from_millis(1000));
}

fn test_c() {
    std::thread::sleep(std::time::Duration::from_millis(200));
}

const TESTS: &[Test] = &[
    Test::new(
        TestFnHandle::from_static_obj(&|| test_a()),
        TestMeta {
            name: Cow::Borrowed("test_a"),
            ignore: IgnoreStatus::Run,
            should_panic: PanicExpectation::ShouldNotPanic,
            origin: origin!(),
            extra: (),
        },
    ),
    Test::new(
        TestFnHandle::from_static_obj(&|| test_b()),
        TestMeta {
            name: Cow::Borrowed("test_b"),
            ignore: IgnoreStatus::Run,
            should_panic: PanicExpectation::ShouldNotPanic,
            origin: origin!(),
            extra: (),
        },
    ),
    Test::new(
        TestFnHandle::from_static_obj(&|| test_c()),
        TestMeta {
            name: Cow::Borrowed("test_c"),
            ignore: IgnoreStatus::Run,
            should_panic: PanicExpectation::ShouldNotPanic,
            origin: origin!(),
            extra: (),
        },
    ),
];

fn main() -> impl Termination {
    let formatter = DebugFormatter::default()
        .with_no_formatter()
        .with_test_start_formatter(|f, d| writeln!(f, "test start: {}", d.meta.name))
        .with_test_outcome_formatter(|f, d| writeln!(f, "test end: {}", d.meta.name))
        .with_default_run_outcomes_formatter();

    kitest::harness(TESTS).with_formatter(formatter).run()
}
