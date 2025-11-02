use std::{borrow::Cow, panic::UnwindSafe};

use crate::{
    outcome::TestStatus,
    test::{TestMeta, TestResult},
};

mod no;
pub use no::*;

mod default;
pub use default::*;

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
