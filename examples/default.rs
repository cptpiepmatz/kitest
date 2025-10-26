use std::borrow::Cow;

use kitest::{
    filter::DefaultFilter,
    formatter::pretty::PrettyFormatter,
    ignore::DefaultIgnore,
    panic_handler::DefaultPanicHandler,
    runner::DefaultRunner,
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
    Test::new(
        TestFnHandle::from_static_obj(&|| test_c()),
        TestMeta {
            name: Cow::Borrowed("test_c"),
            ignore: (false, None),
            should_panic: (false, None),
            extra: (),
        },
    ),
];

fn main() {
    kitest::run_tests(
        TESTS,
        DefaultFilter::default(),
        DefaultRunner::default(),
        DefaultIgnore::default(),
        DefaultPanicHandler::default(),
        PrettyFormatter::default(),
    );
}
