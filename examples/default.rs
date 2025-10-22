use std::borrow::Cow;

use kitest::{
    filter::DefaultFilter,
    formatter::NoFormatter,
    ignore::DefaultIgnore,
    meta::{TestFnHandle, TestMeta},
    panic_handler::DefaultPanicHandler,
    runner::DefaultRunner,
};

fn test_a() {
    std::thread::sleep(std::time::Duration::from_secs(3));
}

fn test_b() {
    std::thread::sleep(std::time::Duration::from_secs(1));
}

fn test_c() {
    std::thread::sleep(std::time::Duration::from_secs(2));
}

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
    TestMeta {
        function: TestFnHandle::from_static_obj(&|| test_c()),
        name: Cow::Borrowed("test_c"),
        ignore: (false, None),
        should_panic: (false, None),
        extra: (),
    },
];

fn main() {
    kitest::run_tests(
        TESTS,
        DefaultFilter::default(),
        DefaultRunner::default(),
        DefaultIgnore::default(),
        DefaultPanicHandler::default(),
        NoFormatter,
    );
}
