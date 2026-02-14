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

#[derive(Debug)]
pub struct RunOutcomes<'t> {
    pub passed: usize,
    pub failed: usize,
    pub ignored: usize,
    pub filtered_out: usize,
    pub duration: Duration,
    pub failures: Vec<Failure<'t>>,
}

#[derive(Debug)]
pub struct Failure<'t> {
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
                        name,
                        failure: failure.clone(),
                        output: outcome.output.clone(),
                    })
                })
                .collect(),
        }
    }
}
