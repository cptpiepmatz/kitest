use std::collections::HashMap;
pub use std::io;

use crate::{
    formatter::{
        common::{
            color::{ColorSetting, SupportsColor, colors::*},
            *,
        },
        *,
    },
    outcome::{TestFailure, TestStatus},
};

#[derive(Debug)]
pub struct TerseFormatter<'t, W: io::Write, Extra> {
    target: W,
    color_setting: ColorSetting,
    tests: HashMap<&'t str, &'t Test<Extra>>,
    progress: usize,
    last_ok: bool,
}

impl<'t, Extra> Default for TerseFormatter<'t, io::Stdout, Extra> {
    fn default() -> Self {
        Self {
            target: io::stdout(),
            color_setting: Default::default(),
            tests: HashMap::default(),
            progress: 0,
            last_ok: false,
        }
    }
}

impl<'t, W: io::Write, Extra> TerseFormatter<'t, W, Extra> {
    pub fn with_target<WithTarget: io::Write>(
        self,
        with_target: WithTarget,
    ) -> TerseFormatter<'t, WithTarget, Extra> {
        TerseFormatter {
            target: with_target,
            color_setting: self.color_setting,
            tests: self.tests,
            progress: self.progress,
            last_ok: self.last_ok,
        }
    }

    pub fn with_color_setting(self, color_setting: impl Into<ColorSetting>) -> Self {
        TerseFormatter {
            color_setting: color_setting.into(),
            ..self
        }
    }
}

impl<'t, W: io::Write + SupportsColor, Extra> TerseFormatter<'t, W, Extra> {
    /// Return whether this formatter will currently emit colored output.
    pub fn use_color(&self) -> bool {
        match self.color_setting {
            ColorSetting::Automatic => self.target.supports_color(),
            ColorSetting::Always => true,
            ColorSetting::Never => false,
        }
    }
}

pub struct TerseTestOutcome<'t> {
    pub name: &'t str,
    pub status: TestStatus,
}

impl<'t, 'o, Extra> From<FmtTestOutcome<'t, 'o, Extra>> for TerseTestOutcome<'t> {
    fn from(value: FmtTestOutcome<'t, 'o, Extra>) -> Self {
        Self {
            name: value.meta.name.as_ref(),
            status: value.outcome.status.clone(),
        }
    }
}

impl<'t, Extra: 't + Sync, W: io::Write + Send + SupportsColor> TestFormatter<'t, Extra>
    for TerseFormatter<'t, W, Extra>
{
    type Error = io::Error;

    type RunInit = fto::Tests<'t, Extra>;
    fn fmt_run_init(&mut self, data: Self::RunInit) -> Result<(), Self::Error> {
        self.tests = HashMap::from_iter(data.0.iter().map(|test| (test.name.as_ref(), test)));
        Ok(())
    }

    type RunStart = fto::TestCount;
    fn fmt_run_start(&mut self, data: Self::RunStart) -> Result<(), Self::Error> {
        match data.0 {
            1 => writeln!(self.target, "\nrunning 1 test"),
            count => writeln!(self.target, "\nrunning {count} tests"),
        }
    }

    type TestOutcome = TerseTestOutcome<'t>;
    fn fmt_test_outcome(&mut self, data: Self::TestOutcome) -> Result<(), Self::Error> {
        let write_res = match data.status {
            TestStatus::Passed => write!(self.target, "."),
            TestStatus::Ignored { .. } => write!(self.target, "i"),
            TestStatus::Other(..) => write!(self.target, "o"),
            TestStatus::Failed(..) | TestStatus::TimedOut => {
                if self.last_ok {
                    writeln!(self.target, " {}/{}", self.progress, self.tests.len())?;
                }
                writeln!(self.target, "{} --- FAILED", data.name)
            }
        };

        match data.status {
            TestStatus::Passed | TestStatus::Ignored { .. } | TestStatus::Other(..) => {
                self.last_ok = true
            }
            TestStatus::TimedOut | TestStatus::Failed(..) => self.last_ok = false,
        }

        self.progress = self.progress + 1;

        write_res
    }

    type RunOutcomes = fto::RunOutcomes<'t>;
    fn fmt_run_outcomes(
        &mut self,
        fto::RunOutcomes {
            passed,
            failed,
            ignored,
            filtered_out,
            duration,
            failures,
        }: Self::RunOutcomes,
    ) -> Result<(), Self::Error> {
        // TODO: move into common behavior
        if !failures.is_empty() {
            writeln!(self.target)?;
            writeln!(self.target, "failures:")?;
            writeln!(self.target)?;
            for failure in failures.iter() {
                writeln!(self.target, "---- {} stdout ----", failure.name)?;
                match &failure.failure {
                    TestFailure::Error(err) => writeln!(self.target, "Error: {err}")?,
                    TestFailure::Panicked(_) => self.target.write_all(failure.output.raw())?,
                    TestFailure::DidNotPanic { .. } => {
                        if let Some(meta) = self.tests.get(failure.name)
                            && let Some(origin) = &meta.origin
                        {
                            write!(
                                self.target,
                                "note: test did not panic as expected at {origin}"
                            )?;
                        }
                    }
                    TestFailure::PanicMismatch {
                        got: _,
                        expected: None,
                    } => unreachable!("mismatch not possible without expectation"),
                    TestFailure::PanicMismatch {
                        got,
                        expected: Some(expected),
                    } => {
                        self.target.write_all(failure.output.raw())?;
                        writeln!(self.target, "note: panic did not contain expected string")?;
                        writeln!(self.target, "      panic message: {got:?}")?;
                        write!(self.target, " expected substring: {expected:?}")?;
                    }
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
        match (failed, self.use_color()) {
            (0, false) => write!(self.target, "ok. ")?,
            (0, true) => write!(self.target, "{GREEN}ok{RESET}. ")?,
            (_, false) => write!(self.target, "FAILED. ")?,
            (_, true) => write!(self.target, "{RED}FAILED{RESET}. ")?,
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

impl<'t, Extra: 't, W: io::Write> TestListFormatter<'t, Extra> for TerseFormatter<'t, W, Extra> {
    type Error = io::Error;

    type ListTest = TestName<'t>;
    fn fmt_list_test(&mut self, data: Self::ListTest) -> Result<(), Self::Error> {
        writeln!(self.target, "{}: test", data.0)
    }

    type InitListing = ();
    type BeginListing = ();
    type EndListing = ();
}

// TODO: need to implement formatting for running tests
