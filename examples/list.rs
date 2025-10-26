use std::borrow::Cow;

use kitest::{
    filter::NoFilter,
    formatter::pretty::PrettyFormatter,
    ignore::TestIgnore,
    test::{Test, TestFnHandle, TestMeta},
};

enum Speed {
    Fast,
    Slow,
}

fn test_fast_ok() {}
fn test_fast_fail() {}
fn test_slow_expensive() {}

const TESTS: &[Test<Speed>] = &[
    Test::new(
        TestFnHandle::from_static_obj(&|| test_fast_ok()),
        TestMeta {
            name: Cow::Borrowed("test_fast_ok"),
            ignore: (false, None),
            should_panic: (false, None),
            extra: Speed::Fast,
        },
    ),
    Test::new(
        TestFnHandle::from_static_obj(&|| test_fast_fail()),
        TestMeta {
            name: Cow::Borrowed("test_fast_fail"),
            ignore: (false, None),
            should_panic: (false, None),
            extra: Speed::Fast,
        },
    ),
    Test::new(
        TestFnHandle::from_static_obj(&|| test_slow_expensive()),
        TestMeta {
            name: Cow::Borrowed("test_slow_expensive"),
            ignore: (false, None),
            should_panic: (false, None),
            extra: Speed::Slow,
        },
    ),
];

fn main() {
    kitest::list_tests(TESTS, NoFilter, IgnoreSlow, PrettyFormatter::default());
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
