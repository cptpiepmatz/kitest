//! Format transfer objects that are used by multiple formatters.

use std::{fmt::Display, marker::PhantomData};

use crate::{capture::OutputCapture, formatter::*, outcome::*};

#[derive(Debug, Clone, Copy)]
pub struct Tests<'t, Extra>(pub &'t [Test<Extra>]);

impl<'t, Extra> From<FmtRunInit<'t, Extra>> for Tests<'t, Extra> {
    fn from(value: FmtRunInit<'t, Extra>) -> Self {
        Tests(value.tests)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct TestCount(pub usize);

impl From<FmtRunStart> for TestCount {
    fn from(value: FmtRunStart) -> Self {
        TestCount(value.active)
    }
}

impl From<FmtEndListing> for TestCount {
    fn from(value: FmtEndListing) -> Self {
        TestCount(value.active + value.ignored)
    }
}

impl From<FmtGroupedRunStart> for TestCount {
    fn from(value: FmtGroupedRunStart) -> Self {
        TestCount(value.tests)
    }
}

/// A small newtype around a test name.
///
/// This is mainly used to make formatter implementations nicer to read, since it
/// can be constructed directly from [`FmtListTest`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct TestName<'t>(pub &'t str);

impl<'t, Extra> From<FmtListTest<'t, Extra>> for TestName<'t> {
    fn from(value: FmtListTest<'t, Extra>) -> Self {
        Self(value.meta.name.as_ref())
    }
}

#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct RunOutcomes<'t> {
    pub passed: usize,
    pub failed: usize,
    pub ignored: usize,
    pub filtered_out: usize,
    pub duration: Duration,
    pub failures: Vec<Failure<'t>>,
}

#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct Failure<'t> {
    pub group: Option<String>,
    pub name: &'t str,
    pub failure: TestFailure,
    pub output: OutputCapture,
}

impl<'t, 'o> From<FmtRunOutcomes<'t, 'o>> for RunOutcomes<'t> {
    fn from(value: FmtRunOutcomes<'t, 'o>) -> Self {
        Self {
            passed: value
                .outcomes
                .iter()
                .map(|(_, outcome)| outcome)
                .filter(|outcome| outcome.passed())
                .count(),
            failed: value
                .outcomes
                .iter()
                .map(|(_, outcome)| outcome)
                .filter(|outcome| outcome.failed())
                .count(),
            ignored: value
                .outcomes
                .iter()
                .map(|(_, outcome)| outcome)
                .filter(|outcome| outcome.ignored())
                .count(),
            filtered_out: value.filtered_out,
            duration: value.duration,
            failures: value
                .outcomes
                .iter()
                .filter_map(|(name, outcome)| {
                    let TestStatus::Failed(failure) = &outcome.status else {
                        return None;
                    };

                    Some(Failure {
                        group: None,
                        name,
                        failure: failure.clone(),
                        output: outcome.output.clone(),
                    })
                })
                .collect(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct GroupStart<L> {
    pub tests: usize,
    pub name: String,
    pub _label_marker: PhantomData<L>,
}

impl<'g, GroupKey, GroupCtx, L> From<FmtGroupStart<'g, GroupKey, GroupCtx>> for GroupStart<L>
where
    for<'b> L: From<&'b FmtGroupStart<'g, GroupKey, GroupCtx>> + Display,
{
    fn from(value: FmtGroupStart<'g, GroupKey, GroupCtx>) -> Self {
        let label = L::from(&value);
        let label = label.to_string();
        Self {
            name: label,
            tests: value.tests,
            _label_marker: PhantomData,
        }
    }
}

#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct GroupedRunOutcomes<'t, L> {
    pub groups: usize,
    pub passed: usize,
    pub failed: usize,
    pub ignored: usize,
    pub filtered_out: usize,
    pub duration: Duration,
    pub failures: Vec<Failure<'t>>,
    pub _label_marker: PhantomData<L>,
}

impl<'t, 'o, GroupKey, GroupCtx, L> From<FmtGroupedRunOutcomes<'t, 'o, GroupKey, GroupCtx>>
    for GroupedRunOutcomes<'t, L>
where
    for<'g> L: From<(&'g GroupKey, Option<&'g GroupCtx>)> + Display,
{
    fn from(value: FmtGroupedRunOutcomes<'t, 'o, GroupKey, GroupCtx>) -> Self {
        fn count_outcomes<GroupKey, GroupCtx, P>(
            value: &FmtGroupedRunOutcomes<'_, '_, GroupKey, GroupCtx>,
            predicate: P,
        ) -> usize
        where
            P: Fn(&TestOutcome) -> bool,
        {
            value
                .outcomes
                .iter()
                .map(|(_, outcomes, _)| {
                    outcomes
                        .iter()
                        .filter(|(_, outcome)| predicate(outcome))
                        .count()
                })
                .sum()
        }

        Self {
            _label_marker: PhantomData,
            groups: value.outcomes.len(),
            passed: count_outcomes(&value, |outcome| outcome.passed()),
            failed: count_outcomes(&value, |outcome| outcome.failed()),
            ignored: count_outcomes(&value, |outcome| outcome.ignored()),
            filtered_out: 0, // TODO: get proper value here
            duration: value.duration,
            failures: value
                .outcomes
                .iter()
                .flat_map(|(group_key, outcomes, group_ctx)| {
                    outcomes
                        .iter()
                        .map(move |(name, outcome)| (name, outcome, group_key, group_ctx))
                })
                .filter_map(|(name, outcome, group_key, group_ctx)| {
                    let TestStatus::Failed(failure) = &outcome.status else {
                        return None;
                    };

                    let group = L::from((group_key, group_ctx.as_ref())).to_string();
                    let group = (!group.is_empty()).then_some(group);

                    Some(Failure {
                        group,
                        name,
                        failure: failure.clone(),
                        output: outcome.output.clone(),
                    })
                })
                .collect(),
        }
    }
}

impl<'t, L> GroupedRunOutcomes<'t, L> {
    pub fn split(self) -> (usize, RunOutcomes<'t>) {
        (
            self.groups,
            RunOutcomes {
                passed: self.passed,
                failed: self.failed,
                ignored: self.ignored,
                filtered_out: self.filtered_out,
                duration: self.duration,
                failures: self.failures,
            },
        )
    }
}
