use std::time::Duration;

use crate::{formatter::FormatError, outcome::TestOutcome};

pub type TestOutcomes<'t> = Vec<(&'t str, TestOutcome)>;

#[derive(Debug)]
#[non_exhaustive]
pub struct TestReport<'t, FmtError: 't> {
    pub outcomes: TestOutcomes<'t>,
    pub duration: Duration,
    pub fmt_errors: Vec<(FormatError, FmtError)>,
}

pub type GroupedTestOutcomes<'t, GroupKey> = Vec<(GroupKey, Vec<(&'t str, TestOutcome)>)>;

#[derive(Debug)]
#[non_exhaustive]
pub struct GroupedTestReport<'t, GroupKey, FmtError: 't> {
    pub outcomes: GroupedTestOutcomes<'t, GroupKey>,
    pub duration: Duration,
    pub fmt_errors: Vec<(FormatError, FmtError)>,
}
