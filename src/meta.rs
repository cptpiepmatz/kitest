use std::{borrow::Cow, fmt::Debug};

pub struct TestResult(pub Result<(), Box<str>>);

impl From<()> for TestResult {
    fn from(_: ()) -> Self {
        Self(Ok(()))
    }
}

impl<E: Debug> From<Result<(), E>> for TestResult {
    fn from(v: Result<(), E>) -> Self {
        TestResult(v.map_err(|e| format!("{e:#?}").into_boxed_str()))
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

pub enum TestFnHandle {
    Ptr(fn() -> TestResult),
    Owned(Box<dyn TestFn + Send + Sync>),
    Static(&'static (dyn TestFn + Send + Sync)),
}

impl TestFnHandle {
    pub const fn from_const_fn(f: fn() -> TestResult) -> Self {
        Self::Ptr(f)
    }

    pub fn from_boxed<F, T>(f: F) -> Self
    where
        F: Fn() -> T + Send + Sync + 'static,
        T: Into<TestResult>,
    {
        Self::Owned(Box::new(f))
    }

    pub const fn from_static_obj(f: &'static (dyn TestFn + Send + Sync)) -> Self {
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

pub struct TestMeta<Extra = ()> {
    pub function: TestFnHandle,
    pub name: Cow<'static, str>,
    pub extra: Extra,
}

impl<Extra> TestMeta<Extra> {
    pub const fn new_const_fn(name: &'static str, f: fn() -> TestResult, extra: Extra) -> Self {
        Self {
            function: TestFnHandle::from_const_fn(f),
            name: Cow::Borrowed(name),
            extra,
        }
    }

    pub fn new_boxed<N, F, T>(name: N, f: F, extra: Extra) -> Self
    where
        N: Into<Cow<'static, str>>,
        F: Fn() -> T + Send + Sync + 'static,
        T: Into<TestResult>,
    {
        Self {
            function: TestFnHandle::from_boxed(f),
            name: name.into(),
            extra,
        }
    }

    pub const fn new_static_obj(
        name: &'static str,
        f: &'static (dyn TestFn + Send + Sync),
        extra: Extra,
    ) -> Self {
        Self {
            function: TestFnHandle::from_static_obj(f),
            name: Cow::Borrowed(name),
            extra,
        }
    }
}
