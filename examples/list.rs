use std::borrow::Cow;

use kitest::{
    filter::NoFilter,
    ignore::{IgnoreStatus, TestIgnore},
    panic_handler::PanicExpectation,
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
            ignore: IgnoreStatus::Run,
            should_panic: PanicExpectation::ShouldNotPanic,
            extra: Speed::Fast,
        },
    ),
    Test::new(
        TestFnHandle::from_static_obj(&|| test_fast_fail()),
        TestMeta {
            name: Cow::Borrowed("test_fast_fail"),
            ignore: IgnoreStatus::Run,
            should_panic: PanicExpectation::ShouldNotPanic,
            extra: Speed::Fast,
        },
    ),
    Test::new(
        TestFnHandle::from_static_obj(&|| test_slow_expensive()),
        TestMeta {
            name: Cow::Borrowed("test_slow_expensive"),
            ignore: IgnoreStatus::Run,
            should_panic: PanicExpectation::ShouldNotPanic,
            extra: Speed::Slow,
        },
    ),
];

fn main() {
    kitest::harness(TESTS)
        .with_filter(NoFilter)
        .with_ignore(IgnoreSlow)
        .list();
}

struct IgnoreSlow;

impl TestIgnore<Speed> for IgnoreSlow {
    fn ignore(&self, meta: &TestMeta<Speed>) -> IgnoreStatus {
        match meta.extra {
            Speed::Fast => IgnoreStatus::Run,
            Speed::Slow => IgnoreStatus::IgnoreWithReason("too slow".into()),
        }
    }
}
