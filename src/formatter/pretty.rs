use std::{io, time::Duration};

use crate::{formatter::*, outcome::TestStatus};

pub use super::common::{ColorSetting, TestName};

#[derive(Debug)]
pub struct PrettyFormatter<W: io::Write + io::IsTerminal> {
    pub target: W,
    pub color_settings: ColorSetting,
}

impl Default for PrettyFormatter<io::Stdout> {
    fn default() -> Self {
        Self {
            target: io::stdout(),
            color_settings: Default::default(),
        }
    }
}

impl<W: io::Write + io::IsTerminal> PrettyFormatter<W> {
    pub fn use_color(&self) -> bool {
        match self.color_settings {
            ColorSetting::Automatic => self.target.is_terminal(),
            ColorSetting::Always => true,
            ColorSetting::Never => false,
        }
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

#[derive(Debug, PartialEq, Eq)]
pub struct PrettyTestOutcome<'t> {
    pub name: &'t str,
    pub status: TestStatus,
}

impl<'t, 'o, Extra> From<FmtTestOutcome<'t, 'o, Extra>> for PrettyTestOutcome<'t> {
    fn from(value: FmtTestOutcome<'t, 'o, Extra>) -> Self {
        Self {
            name: value.meta.name.as_ref(),
            status: value.outcome.status.clone(),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct PrettyRunOutcomes {
    pub passed: usize,
    pub failed: usize,
    pub ignored: usize,
    pub filtered_out: usize,
    pub duration: Duration,
}

impl<'t, 'o> From<FmtRunOutcomes<'t, 'o>> for PrettyRunOutcomes {
    fn from(value: FmtRunOutcomes<'t, 'o>) -> Self {
        Self {
            passed: value
                .outcomes
                .values()
                .filter(|outcome| outcome.status == TestStatus::Passed)
                .count(),
            failed: value
                .outcomes
                .values()
                .filter(|outcome| matches!(outcome.status, TestStatus::Failed(_)))
                .count(),
            ignored: value
                .outcomes
                .values()
                .filter(|outcome| matches!(outcome.status, TestStatus::Ignored { .. }))
                .count(),
            filtered_out: value.filtered_out,
            duration: value.duration,
        }
    }
}

impl<'t, Extra: 't, W: io::Write + io::IsTerminal + Send> TestFormatter<'t, Extra>
    for PrettyFormatter<W>
{
    type Error = io::Error;

    type RunStart = PrettyTestCount;
    fn fmt_run_start(&mut self, data: Self::RunStart) -> Result<(), Self::Error> {
        writeln!(self.target, "\nrunning {} tests", data.0)
    }

    type TestOutcome = PrettyTestOutcome<'t>;
    fn fmt_test_outcome(&mut self, data: Self::TestOutcome) -> Result<(), Self::Error> {
        write!(self.target, "test {} ... ", data.name)?;
        match data.status {
            TestStatus::Passed => write!(self.target, "ok")?,
            TestStatus::Ignored {
                reason: Some(reason),
            } => write!(self.target, "ignored, {}", reason)?,
            TestStatus::Ignored { reason: None } => write!(self.target, "ignored")?,
            TestStatus::TimedOut => todo!(),
            TestStatus::Failed(_test_failure) => todo!(),
            TestStatus::Other(_) => todo!(),
        };
        writeln!(self.target)
    }

    type RunOutcomes = PrettyRunOutcomes;
    fn fmt_run_outcomes(
        &mut self,
        PrettyRunOutcomes {
            passed,
            failed,
            ignored,
            filtered_out,
            duration,
        }: Self::RunOutcomes,
    ) -> Result<(), Self::Error> {
        writeln!(
            self.target,
            "\ntest result: ok. {passed} passed; {failed} failed; {ignored} ignored; {filtered_out} filtered out; finished in {:.2}s",
            duration.as_secs_f64()
        )
    }

    type RunInit = ();
    type TestIgnored = ();
    type TestStart = ();
}

impl<'t, Extra: 't, W: io::Write + io::IsTerminal> TestListFormatter<'t, Extra>
    for PrettyFormatter<W>
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
