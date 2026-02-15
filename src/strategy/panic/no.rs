use crate::{
    outcome::TestStatus,
    panic::TestPanicHandler,
    test::{TestMeta, TestResult},
};

/// A [`TestPanicHandler`] that does not catch panics.
///
/// This handler simply executes the test function and converts its return value
/// into a [`TestStatus`]. Panics are not caught and will unwind normally.
///
/// - `Ok(())` results in [`TestStatus::Passed`]
/// - `Err(_)` results in [`TestStatus::Failed`]
///
/// Since no unwinding happens inside the handler, the test function does not need
/// to be [`UnwindSafe`](std::panic::UnwindSafe).
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct NoPanicHandler;

impl<Extra> TestPanicHandler<Extra> for NoPanicHandler {
    fn handle<F: FnOnce() -> TestResult>(&self, f: F, _: &TestMeta<Extra>) -> TestStatus {
        f().into()
    }
}
