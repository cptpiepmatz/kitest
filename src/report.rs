use std::{
    process::{ExitCode, Termination},
    time::Duration,
};

use crate::{formatter::FormatError, outcome::TestOutcome};

pub type TestOutcomes<'t> = Vec<(&'t str, TestOutcome)>;

#[derive(Debug)]
#[non_exhaustive]
#[must_use = "ignoring this report may hide test failures or formatting errors"]
pub struct TestReport<'t, FmtError: 't> {
    pub outcomes: TestOutcomes<'t>,
    pub duration: Duration,
    pub fmt_errors: Vec<(FormatError, FmtError)>,
}

impl<'t, FmtError: 't> TestReport<'t, FmtError> {
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

pub type GroupedTestOutcomes<'t, GroupKey> = Vec<(GroupKey, Vec<(&'t str, TestOutcome)>)>;

#[derive(Debug)]
#[non_exhaustive]
#[must_use = "ignoring this report may hide test failures or formatting errors"]
pub struct GroupedTestReport<'t, GroupKey, FmtError: 't> {
    pub outcomes: GroupedTestOutcomes<'t, GroupKey>,
    pub duration: Duration,
    pub fmt_errors: Vec<(FormatError, FmtError)>,
}

impl<'t, GroupKey, FmtError: 't> GroupedTestReport<'t, GroupKey, FmtError> {
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
