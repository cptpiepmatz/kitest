pub use std::io;

use crate::{
    formatter::{
        common::{
            color::{ColorSetting, SupportsColor, colors::*},
            *,
        },
        *,
    },
    outcome::TestStatus,
};

#[derive(Debug)]
pub struct TerseFormatter<'t, W: io::Write, Extra> {
    common: CommonFormatter<'t, W, Extra>,
    progress: usize,
    last_ok: bool,
}

impl<'t, Extra> Default for TerseFormatter<'t, io::Stdout, Extra> {
    fn default() -> Self {
        Self {
            common: CommonFormatter::default(),
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
            common: CommonFormatter {
                target: with_target,
                color_setting: self.common.color_setting,
                tests: self.common.tests,
            },
            progress: self.progress,
            last_ok: self.last_ok,
        }
    }

    pub fn with_color_setting(self, color_setting: impl Into<ColorSetting>) -> Self {
        TerseFormatter {
            common: CommonFormatter {
                color_setting: color_setting.into(),
                ..self.common
            },
            ..self
        }
    }
}

impl<'t, W: io::Write + SupportsColor, Extra> TerseFormatter<'t, W, Extra> {
    /// Return whether this formatter will currently emit colored output.
    pub fn use_color(&self) -> bool {
        self.common.use_color()
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
        self.common.fmt_run_init(data)
    }

    type RunStart = fto::TestCount;
    fn fmt_run_start(&mut self, data: Self::RunStart) -> Result<(), Self::Error> {
        self.common.fmt_run_start(data)
    }

    type TestOutcome = TerseTestOutcome<'t>;
    fn fmt_test_outcome(&mut self, data: Self::TestOutcome) -> Result<(), Self::Error> {
        let write_res = match data.status {
            TestStatus::Passed => write!(self.common.target, "."),
            TestStatus::Ignored { .. } => write!(self.common.target, "i"),
            TestStatus::Other(..) => write!(self.common.target, "o"),
            TestStatus::Failed(..) | TestStatus::TimedOut => {
                if self.last_ok {
                    writeln!(
                        self.common.target,
                        " {}/{}",
                        self.progress,
                        self.common.tests.len()
                    )?;
                }
                writeln!(self.common.target, "{} --- FAILED", data.name)
            }
        };

        match data.status {
            TestStatus::Passed | TestStatus::Ignored { .. } | TestStatus::Other(..) => {
                self.last_ok = true
            }
            TestStatus::TimedOut | TestStatus::Failed(..) => self.last_ok = false,
        }

        self.progress += 1;

        write_res
    }

    type RunOutcomes = fto::RunOutcomes<'t>;
    fn fmt_run_outcomes(&mut self, data: Self::RunOutcomes) -> Result<(), Self::Error> {
        self.common.fmt_run_outcomes(data)
    }

    type TestIgnored = ();
    type TestStart = ();
}

impl<'t, Extra: 't, W: io::Write> TestListFormatter<'t, Extra> for TerseFormatter<'t, W, Extra> {
    type Error = io::Error;

    type ListTest = TestName<'t>;
    fn fmt_list_test(&mut self, data: Self::ListTest) -> Result<(), Self::Error> {
        writeln!(self.common.target, "{}: test", data.0)
    }

    type InitListing = ();
    type BeginListing = ();
    type EndListing = ();
}

// TODO: need to implement formatting for running tests
