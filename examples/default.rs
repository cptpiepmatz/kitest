use std::borrow::Cow;

use kitest::{
    filter::DefaultFilter,
    ignore::DefaultIgnore,
    meta::{TestFnHandle, TestMeta},
    panic_handler::DefaultPanicHandler,
    runner::DefaultRunner,
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

fn main() {
    kitest::run_tests(
        TESTS.iter(),
        DefaultFilter::default(),
        DefaultRunner::default(),
        DefaultIgnore::default(),
        DefaultPanicHandler::default(),
    );
}
