use std::{
    any::Any,
    panic::{UnwindSafe, catch_unwind},
};

use crate::{
    outcome::{TestFailure, TestStatus},
    panic::{PanicExpectation, TestPanicHandler},
    test::{TestMeta, TestResult},
};

#[derive(Debug, Default)]
pub struct DefaultPanicHandler;

impl DefaultPanicHandler {
    pub fn payload_as_string(err: Box<dyn Any + Send + 'static>) -> String {
        err.downcast::<&'static str>()
            .map(|s| s.to_string())
            .or_else(|err| err.downcast::<String>().map(|s| *s))
            .unwrap_or_else(|_| String::from("Box<dyn Any>"))
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
                TestFailure::Panicked(Self::payload_as_string(err))
            }
            (Err(_), PanicExpectation::ShouldPanic) => return TestStatus::Passed,
            (Err(err), PanicExpectation::ShouldPanicWithExpected(expected)) => {
                let msg = Self::payload_as_string(err);
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{runner::SimpleRunner, test_support::*};

    #[test]
    fn handling_panics_works() {
        let tests = &[
            test! {
                name: "ok",
                should_panic: false,
                func: || ()
            },
            test! {
                name: "panic",
                should_panic: true,
                func: || if true { panic!() }
            },
            test! {
                name: "panic_with_expectation",
                should_panic: "expectation",
                func: || if true { panic!("expectation here") }
            },
            test! {
                name: "did_not_panic",
                should_panic: true,
                func: || if false { panic!() }
            },
            test! {
                name: "panic_mismatch",
                should_panic: "expectation",
                func: || if true { panic!("something else") }
            },
        ];

        let report = harness(tests)
            .with_panic_handler(DefaultPanicHandler)
            .with_runner(SimpleRunner::default())
            .run();
        let outcomes = report.outcomes;

        assert!(outcomes[0].1.passed());
        assert!(outcomes[1].1.passed());
        assert!(outcomes[2].1.passed());
        assert!(outcomes[3].1.failed());
        assert!(outcomes[4].1.failed());
    }
}
