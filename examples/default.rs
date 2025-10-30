use std::borrow::Cow;

use kitest::{
    ignore::IgnoreStatus,
    panic_handler::PanicExpectation,
    test::{Test, TestFnHandle, TestMeta},
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
    Test::new(
        TestFnHandle::from_static_obj(&|| test_c()),
        TestMeta {
            name: Cow::Borrowed("test_c"),
            ignore: IgnoreStatus::Run,
            should_panic: PanicExpectation::ShouldNotPanic,
            extra: (),
        },
    ),
];

fn main() {
    kitest::harness(TESTS).run();
}
