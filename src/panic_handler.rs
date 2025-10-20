use std::{any::Any, panic::{catch_unwind, RefUnwindSafe}};

use crate::meta::{TestMeta, TestResult};

pub trait TestPanicHandler<Extra> {
    fn handle(&self, meta: &TestMeta<Extra>) -> TestResult;
}

pub struct NoPanicHandler;

impl<Extra> TestPanicHandler<Extra> for NoPanicHandler {
    fn handle(&self, meta: &TestMeta<Extra>) -> TestResult {
        meta.function.call()
    }
}

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
    fn handle(&self, meta: &TestMeta<Extra>) -> TestResult {
        let result = catch_unwind(|| meta.function.call());
        match (result, meta.should_panic.0) {
            (Ok(test_result), false) => test_result,
            (Ok(_), true) => TestResult(Err("test did not panic as expected".into())),
            (Err(err), false) => TestResult(Err(Self::downcast_panic_err(err))),
            (Err(err), true) => match &meta.should_panic.1 {
                None => ().into(),
                Some(expected) => {
                    let msg = Self::downcast_panic_err(err);
                    match msg.contains(expected.as_ref()) {
                        true => ().into(),
                        false => TestResult(Err(
                            "panic message did not contain expected substring".into()
                        ))
                    }
                }
            },
        }
    }
}

impl<F, Extra> TestPanicHandler<Extra> for F where F: Fn(&TestMeta<Extra>) -> TestResult {
    fn handle(&self, meta: &TestMeta<Extra>) -> TestResult {
        self(meta)
    }
}
