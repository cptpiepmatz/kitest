use std::{
    any::Any,
    panic::{RefUnwindSafe, catch_unwind},
};

use crate::{
    meta::TestMeta,
    outcome::{TestFailure, TestStatus},
};

pub trait TestPanicHandler<Extra> {
    fn handle(&self, meta: &TestMeta<Extra>) -> TestStatus;
}

pub struct NoPanicHandler;

impl<Extra> TestPanicHandler<Extra> for NoPanicHandler {
    fn handle(&self, meta: &TestMeta<Extra>) -> TestStatus {
        meta.function.call().into()
    }
}

#[derive(Default)]
pub struct DefaultPanicHandler;

impl DefaultPanicHandler {
    pub fn downcast_panic_err(err: Box<dyn Any + Send + 'static>) -> Box<str> {
        err.downcast::<&'static str>()
            .map(|s| s.to_string().into_boxed_str())
            .or_else(|err| err.downcast::<String>().map(|s| s.into_boxed_str()))
            .unwrap_or_else(|_| String::from("non-string panic payload").into_boxed_str())
    }
}

impl<Extra: RefUnwindSafe> TestPanicHandler<Extra> for DefaultPanicHandler {
    fn handle(&self, meta: &TestMeta<Extra>) -> TestStatus {
        let result = catch_unwind(|| meta.function.call());
        TestStatus::Failed(match (result, meta.should_panic.0) {
            (Ok(test_result), false) => return test_result.into(),
            (Ok(_), true) => TestFailure::DidNotPanic {
                expected: meta
                    .should_panic
                    .1
                    .as_ref()
                    .map(|s| s.to_string().into_boxed_str()),
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
                            expected: Some(expected.to_string().into_boxed_str()),
                        },
                    }
                }
            },
        })
    }
}

impl<F, Extra> TestPanicHandler<Extra> for F
where
    F: Fn(&TestMeta<Extra>) -> TestStatus,
{
    fn handle(&self, meta: &TestMeta<Extra>) -> TestStatus {
        self(meta)
    }
}
