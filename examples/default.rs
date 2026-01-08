use std::{borrow::Cow, process::Termination};

use kitest::{
    formatter::{common::color::ColorSetting, pretty::PrettyFormatter},
    prelude::*,
};

fn test_a() {
    std::thread::sleep(std::time::Duration::from_millis(300));
}

fn test_b() {
    std::thread::sleep(std::time::Duration::from_millis(100));
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
            ignore: IgnoreStatus::IgnoreWithReason(Cow::Borrowed("we don't need this")),
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
    kitest::harness(TESTS)
        .with_formatter(PrettyFormatter::default().with_color_settings(ColorSetting::Always))
        .run()
}
