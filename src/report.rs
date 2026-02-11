//! Test reports.
//!
//! This module defines report types returned by test harness runs.
//!
//! A report is the final product of running a harness.
//! It contains all test outcomes, the total duration of the run, and any formatting errors that
//! occurred while printing results.
//!
//! Reports implement [`Termination`], so they can be returned directly from `main` in a custom
//! test binary.
//! The exit code is derived from test failures and formatter errors.

use std::{
    process::{ExitCode, Termination},
    time::Duration,
};

use crate::{formatter::FormatError, outcome::TestOutcome};

/// Collected outcomes of a test run.
///
/// [`TestOutcomes`] is a list of `(test_name, outcome)` pairs produced by a
/// [`TestHarness`](super::TestHarness) run.
///
/// The test name is a borrowed string tied to the lifetime of the original
/// test list, and the [`TestOutcome`] contains the full result of executing
/// that test.
pub type TestOutcomes<'t> = Vec<(&'t str, TestOutcome)>;

/// The report produced by running a [`TestHarness`](super::TestHarness).
///
/// [`TestReport`] is returned by [`TestHarness::run`](super::TestHarness::run).
/// It contains all outcomes of the test run, the total time spent executing the harness, and any
/// errors reported by the formatter.
///
/// The recorded duration only covers the time spent inside the harness itself.
/// Any work done before calling `run` (for example test discovery or data loading)
/// is not included and must be tracked separately if needed.
///
/// Formatter errors are collected instead of aborting the run early. This allows
/// test execution to complete even if formatting fails partway through.
#[derive(Debug)]
#[non_exhaustive]
#[must_use = "ignoring this report may hide test failures or formatting errors"]
pub struct TestReport<'t, FmtError: 't> {
    /// Outcomes of all executed tests.
    pub outcomes: TestOutcomes<'t>,

    /// Total duration of the test run.
    pub duration: Duration,

    /// Errors reported by the formatter.
    ///
    /// Each entry contains the formatting stage and the formatter specific error.
    pub fmt_errors: Vec<(FormatError, FmtError)>,
}

impl<'t, FmtError: 't> TestReport<'t, FmtError> {
    /// Compute the process exit code for this test report.
    ///
    /// The exit code is determined as follows:
    ///
    /// - If any test failed, the exit code is [`ExitCode::FAILURE`]
    /// - Otherwise, if any formatter errors occurred, the exit code is
    ///   [`ExitCode::FAILURE`]
    /// - Otherwise, the exit code is [`ExitCode::SUCCESS`]
    ///
    /// This mirrors the behavior of the built in Rust test harness, where
    /// formatting errors are treated as fatal.
    pub fn exit_code(&self) -> ExitCode {
        let any_failed = self.outcomes.iter().any(|(_, outcome)| outcome.failed());
        if any_failed {
            return ExitCode::FAILURE;
        }

        match self.fmt_errors.is_empty() {
            true => ExitCode::SUCCESS,
            false => ExitCode::FAILURE,
        }
    }
}

impl<'t, FmtError: 't> Termination for TestReport<'t, FmtError> {
    fn report(self) -> ExitCode {
        self.exit_code()
    }
}

/// Collected outcomes of a grouped test run.
///
/// [`GroupedTestOutcomes`] is a list of `(group_key, test_outcomes)` pairs produced by a
/// [`GroupedTestHarness`](super::GroupedTestHarness) run.
///
/// Each entry represents one group, identified by its `GroupKey`, and contains the
/// outcomes of all tests executed in that group.
///
/// This mirrors [`TestOutcomes`], but adds one level of structure for grouping.
pub type GroupedTestOutcomes<'t, GroupKey> = Vec<(GroupKey, Vec<(&'t str, TestOutcome)>)>;

/// The report produced by running a [`GroupedTestHarness`](super::GroupedTestHarness).
///
/// [`GroupedTestReport`] is returned by [`GroupedTestHarness::run`](super::GroupedTestHarness::run).
/// It contains the outcomes of all executed test groups, the total duration of the run, and any
/// errors reported by the formatter.
///
/// Like [`TestReport`], the recorded duration only covers the time spent inside the harness itself.
/// Any work done before calling `run` (such as test discovery or data loading) is not included.
///
/// Formatter errors are collected instead of aborting the run early, so grouped
/// execution can finish even if formatting fails partway through.
#[derive(Debug)]
#[non_exhaustive]
#[must_use = "ignoring this report may hide test failures or formatting errors"]
pub struct GroupedTestReport<'t, GroupKey, FmtError: 't> {
    /// Outcomes of all executed test groups.
    ///
    /// Each entry contains the group key and the outcomes of the tests in that group.
    pub outcomes: GroupedTestOutcomes<'t, GroupKey>,

    /// Total duration of the grouped test run.
    pub duration: Duration,

    /// Errors reported by the formatter.
    ///
    /// Each entry contains the formatting stage and the formatter specific error.
    pub fmt_errors: Vec<(FormatError, FmtError)>,
}

impl<'t, GroupKey, FmtError: 't> GroupedTestReport<'t, GroupKey, FmtError> {
    /// Compute the process exit code for this grouped test report.
    ///
    /// The exit code is determined as follows:
    ///
    /// - If any test in any group failed, the exit code is [`ExitCode::FAILURE`]
    /// - Otherwise, if any formatter errors occurred, the exit code is
    ///   [`ExitCode::FAILURE`]
    /// - Otherwise, the exit code is [`ExitCode::SUCCESS`]
    ///
    /// This mirrors the behavior of the built in Rust test harness and matches
    /// the non grouped [`TestReport`] behavior.
    pub fn exit_code(&self) -> ExitCode {
        let any_failed = self
            .outcomes
            .iter()
            .any(|(_, outcomes)| outcomes.iter().any(|(_, outcome)| outcome.failed()));
        if any_failed {
            return ExitCode::FAILURE;
        }

        match self.fmt_errors.is_empty() {
            true => ExitCode::SUCCESS,
            false => ExitCode::FAILURE,
        }
    }
}

impl<'t, GroupKey, FmtError: 't> Termination for GroupedTestReport<'t, GroupKey, FmtError> {
    fn report(self) -> ExitCode {
        self.exit_code()
    }
}
