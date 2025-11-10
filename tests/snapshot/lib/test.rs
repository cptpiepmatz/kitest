use std::panic::RefUnwindSafe;

use kitest::prelude::*;

pub struct BuildTest<Extra> {
    pub func: WrappedTestFnHandle,
    pub name: Cow<'static, str>,
    pub ignore: IgnoreStatus,
    pub should_panic: PanicExpectation,
    pub extra: Extra,
}

impl Default for BuildTest<()> {
    fn default() -> Self {
        Self {
            func: WrappedTestFnHandle(TestFnHandle::Static(&|| ())),
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
            value.func.0,
            TestMeta {
                name: value.name,
                ignore: value.ignore,
                should_panic: value.should_panic,
                extra: value.extra,
            },
        )
    }
}

pub struct WrappedTestFnHandle(TestFnHandle);

impl<F> From<F> for WrappedTestFnHandle
where
    F: TestFn + Send + Sync + RefUnwindSafe + 'static,
{
    fn from(value: F) -> Self {
        Self(TestFnHandle::Owned(Box::new(value)))
    }
}

macro_rules! test {
    {$($field:ident: $value:expr),* $(,)?} => {
        kitest::test::Test::from($crate::lib::test::BuildTest {
            $($field: From::from($value),)*
            ..($crate::lib::test::BuildTest {
                name: concat!(module_path!(), "::", file!(), ":", line!(), ":", column!()).into(),
                ..Default::default()
            })
        })
    };
}

pub(crate) use test;
