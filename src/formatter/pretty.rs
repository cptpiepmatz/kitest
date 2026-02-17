use std::{fmt::Display, io, marker::PhantomData, time::Duration};

use crate::{
    capture::OutputCapture,
    formatter::{
        common::{
            CommonFormatter,
            color::{ColorSetting, SupportsColor, colors::*},
            fto,
            fto::TestName,
            label::{FromGroupCtx, FromGroupKey, GroupLabel},
        },
        *,
    },
    outcome::{TestFailure, TestStatus},
    panic::PanicExpectation,
};

/// A human friendly formatter that behaves similar to the built in Rust test harness.
///
/// It prints per test status lines and a final summary. On failures it prints
/// additional details and captured output.
///
/// The formatter writes to a target `W`, which makes it possible to format into
/// something other than the console (for example a log file or an in memory buffer).
///
/// Coloring is controlled via [`ColorSetting`].
/// In automatic mode, the formatter uses the target's [`SupportsColor`] implementation to decide
/// if color should be used.
#[derive(Debug, Clone)]
pub struct PrettyFormatter<'t, W: io::Write, L, Extra> {
    common: CommonFormatter<'t, W, L, Extra>,
}

impl<'t, W: io::Write, L, Extra> PrettyFormatter<'t, W, L, Extra> {
    /// Create a `PrettyFormatter` that writes to stdout.
    ///
    /// By default, group labels are derived from the group key via [`GroupLabel`].
    pub fn new() -> PrettyFormatter<'t, io::Stdout, GroupLabel<FromGroupKey>, Extra> {
        PrettyFormatter::default()
    }

    /// Replace the output target.
    ///
    /// This can be used to write into a file, a buffer, or any other writer.
    pub fn with_target<WithTarget: io::Write>(
        self,
        target: WithTarget,
    ) -> PrettyFormatter<'t, WithTarget, L, Extra> {
        PrettyFormatter {
            common: CommonFormatter {
                target,
                color_setting: self.common.color_setting,
                tests: self.common.tests,
                _label_marker: PhantomData,
            },
        }
    }

    /// Replace the color settings.
    pub fn with_color_setting(self, color_setting: impl Into<ColorSetting>) -> Self {
        Self {
            common: CommonFormatter {
                color_setting: color_setting.into(),
                ..self.common
            },
        }
    }

    /// Choose group labels based on the group key.
    ///
    /// This affects only grouped output and uses [`GroupLabel`] with
    /// [`FromGroupKey`] to derive the display name.
    pub fn with_group_label_from_key(
        self,
    ) -> PrettyFormatter<'t, W, GroupLabel<FromGroupKey>, Extra> {
        PrettyFormatter {
            common: CommonFormatter {
                target: self.common.target,
                color_setting: self.common.color_setting,
                tests: self.common.tests,
                _label_marker: PhantomData,
            },
        }
    }

    /// Choose group labels based on the group context.
    ///
    /// This affects only grouped output and uses [`GroupLabel`] with
    /// [`FromGroupCtx`] to derive the display name.
    pub fn with_group_label_from_ctx(
        self,
    ) -> PrettyFormatter<'t, W, GroupLabel<FromGroupCtx>, Extra> {
        PrettyFormatter {
            common: CommonFormatter {
                target: self.common.target,
                color_setting: self.common.color_setting,
                tests: self.common.tests,
                _label_marker: PhantomData,
            },
        }
    }
}

impl<'t, Extra> Default for PrettyFormatter<'t, io::Stdout, GroupLabel<FromGroupKey>, Extra> {
    fn default() -> Self {
        Self {
            common: Default::default(),
        }
    }
}

impl<'t, W: io::Write + SupportsColor, L, Extra> PrettyFormatter<'t, W, L, Extra> {
    /// Return whether this formatter will currently emit colored output.
    pub fn use_color(&self) -> bool {
        self.common.use_color()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
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

#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct PrettyRunOutcomes<'t> {
    pub passed: usize,
    pub failed: usize,
    pub ignored: usize,
    pub filtered_out: usize,
    pub duration: Duration,
    pub failures: Vec<PrettyFailure<'t>>,
}

#[derive(Debug, Clone)]
#[non_exhaustive]
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

    type RunInit = fto::Tests<'t, Extra>;
    fn fmt_run_init(&mut self, data: Self::RunInit) -> Result<(), Self::Error> {
        self.common.fmt_run_init(data)
    }

    type RunStart = fto::TestCount;
    fn fmt_run_start(&mut self, data: Self::RunStart) -> Result<(), Self::Error> {
        self.common.fmt_run_start(data)
    }

    type TestOutcome = PrettyTestOutcome<'t>;
    fn fmt_test_outcome(&mut self, data: Self::TestOutcome) -> Result<(), Self::Error> {
        let use_color = self.use_color();
        let target = &mut self.common.target;

        write!(target, "test {}", data.name)?;
        if let PanicExpectation::ShouldPanic | PanicExpectation::ShouldPanicWithExpected(..) =
            data.should_panic
        {
            write!(target, " - should panic")?;
        }
        write!(target, " ... ")?;
        match (data.status, use_color) {
            (TestStatus::Passed, true) => write!(target, "{GREEN}ok{RESET}")?,
            (TestStatus::Passed, false) => write!(target, "ok")?,
            (
                TestStatus::Ignored {
                    reason: Some(reason),
                },
                true,
            ) => write!(target, "{YELLOW}ignored, {reason}{RESET}")?,
            (
                TestStatus::Ignored {
                    reason: Some(reason),
                },
                false,
            ) => write!(target, "ignored, {reason}")?,
            (TestStatus::Ignored { reason: None }, true) => {
                write!(target, "{YELLOW}ignored{RESET}")?
            }
            (TestStatus::Ignored { reason: None }, false) => write!(target, "ignored")?,
            (TestStatus::TimedOut, true) => write!(target, "{RED}timed out{RESET}")?,
            (TestStatus::TimedOut, false) => write!(target, "timed out")?,
            (TestStatus::Failed(_test_failure), true) => write!(target, "{RED}FAILED{RESET}")?,
            (TestStatus::Failed(_test_failure), false) => write!(target, "FAILED")?,
            (TestStatus::Other(_), true) => write!(target, "{CYAN}other{RESET}")?,
            (TestStatus::Other(_), false) => write!(target, "other")?,
        };
        writeln!(target)
    }

    type RunOutcomes = fto::RunOutcomes<'t>;
    fn fmt_run_outcomes(&mut self, data: Self::RunOutcomes) -> Result<(), Self::Error> {
        self.common.fmt_run_outcomes(data)
    }

    type TestIgnored = ();
    type TestStart = ();
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
    type GroupedRunStart = fto::TestCount;
    fn fmt_grouped_run_start(&mut self, data: Self::GroupedRunStart) -> Result<(), Self::Error> {
        self.common.fmt_grouped_run_start(data)
    }

    type GroupStart = fto::GroupStart<L>;
    fn fmt_group_start(&mut self, data: Self::GroupStart) -> Result<(), Self::Error> {
        self.common.fmt_group_start(data)
    }

    type GroupedRunOutcomes = fto::GroupedRunOutcomes<'t>;
    fn fmt_grouped_run_outcomes(
        &mut self,
        data: Self::GroupedRunOutcomes,
    ) -> Result<(), Self::Error> {
        self.common.fmt_grouped_run_outcomes(data)
    }

    type GroupOutcomes = ();
}

impl<'t, Extra: 't, W: io::Write, L> TestListFormatter<'t, Extra>
    for PrettyFormatter<'t, W, L, Extra>
{
    type Error = io::Error;

    type ListTest = TestName<'t>;
    fn fmt_list_test(&mut self, data: Self::ListTest) -> Result<(), Self::Error> {
        writeln!(self.common.target, "{}: test", data.0)
    }

    type EndListing = fto::TestCount;
    fn fmt_end_listing(&mut self, data: Self::EndListing) -> Result<(), Self::Error> {
        match data.0 {
            1 => writeln!(self.common.target, "\n1 test"),
            n => writeln!(self.common.target, "\n{n} tests"),
        }
    }

    type InitListing = ();
    type BeginListing = ();
}
