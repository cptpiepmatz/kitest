use crate::{
    outcome::TestStatus,
    panic_handler::TestPanicHandler,
    test::{TestMeta, TestResult},
};

#[derive(Debug, Default)]
pub struct NoPanicHandler;

impl<Extra> TestPanicHandler<Extra> for NoPanicHandler {
    fn handle<F: FnOnce() -> TestResult>(&self, f: F, _: &TestMeta<Extra>) -> TestStatus {
        f().into()
    }
}
