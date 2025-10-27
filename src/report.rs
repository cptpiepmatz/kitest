use std::{collections::HashMap, time::Duration};

use crate::{formatter::FormatError, outcome::TestOutcome};

pub type TestOutcomes<'t> = HashMap<&'t str, TestOutcome, ahash::RandomState>;

#[derive(Debug)]
#[non_exhaustive]
pub struct TestReport<'t, FmtError: 't> {
    pub outcomes: TestOutcomes<'t>,
    pub duration: Duration,
    pub fmt_errors: Vec<(FormatError, FmtError)>,
}

pub type GroupedTestOutcomes<'t, GroupKey> =
    HashMap<GroupKey, HashMap<&'t str, TestOutcome, ahash::RandomState>, ahash::RandomState>;

#[derive(Debug)]
#[non_exhaustive]
pub struct GroupedTestReport<'t, GroupKey, FmtError: 't> {
    pub outcomes: GroupedTestOutcomes<'t, GroupKey>,
    pub duration: Duration,
    pub fmt_errors: Vec<(FormatError, FmtError)>,
}
