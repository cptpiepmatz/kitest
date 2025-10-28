use std::{
    any::Any,
    panic::{UnwindSafe, catch_unwind},
};

use crate::{
    outcome::{TestFailure, TestStatus},
    test::{TestMeta, TestResult},
};

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
        TestStatus::Failed(match (result, meta.should_panic.0) {
            (Ok(test_result), false) => return test_result.into(),
            (Ok(_), true) => TestFailure::DidNotPanic {
                expected: meta.should_panic.1.as_ref().map(|s| s.to_string()),
            },
            (Err(err), false) => TestFailure::Panicked(Self::downcast_panic_err(err)),
            (Err(err), true) => match &meta.should_panic.1 {
                None => return TestStatus::Passed,
                Some(expected) => {
                    let msg = Self::downcast_panic_err(err);
                    match msg.contains(expected.as_ref()) {
                        true => return TestStatus::Passed,
                        false => TestFailure::PanicMismatch {
                            got: msg,
                            expected: Some(expected.to_string()),
                        },
                    }
                }
            },
        })
    }
}
