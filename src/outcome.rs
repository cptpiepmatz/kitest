use std::{
    any::{Any, TypeId},
    borrow::Cow,
    collections::HashMap,
    time::Duration,
};

use crate::meta::TestResult;

pub struct TestOutcome {
    pub status: TestStatus,
    pub duration: Duration,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
    pub attachments: TestOutcomeAttachments,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TestStatus {
    Passed,
    TimedOut,
    Ignored { reason: Option<Cow<'static, str>> },
    Failed(TestFailure),
    Other,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TestFailure {
    Error(Box<str>),
    Panicked(Box<str>),
    DidNotPanic {
        expected: Option<Box<str>>,
    },
    PanicMismatch {
        got: Box<str>,
        expected: Option<Box<str>>,
    },
}

impl From<TestResult> for TestStatus {
    fn from(value: TestResult) -> Self {
        match value.0 {
            Ok(_) => TestStatus::Passed,
            Err(err) => TestStatus::Failed(TestFailure::Error(err)),
        }
    }
}

#[derive(Default)]
pub struct TestOutcomeAttachments(
    HashMap<TypeId, Box<dyn Any + Send + Sync + 'static>, ahash::RandomState>,
);

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
