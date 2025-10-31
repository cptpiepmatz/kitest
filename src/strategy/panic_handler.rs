use std::{
    any::Any,
    borrow::Cow,
    panic::{UnwindSafe, catch_unwind},
};

use crate::{
    outcome::{TestFailure, TestStatus},
    test::{TestMeta, TestResult},
};

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub enum PanicExpectation {
    #[default]
    ShouldNotPanic,
    ShouldPanic,
    ShouldPanicWithExpected(Cow<'static, str>),
}

impl From<bool> for PanicExpectation {
    fn from(value: bool) -> Self {
        match value {
            true => Self::ShouldPanic,
            false => Self::ShouldNotPanic,
        }
    }
}

impl From<&'static str> for PanicExpectation {
    fn from(value: &'static str) -> Self {
        Self::ShouldPanicWithExpected(value.into())
    }
}

impl From<String> for PanicExpectation {
    fn from(value: String) -> Self {
        Self::ShouldPanicWithExpected(value.into())
    }
}

pub trait TestPanicHandler<Extra> {
    fn handle<F: FnOnce() -> TestResult + UnwindSafe>(
        &self,
        f: F,
        meta: &TestMeta<Extra>,
    ) -> TestStatus;
}

#[derive(Debug, Default)]
pub struct NoPanicHandler;

impl<Extra> TestPanicHandler<Extra> for NoPanicHandler {
    fn handle<F: FnOnce() -> TestResult>(&self, f: F, _: &TestMeta<Extra>) -> TestStatus {
        f().into()
    }
}

#[derive(Debug, Default)]
pub struct DefaultPanicHandler;

impl DefaultPanicHandler {
    pub fn downcast_panic_err(err: Box<dyn Any + Send + 'static>) -> String {
        err.downcast::<&'static str>()
            .map(|s| s.to_string())
            .or_else(|err| err.downcast::<String>().map(|s| *s))
            .unwrap_or_else(|_| String::from("non-string panic payload"))
    }
}

impl<Extra> TestPanicHandler<Extra> for DefaultPanicHandler {
    fn handle<F: FnOnce() -> TestResult + UnwindSafe>(
        &self,
        f: F,
        meta: &TestMeta<Extra>,
    ) -> TestStatus {
        let result = catch_unwind(f);
        TestStatus::Failed(match (result, &meta.should_panic) {
            (Ok(result), PanicExpectation::ShouldNotPanic) => return result.into(),
            (Ok(_), PanicExpectation::ShouldPanic) => TestFailure::DidNotPanic { expected: None },
            (Ok(_), PanicExpectation::ShouldPanicWithExpected(expected)) => {
                TestFailure::DidNotPanic {
                    expected: Some(expected.to_string()),
                }
            }
            (Err(err), PanicExpectation::ShouldNotPanic) => {
                TestFailure::Panicked(Self::downcast_panic_err(err))
            }
            (Err(_), PanicExpectation::ShouldPanic) => return TestStatus::Passed,
            (Err(err), PanicExpectation::ShouldPanicWithExpected(expected)) => {
                let msg = Self::downcast_panic_err(err);
                match msg.contains(expected.as_ref()) {
                    true => return TestStatus::Passed,
                    false => TestFailure::PanicMismatch {
                        got: msg,
                        expected: Some(expected.to_string()),
                    },
                }
            }
        })
    }
}
