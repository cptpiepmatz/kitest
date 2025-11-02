use std::{borrow::Cow, fmt::Debug, ops::Deref, panic::RefUnwindSafe};

use crate::{ignore::IgnoreStatus, panic::PanicExpectation};

#[derive(Debug, Default)]
#[non_exhaustive]
pub struct Test<Extra = ()> {
    function: TestFnHandle,
    pub meta: TestMeta<Extra>,
}

impl<Extra> Test<Extra> {
    pub const fn new(function: TestFnHandle, meta: TestMeta<Extra>) -> Self {
        Self { function, meta }
    }

    pub(crate) fn call(&self) -> TestResult {
        self.function.call()
    }
}

impl<Extra> Deref for Test<Extra> {
    type Target = TestMeta<Extra>;

    fn deref(&self) -> &Self::Target {
        &self.meta
    }
}

#[derive(Debug, Clone, Default)]
pub struct TestMeta<Extra = ()> {
    pub name: Cow<'static, str>,
    pub ignore: IgnoreStatus,
    pub should_panic: PanicExpectation,
    pub extra: Extra,
}

#[non_exhaustive]
pub enum TestFnHandle {
    Ptr(fn() -> TestResult),
    Owned(Box<dyn TestFn + Send + Sync + RefUnwindSafe>),
    Static(&'static (dyn TestFn + Send + Sync + RefUnwindSafe)),
}

impl Debug for TestFnHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Ptr(ptr) => f.debug_tuple("Ptr").field(ptr).finish(),
            Self::Owned(_) => write!(f, "Owned(...)"),
            Self::Static(_) => write!(f, "Static(...)"),
        }
    }
}

impl Default for TestFnHandle {
    fn default() -> Self {
        Self::Static(&|| {})
    }
}

impl TestFnHandle {
    pub const fn from_const_fn(f: fn() -> TestResult) -> Self {
        Self::Ptr(f)
    }

    pub fn from_boxed<F, T>(f: F) -> Self
    where
        F: Fn() -> T + Send + Sync + RefUnwindSafe + 'static,
        T: Into<TestResult>,
    {
        Self::Owned(Box::new(f))
    }

    pub const fn from_static_obj(f: &'static (dyn TestFn + Send + Sync + RefUnwindSafe)) -> Self {
        Self::Static(f)
    }

    pub fn call(&self) -> TestResult {
        match self {
            Self::Ptr(f) => f(),
            Self::Owned(f) => f.call_test(),
            Self::Static(f) => f.call_test(),
        }
    }
}

pub trait TestFn {
    fn call_test(&self) -> TestResult;
}

impl<F, T> TestFn for F
where
    F: Fn() -> T,
    T: Into<TestResult>,
{
    fn call_test(&self) -> TestResult {
        (self)().into()
    }
}

#[derive(Debug)]
pub struct TestResult(pub Result<(), String>);

impl From<()> for TestResult {
    fn from(_: ()) -> Self {
        Self(Ok(()))
    }
}

impl<E: Debug> From<Result<(), E>> for TestResult {
    fn from(v: Result<(), E>) -> Self {
        TestResult(v.map_err(|e| format!("{e:#?}")))
    }
}
