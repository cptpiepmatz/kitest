use std::borrow::Cow;

use kitest::{
    filter::NoFilter,
    formatter::NoFormatter,
    ignore::TestIgnore,
    meta::{TestFnHandle, TestMeta},
};

enum Speed {
    Fast,
    Slow,
}

fn test_fast_ok() {}
fn test_fast_fail() {}
fn test_slow_expensive() {}

const TESTS: &[TestMeta<Speed>] = &[
    TestMeta {
        function: TestFnHandle::from_static_obj(&|| test_fast_ok()),
        name: Cow::Borrowed("test_fast_ok"),
        ignore: (false, None),
        should_panic: (false, None),
        extra: Speed::Fast,
    },
    TestMeta {
        function: TestFnHandle::from_static_obj(&|| test_fast_fail()),
        name: Cow::Borrowed("test_fast_fail"),
        ignore: (false, None),
        should_panic: (false, None),
        extra: Speed::Fast,
    },
    TestMeta {
        function: TestFnHandle::from_static_obj(&|| test_slow_expensive()),
        name: Cow::Borrowed("test_slow_expensive"),
        ignore: (false, None),
        should_panic: (false, None),
        extra: Speed::Slow,
    },
];

fn main() {
    kitest::list_tests(TESTS, NoFilter, IgnoreSlow, NoFormatter);
}

struct IgnoreSlow;

impl TestIgnore<Speed> for IgnoreSlow {
    fn ignore(&self, meta: &TestMeta<Speed>) -> (bool, Option<Cow<'static, str>>) {
        match meta.extra {
            Speed::Fast => (false, None),
            Speed::Slow => (true, Some("too slow".into())),
        }
    }
}
