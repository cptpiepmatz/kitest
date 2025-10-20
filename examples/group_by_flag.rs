use std::borrow::Cow;

use kitest::{
    filter::DefaultFilter, group::{SimpleGroupRunner, TestGroupHashMap}, ignore::DefaultIgnore, meta::{TestFnHandle, TestMeta}, panic_handler::DefaultPanicHandler, runner::{DefaultRunner, SimpleRunner}
};

fn test_a() {}

fn test_b() {}

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
enum Flag {
    A,
    B,
}

const TESTS: &[TestMeta<Flag>] = &[
    TestMeta {
        function: TestFnHandle::from_static_obj(&|| test_a()),
        name: Cow::Borrowed("test_a"),
        ignore: (false, None),
        should_panic: (false, None),
        extra: Flag::A,
    },
    TestMeta {
        function: TestFnHandle::from_static_obj(&|| test_b()),
        name: Cow::Borrowed("test_b"),
        ignore: (false, None),
        should_panic: (false, None),
        extra: Flag::B,
    },
];

fn main() {
    kitest::run_grouped_tests(
        TESTS.iter(),
        DefaultFilter::default(),
        |meta: &TestMeta<Flag>| meta.extra,
        TestGroupHashMap::<'_, _, _>::default(),
        SimpleGroupRunner::default(),
        SimpleRunner::default(),
        DefaultIgnore::default(),
        DefaultPanicHandler::default(),
    );
}
