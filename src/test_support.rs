use std::{borrow::Cow, panic::RefUnwindSafe};

use crate::{
    TestHarness, capture::DefaultPanicHookProvider, filter::NoFilter, formatter::no::NoFormatter, ignore::{IgnoreStatus, NoIgnore}, panic::{NoPanicHandler, PanicExpectation}, runner::SimpleRunner, test::{Test, TestFn, TestFnHandle, TestMeta}
};

pub struct BuildTest<Extra> {
    pub func: TestFnHandle,
    pub name: Cow<'static, str>,
    pub ignore: IgnoreStatus,
    pub should_panic: PanicExpectation,
    pub extra: Extra,
}

impl Default for BuildTest<()> {
    fn default() -> Self {
        Self {
            func: TestFnHandle::Static(&|| ()),
            name: Default::default(),
            ignore: Default::default(),
            should_panic: Default::default(),
            extra: Default::default(),
        }
    }
}

impl<Extra> From<BuildTest<Extra>> for Test<Extra> {
    fn from(value: BuildTest<Extra>) -> Self {
        Test::new(
            value.func,
            TestMeta {
                name: value.name,
                ignore: value.ignore,
                should_panic: value.should_panic,
                extra: value.extra,
            },
        )
    }
}

impl<F> From<F> for TestFnHandle
where
    F: TestFn + Send + Sync + RefUnwindSafe + 'static,
{
    fn from(value: F) -> Self {
        TestFnHandle::Owned(Box::new(value))
    }
}

macro_rules! test {
    {$($field:ident: $value:expr),* $(,)?} => {
        $crate::test::Test::from($crate::test_support::BuildTest {
            $($field: From::from($value),)*
            ..($crate::test_support::BuildTest {
                name: concat!(module_path!(), "::", file!(), ":", line!(), ":", column!()).into(),
                ..Default::default()
            })
        })
    };
}

pub(crate) use test;

pub fn harness<'t>(
    tests: &'t [Test],
) -> TestHarness<'t, (), NoFilter, NoIgnore, NoPanicHandler, SimpleRunner<DefaultPanicHookProvider>, NoFormatter> {
    TestHarness {
        tests,
        filter: NoFilter,
        ignore: NoIgnore,
        panic_handler: NoPanicHandler,
        runner: SimpleRunner::default(),
        formatter: NoFormatter,
    }
}

macro_rules! nonzero {
    (0) => {
        compile_error!("0 is zero")
    };

    ($value:literal) => {
        std::convert::TryFrom::try_from($value).unwrap()
    };
}

pub(crate) use nonzero;
