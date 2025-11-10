use std::{
    any::{Any, TypeId},
    borrow::Cow,
    collections::HashMap,
    time::Duration,
};

use crate::{Whatever, test::TestResult};

#[derive(Debug)]
#[non_exhaustive]
pub struct TestOutcome {
    pub status: TestStatus,
    pub duration: Duration,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
    pub attachments: TestOutcomeAttachments,
}

impl TestOutcome {
    pub fn is_good(&self) -> bool {
        self.status.is_good()
    }

    pub fn is_bad(&self) -> bool {
        self.status.is_bad()
    }
}

impl TestOutcome {
    pub fn passed(&self) -> bool {
        self.status.passed()
    }

    pub fn timed_out(&self) -> bool {
        self.status.timed_out()
    }

    pub fn ignored(&self) -> bool {
        self.status.ignored()
    }

    pub fn failed(&self) -> bool {
        self.status.failed()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum TestStatus {
    Passed,
    TimedOut,
    Ignored { reason: Option<Cow<'static, str>> },
    Failed(TestFailure),
    Other(Whatever),
}

impl TestStatus {
    pub fn is_good(&self) -> bool {
        matches!(
            self,
            TestStatus::Passed | TestStatus::Ignored { .. } | TestStatus::Other(_)
        )
    }

    pub fn is_bad(&self) -> bool {
        matches!(self, TestStatus::Failed(_) | TestStatus::TimedOut)
    }
}

impl TestStatus {
    pub fn passed(&self) -> bool {
        matches!(self, TestStatus::Passed)
    }

    pub fn timed_out(&self) -> bool {
        matches!(self, TestStatus::TimedOut)
    }

    pub fn ignored(&self) -> bool {
        matches!(self, TestStatus::Ignored { .. })
    }

    pub fn failed(&self) -> bool {
        matches!(self, TestStatus::Failed(_))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum TestFailure {
    Error(Whatever),
    Panicked(String),
    DidNotPanic {
        expected: Option<String>,
    },
    PanicMismatch {
        got: String,
        expected: Option<String>,
    },
}

impl From<TestResult> for TestStatus {
    fn from(value: TestResult) -> Self {
        match value.0 {
            Ok(_) => TestStatus::Passed,
            Err(err) => TestStatus::Failed(TestFailure::Error(Whatever::from(err))),
        }
    }
}

#[derive(Default, Debug)]
pub struct TestOutcomeAttachments(HashMap<TypeId, Box<dyn Any + Send + Sync + 'static>>);

impl TestOutcomeAttachments {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert<T: Send + Sync + 'static>(&mut self, v: T) {
        self.0.insert(TypeId::of::<T>(), Box::new(v));
    }

    pub fn get<T: Send + Sync + 'static>(&self) -> Option<&T> {
        self.0.get(&TypeId::of::<T>())?.downcast_ref()
    }

    pub fn get_mut<T: Send + Sync + 'static>(&mut self) -> Option<&mut T> {
        self.0.get_mut(&TypeId::of::<T>())?.downcast_mut()
    }

    pub fn take<T: Send + Sync + 'static>(&mut self) -> Option<T> {
        self.0
            .remove(&TypeId::of::<T>())?
            .downcast()
            .ok()
            .map(|b| *b)
    }
}
