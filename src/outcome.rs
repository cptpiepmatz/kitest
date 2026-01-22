//! Test outcomes.
//!
//! This module defines the types that represent the result of running a test.
//!
//! The central type is [`TestOutcome`]. It combines a [`TestStatus`] with timing
//! information and captured output.
//!
//! Outcomes are produced by running a harness and are used by reports and
//! formatters to present results. They are also a useful extension point:
//!
//! - [`TestStatus::Other`] allows custom statuses when the built in variants are not enough
//! - [`TestOutcomeAttachments`] allows attaching additional typed data to an outcome
//!
//! The types in this module are marked `#[non_exhaustive]` where appropriate, so we can
//! extend them over time without breaking downstream code.

use std::{
    any::{Any, TypeId},
    borrow::Cow,
    collections::HashMap,
    ops::Deref,
    time::Duration,
};

use crate::{Whatever, capture::OutputCapture, test::TestResult};

/// The outcome of a single test execution.
///
/// A [`TestOutcome`] represents everything produced while running one test.
/// A harness [`TestRunner`](super::runner::TestRunner) is expected to produce outcomes paired with
/// the test name, so the harness can collect and report them.
///
/// [`TestOutcome`] is intentionally a bundle of the common things that are useful for
/// reporting and formatting.
#[derive(Debug)]
pub struct TestOutcome {
    /// The status of the test execution.
    ///
    /// This is the primary signal used to decide whether a test was successful or not.
    pub status: TestStatus,

    /// How long the test took to execute.
    pub duration: Duration,

    /// Captured output produced during the test run.
    ///
    /// This includes anything routed through Kitest's output capture, such as `println`
    /// style output and panic output, depending on the configured capture setup.
    pub output: OutputCapture,

    /// Additional type erased data attached to this outcome.
    ///
    /// This is a bag for any extra information that may be useful for reporting or
    /// custom formatting, without forcing more generics through the core types.
    pub attachments: TestOutcomeAttachments,
}

impl Deref for TestOutcome {
    type Target = TestStatus;

    fn deref(&self) -> &Self::Target {
        &self.status
    }
}

/// The status of a test execution.
///
/// [`TestStatus`] describes the result of running a test.
/// It is primarily produced by a [`TestPanicHandler`](super::panic::TestPanicHandler), which
/// decides whether a test passed or failed based on its panic behavior and metadata.
///
/// A [`TestRunner`](super::runner::TestRunner) may further process or wrap this status when
/// producing the final [`TestOutcome`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TestStatus {
    /// The test passed successfully.
    Passed,

    /// The test exceeded its allowed execution time.
    ///
    /// This variant is currently not produced by the built in runners, but is reserved
    /// for runners that implement timeouts.
    TimedOut,

    /// The test was ignored.
    ///
    /// A reason may be provided.
    /// This does not have to originate from the test metadata and may be decided dynamically by
    /// the [`TestIgnore`](super::ignore::TestIgnore) strategy.
    Ignored {
        /// Optional reason why the test was ignored.
        reason: Option<Cow<'static, str>>,
    },

    /// The test failed.
    ///
    /// This variant carries more detailed failure information.
    Failed(TestFailure),

    /// A custom test status.
    ///
    /// This is intended for cases where the built in status variants are not expressive enough.
    /// It is not necessarily a failure.
    ///
    /// When deciding whether a test is considered "good" or "bad", this variant is
    /// treated as a successful outcome.
    Other(Whatever),
}

impl TestStatus {
    /// Returns `true` if this status is considered successful.
    ///
    /// The following statuses are treated as good:
    /// - [`Passed`](TestStatus::Passed)
    /// - [`Ignored`](TestStatus::Ignored)
    /// - [`Other`](TestStatus::Other)
    pub fn is_good(&self) -> bool {
        matches!(
            self,
            TestStatus::Passed | TestStatus::Ignored { .. } | TestStatus::Other(_)
        )
    }

    /// Returns `true` if this status is considered a failure.
    ///
    /// The following statuses are treated as bad:
    /// - [`Failed`](TestStatus::Failed)
    /// - [`TimedOut`](TestStatus::TimedOut)
    pub fn is_bad(&self) -> bool {
        matches!(self, TestStatus::Failed(_) | TestStatus::TimedOut)
    }
}

impl TestStatus {
    /// Returns `true` if the test passed.
    pub fn passed(&self) -> bool {
        matches!(self, TestStatus::Passed)
    }

    /// Returns `true` if the test timed out.
    pub fn timed_out(&self) -> bool {
        matches!(self, TestStatus::TimedOut)
    }

    /// Returns `true` if the test was ignored.
    pub fn ignored(&self) -> bool {
        matches!(self, TestStatus::Ignored { .. })
    }

    /// Returns `true` if the test failed.
    pub fn failed(&self) -> bool {
        matches!(self, TestStatus::Failed(_))
    }
}

/// Describes why a test failed.
///
/// `TestFailure` provides more detailed information for a failed test outcome.
/// It is carried by [`TestStatus::Failed`] and is typically produced by a
/// [`TestPanicHandler`](super::panic::TestPanicHandler).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TestFailure {
    /// The test failed with an error.
    ///
    /// This is used for failures that are not directly caused by panics.
    /// The contained value stores additional error information in a type erased
    /// form.
    Error(Whatever),

    /// The test panicked when it was not expected to.
    ///
    /// The contained string represents the panic payload formatted as text.
    Panicked(String),

    /// The test was expected to panic, but no panic occurred.
    ///
    /// The optional string describes the expected panic message, if one was
    /// specified.
    DidNotPanic {
        /// The expected panic message, if any.
        expected: Option<String>,
    },

    /// The test panicked, but the panic did not match the expectation.
    ///
    /// This is used when a panic occurred, but the panic message did not match
    /// the expected value.
    PanicMismatch {
        /// The panic message that was observed.
        got: String,
        /// The expected panic message, if any.
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

/// Additional typed data attached to a [`TestOutcome`].
///
/// [`TestOutcomeAttachments`] is a bag for values that come up during test execution but do not
/// fit into the fixed [`TestOutcome`] fields.
///
/// Multiple attachments can be stored at once.
/// Values are keyed by their [`TypeId`], so there can be at most one stored value per concrete
/// type.
///
/// This intentionally does not use [`Whatever`].
/// Attachments are expected to be specific to a particular runner or setup, and typically only
/// make sense together with a formatter that knows the involved types and can downcast them.
///
/// An empty attachments map does not allocate, so keeping this field around is still cheap for
/// most use cases.
#[derive(Default, Debug)]
pub struct TestOutcomeAttachments(HashMap<TypeId, Box<dyn Any + Send + Sync + 'static>>);

impl TestOutcomeAttachments {
    /// Create an empty attachments container.
    pub fn new() -> Self {
        Self::default()
    }

    /// Insert an attachment value.
    ///
    /// The value is stored under its concrete type.
    /// If another value of the same type already exists, it is replaced.
    ///
    /// Only one value per type can be stored at a time.
    pub fn insert<T: Send + Sync + 'static>(&mut self, v: T) {
        self.0.insert(TypeId::of::<T>(), Box::new(v));
    }

    /// Get a shared reference to an attachment of type `T`.
    ///
    /// Returns [`None`] if no value of this type is attached or if the stored value
    /// has a different type.
    pub fn get<T: Send + Sync + 'static>(&self) -> Option<&T> {
        self.0.get(&TypeId::of::<T>())?.downcast_ref()
    }

    /// Get a mutable reference to an attachment of type `T`.
    ///
    /// Returns [`None`] if no value of this type is attached or if the stored value
    /// has a different type.
    pub fn get_mut<T: Send + Sync + 'static>(&mut self) -> Option<&mut T> {
        self.0.get_mut(&TypeId::of::<T>())?.downcast_mut()
    }

    /// Remove and return an attachment of type `T`.
    ///
    /// Returns `None` if no value of this type is attached or if the stored value
    /// has a different type.
    pub fn take<T: Send + Sync + 'static>(&mut self) -> Option<T> {
        self.0
            .remove(&TypeId::of::<T>())?
            .downcast()
            .ok()
            .map(|b| *b)
    }
}
