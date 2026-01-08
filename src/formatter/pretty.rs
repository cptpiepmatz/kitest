use std::{collections::HashMap, fmt::Display, io, marker::PhantomData, time::Duration};

use crate::{
    capture::OutputCapture,
    formatter::{
        common::{
            TestName,
            color::{ColorSetting, SupportsColor, colors::*},
            label::{FromGroupCtx, FromGroupKey, GroupLabel},
        },
        *,
    },
    outcome::{TestFailure, TestStatus},
    panic::PanicExpectation,
};

#[derive(Debug)]
pub struct PrettyFormatter<'t, W: io::Write + SupportsColor, L, Extra> {
    target: W,
    color_settings: ColorSetting,
    _label_marker: PhantomData<L>,
    tests: HashMap<&'t str, &'t Test<Extra>>,
}

impl<'t, W: io::Write + SupportsColor, L, Extra> PrettyFormatter<'t, W, L, Extra> {
    pub fn new() -> PrettyFormatter<'t, io::Stdout, GroupLabel<FromGroupKey>, Extra> {
        PrettyFormatter::default()
    }

    pub fn with_target<WithTarget: io::Write + SupportsColor>(
        self,
        target: WithTarget,
    ) -> PrettyFormatter<'t, WithTarget, L, Extra> {
        PrettyFormatter {
            target,
            color_settings: self.color_settings,
            _label_marker: PhantomData,
            tests: Default::default(),
        }
    }

    pub fn with_color_settings(self, color_settings: ColorSetting) -> Self {
        Self {
            color_settings,
            ..self
        }
    }

    pub fn with_group_label_from_key(
        self,
    ) -> PrettyFormatter<'t, W, GroupLabel<FromGroupKey>, Extra> {
        PrettyFormatter {
            target: self.target,
            color_settings: self.color_settings,
            _label_marker: PhantomData,
            tests: Default::default(),
        }
    }

    pub fn with_group_label_from_ctx(
        self,
    ) -> PrettyFormatter<'t, W, GroupLabel<FromGroupCtx>, Extra> {
        PrettyFormatter {
            target: self.target,
            color_settings: self.color_settings,
            _label_marker: PhantomData,
            tests: Default::default(),
        }
    }
}

impl<'t, Extra> Default for PrettyFormatter<'t, io::Stdout, GroupLabel<FromGroupKey>, Extra> {
    fn default() -> Self {
        Self {
            target: io::stdout(),
            color_settings: Default::default(),
            _label_marker: PhantomData,
            tests: Default::default(),
        }
    }
}

impl<'t, W: io::Write + SupportsColor, L, Extra> PrettyFormatter<'t, W, L, Extra> {
    pub fn use_color(&self) -> bool {
        match self.color_settings {
            ColorSetting::Automatic => self.target.supports_color(),
            ColorSetting::Always => true,
            ColorSetting::Never => false,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PrettyRunInit<'t, Extra> {
    pub tests: &'t [Test<Extra>],
}

impl<'t, Extra> From<FmtRunInit<'t, Extra>> for PrettyRunInit<'t, Extra> {
    fn from(value: FmtRunInit<'t, Extra>) -> Self {
        Self { tests: value.tests }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct PrettyTestCount(usize);

impl From<FmtRunStart> for PrettyTestCount {
    fn from(value: FmtRunStart) -> Self {
        PrettyTestCount(value.active)
    }
}

impl From<FmtEndListing> for PrettyTestCount {
    fn from(value: FmtEndListing) -> Self {
        PrettyTestCount(value.active + value.ignored)
    }
}

impl From<FmtGroupedRunStart> for PrettyTestCount {
    fn from(value: FmtGroupedRunStart) -> Self {
        PrettyTestCount(value.tests)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct PrettyTestOutcome<'t> {
    pub name: &'t str,
    pub status: TestStatus,
    pub should_panic: PanicExpectation,
}

impl<'t, 'o, Extra> From<FmtTestOutcome<'t, 'o, Extra>> for PrettyTestOutcome<'t> {
    fn from(value: FmtTestOutcome<'t, 'o, Extra>) -> Self {
        Self {
            name: value.meta.name.as_ref(),
            status: value.outcome.status.clone(),
            should_panic: value.meta.should_panic.clone(),
        }
    }
}

#[derive(Debug)]
pub struct PrettyRunOutcomes<'t> {
    pub passed: usize,
    pub failed: usize,
    pub ignored: usize,
    pub filtered_out: usize,
    pub duration: Duration,
    pub failures: Vec<PrettyFailure<'t>>,
}

#[derive(Debug)]
pub struct PrettyFailure<'t> {
    pub name: &'t str,
    pub failure: TestFailure,
    pub output: OutputCapture,
}

impl<'t, 'o> From<FmtRunOutcomes<'t, 'o>> for PrettyRunOutcomes<'t> {
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

                    Some(PrettyFailure {
                        name,
                        failure: failure.clone(),
                        output: outcome.output.clone(),
                    })
                })
                .collect(),
        }
    }
}

impl<'t, Extra: 't + Sync, W: io::Write + SupportsColor + Send, L: Send> TestFormatter<'t, Extra>
    for PrettyFormatter<'t, W, L, Extra>
{
    type Error = io::Error;

    type RunInit = PrettyRunInit<'t, Extra>;
    fn fmt_run_init(&mut self, data: Self::RunInit) -> Result<(), Self::Error> {
        self.tests = HashMap::from_iter(data.tests.iter().map(|test| (test.name.as_ref(), test)));
        Ok(())
    }

    type RunStart = PrettyTestCount;
    fn fmt_run_start(&mut self, data: Self::RunStart) -> Result<(), Self::Error> {
        match data.0 {
            1 => writeln!(self.target, "\nrunning 1 test"),
            count => writeln!(self.target, "\nrunning {count} tests"),
        }
    }

    type TestOutcome = PrettyTestOutcome<'t>;
    fn fmt_test_outcome(&mut self, data: Self::TestOutcome) -> Result<(), Self::Error> {
        write!(self.target, "test {}", data.name)?;
        if let PanicExpectation::ShouldPanic = data.should_panic {
            write!(self.target, " - should panic")?;
        }
        write!(self.target, " ... ")?;
        match (data.status, self.use_color()) {
            (TestStatus::Passed, true) => write!(self.target, "{GREEN}ok{RESET}")?,
            (TestStatus::Passed, false) => write!(self.target, "ok")?,
            (
                TestStatus::Ignored {
                    reason: Some(reason),
                },
                true,
            ) => write!(self.target, "{YELLOW}ignored, {reason}{RESET}")?,
            (
                TestStatus::Ignored {
                    reason: Some(reason),
                },
                false,
            ) => write!(self.target, "ignored, {reason}")?,
            (TestStatus::Ignored { reason: None }, true) => {
                write!(self.target, "{YELLOW}ignored{RESET}")?
            }
            (TestStatus::Ignored { reason: None }, false) => write!(self.target, "ignored")?,
            (TestStatus::TimedOut, true) => write!(self.target, "{RED}timed out{RESET}")?,
            (TestStatus::TimedOut, false) => write!(self.target, "timed out")?,
            (TestStatus::Failed(_test_failure), true) => write!(self.target, "{RED}FAILED{RESET}")?,
            (TestStatus::Failed(_test_failure), false) => write!(self.target, "FAILED")?,
            (TestStatus::Other(_), true) => write!(self.target, "{CYAN}other{RESET}")?,
            (TestStatus::Other(_), false) => write!(self.target, "other")?,
        };
        writeln!(self.target)
    }

    type RunOutcomes = PrettyRunOutcomes<'t>;
    fn fmt_run_outcomes(
        &mut self,
        PrettyRunOutcomes {
            passed,
            failed,
            ignored,
            filtered_out,
            duration,
            failures,
        }: Self::RunOutcomes,
    ) -> Result<(), Self::Error> {
        if !failures.is_empty() {
            writeln!(self.target)?;
            writeln!(self.target, "failures:")?;
            writeln!(self.target)?;
            for failure in failures.iter() {
                writeln!(self.target, "---- {} stdout ----", failure.name)?;
                match &failure.failure {
                    TestFailure::Error(err) => writeln!(self.target, "Error: {err}")?,
                    TestFailure::Panicked(_) => self.target.write_all(failure.output.raw())?,
                    TestFailure::DidNotPanic { expected: None } => {
                        if let Some(meta) = self.tests.get(failure.name)
                            && let Some(origin) = &meta.origin
                        {
                            write!(
                                self.target,
                                "note: test did not panic as expected at {origin}"
                            )?;
                        }
                    }
                    _ => todo!(),
                }
                writeln!(self.target)?;
            }
            writeln!(self.target)?;
            writeln!(self.target, "failures:")?;
            for failure in failures.iter() {
                writeln!(self.target, "    {}", failure.name)?;
            }
        }

        writeln!(self.target)?;
        write!(self.target, "test result: ")?;
        match failed {
            0 => write!(self.target, "ok. ")?,
            _ => write!(self.target, "FAILED. ")?,
        }
        writeln!(
            self.target,
            "{passed} passed; {failed} failed; {ignored} ignored; 0 measured; {filtered_out} filtered out; finished in {:.2}s",
            duration.as_secs_f64()
        )?;
        writeln!(self.target)
    }

    type TestIgnored = ();
    type TestStart = ();
}

#[derive(Debug, PartialEq, Eq)]
pub struct PrettyGroupStart<L> {
    pub tests: usize,
    pub name: String,
    pub _label_marker: PhantomData<L>,
}

impl<'g, GroupKey, GroupCtx, L> From<FmtGroupStart<'g, GroupKey, GroupCtx>> for PrettyGroupStart<L>
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

#[derive(Debug, PartialEq, Eq)]
pub struct PrettyGroupedRunOutcomes {
    pub groups: usize,
    pub passed: usize,
    pub failed: usize,
    pub ignored: usize,
    pub filtered_out: usize,
    pub duration: Duration,
}

impl<'t, 'o, GroupKey> From<FmtGroupedRunOutcomes<'t, 'o, GroupKey>> for PrettyGroupedRunOutcomes {
    fn from(value: FmtGroupedRunOutcomes<'t, 'o, GroupKey>) -> Self {
        fn count_outcomes<GroupKey, P>(
            value: &FmtGroupedRunOutcomes<'_, '_, GroupKey>,
            predicate: P,
        ) -> usize
        where
            P: Fn(&TestOutcome) -> bool,
        {
            value
                .outcomes
                .iter()
                .map(|(_, outcomes)| {
                    outcomes
                        .iter()
                        .filter(|(_, outcome)| predicate(outcome))
                        .count()
                })
                .sum()
        }

        Self {
            groups: value.outcomes.len(),
            passed: count_outcomes(&value, TestOutcome::passed),
            failed: count_outcomes(&value, TestOutcome::failed),
            ignored: count_outcomes(&value, TestOutcome::ignored),
            filtered_out: 0, // TODO: get proper value here
            duration: value.duration,
        }
    }
}

impl<'t, Extra: 't + Sync, GroupKey, GroupCtx, W, L>
    GroupedTestFormatter<'t, Extra, GroupKey, GroupCtx> for PrettyFormatter<'t, W, L, Extra>
where
    Extra: 't,
    GroupKey: 't,
    GroupCtx: 't,
    W: io::Write + SupportsColor + Send,
    L: Send + Display,
    for<'b, 'g> L: From<&'b FmtGroupStart<'g, GroupKey, GroupCtx>>,
{
    type GroupedRunStart = PrettyTestCount;
    fn fmt_grouped_run_start(&mut self, data: Self::GroupedRunStart) -> Result<(), Self::Error> {
        <PrettyFormatter<'_, _, _, _> as TestFormatter<'_, Extra>>::fmt_run_start(self, data)
    }

    type GroupStart = PrettyGroupStart<L>;
    fn fmt_group_start(&mut self, data: Self::GroupStart) -> Result<(), Self::Error> {
        writeln!(self.target)?;
        let group_name = match data.name.is_empty() {
            true => "default",
            false => data.name.as_str(),
        };
        writeln!(
            self.target,
            "group {group_name}, running {} tests",
            data.tests
        )
    }

    type GroupedRunOutcomes = PrettyGroupedRunOutcomes;
    fn fmt_grouped_run_outcomes(
        &mut self,
        PrettyGroupedRunOutcomes {
            groups,
            passed,
            failed,
            ignored,
            filtered_out,
            duration,
        }: Self::GroupedRunOutcomes,
    ) -> Result<(), Self::Error> {
        writeln!(self.target)?;
        write!(self.target, "test result: ")?;
        match failed {
            0 => write!(self.target, "ok. ")?,
            _ => write!(self.target, "FAILED. ")?,
        }
        writeln!(
            self.target,
            "{passed} passed; {failed} failed; {ignored} ignored; {filtered_out} filtered out; across {groups} groups, finished in {:.2}s",
            duration.as_secs_f64()
        )?;
        writeln!(self.target)
    }

    type GroupOutcomes = ();
}

impl<'t, Extra: 't, W: io::Write + io::IsTerminal, L> TestListFormatter<'t, Extra>
    for PrettyFormatter<'t, W, L, Extra>
{
    type Error = io::Error;

    type ListTest = TestName<'t>;
    fn fmt_list_test(&mut self, data: Self::ListTest) -> Result<(), Self::Error> {
        writeln!(self.target, "{}: test", data.0)
    }

    type EndListing = PrettyTestCount;
    fn fmt_end_listing(&mut self, data: Self::EndListing) -> Result<(), Self::Error> {
        writeln!(self.target, "\n{} tests", data.0)
    }

    type InitListing = ();
    type BeginListing = ();
}
