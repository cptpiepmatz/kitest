pub use std::io;
use std::{fmt::Display, marker::PhantomData};

use crate::{
    formatter::{
        common::{
            color::{ColorSetting, SupportsColor, colors::*},
            label::{FromGroupCtx, FromGroupKey, GroupLabel},
            *,
        },
        *,
    },
    outcome::TestStatus,
};

/// A compact formatter that keeps output minimal, while still being readable.
///
/// Instead of printing one full status line per test, it prints a single character per test:
/// - `.` for passed
/// - `i` for ignored
/// - `o` for other
///
/// On the first failing or timed out test, it switches to printing failure lines so we can
/// immediately see what broke.
///
/// Coloring is controlled via [`ColorSetting`]. In automatic mode, the formatter uses the
/// target's [`SupportsColor`] implementation to decide if color should be used.
#[derive(Debug)]
pub struct TerseFormatter<'t, W: io::Write, L, Extra> {
    common: CommonFormatter<'t, W, L, Extra>,
    progress: usize,
    last_ok: bool,
}

impl<'t, Extra> Default for TerseFormatter<'t, io::Stdout, GroupLabel<FromGroupKey>, Extra> {
    fn default() -> Self {
        Self {
            common: CommonFormatter::default(),
            progress: 0,
            last_ok: false,
        }
    }
}

impl<'t, W: io::Write, L, Extra> TerseFormatter<'t, W, L, Extra> {
    /// Create a `TerseFormatter` that writes to stdout.
    ///
    /// By default, group labels are derived from the group key via [`GroupLabel`].
    pub fn new() -> TerseFormatter<'t, io::Stdout, GroupLabel<FromGroupKey>, Extra> {
        TerseFormatter::default()
    }

    /// Replace the output target.
    ///
    /// This can be used to write into a file, a buffer, or any other writer.
    pub fn with_target<WithTarget: io::Write>(
        self,
        with_target: WithTarget,
    ) -> TerseFormatter<'t, WithTarget, L, Extra> {
        TerseFormatter {
            common: CommonFormatter {
                target: with_target,
                color_setting: self.common.color_setting,
                tests: self.common.tests,
                _label_marker: self.common._label_marker,
            },
            progress: self.progress,
            last_ok: self.last_ok,
        }
    }

    /// Replace the color settings.
    pub fn with_color_setting(self, color_setting: impl Into<ColorSetting>) -> Self {
        TerseFormatter {
            common: CommonFormatter {
                color_setting: color_setting.into(),
                ..self.common
            },
            ..self
        }
    }

    /// Choose group labels based on the group key.
    ///
    /// This affects only grouped output and uses [`GroupLabel`] with
    /// [`FromGroupKey`] to derive the display name.
    pub fn with_group_label_from_key(
        self,
    ) -> TerseFormatter<'t, W, GroupLabel<FromGroupKey>, Extra> {
        TerseFormatter {
            common: CommonFormatter {
                target: self.common.target,
                color_setting: self.common.color_setting,
                tests: self.common.tests,
                _label_marker: PhantomData,
            },
            progress: self.progress,
            last_ok: self.last_ok,
        }
    }

    /// Choose group labels based on the group context.
    ///
    /// This affects only grouped output and uses [`GroupLabel`] with
    /// [`FromGroupCtx`] to derive the display name.
    pub fn with_group_label_from_ctx(
        self,
    ) -> TerseFormatter<'t, W, GroupLabel<FromGroupCtx>, Extra> {
        TerseFormatter {
            common: CommonFormatter {
                target: self.common.target,
                color_setting: self.common.color_setting,
                tests: self.common.tests,
                _label_marker: PhantomData,
            },
            progress: self.progress,
            last_ok: self.last_ok,
        }
    }
}

impl<'t, W: io::Write + SupportsColor, L, Extra> TerseFormatter<'t, W, L, Extra> {
    /// Return whether this formatter will currently emit colored output.
    pub fn use_color(&self) -> bool {
        self.common.use_color()
    }
}

#[derive(Debug, Clone)]
#[non_exhaustive]
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

impl<'t, W: io::Write + Send + SupportsColor, L: Send, Extra: 't + Sync> TestFormatter<'t, Extra>
    for TerseFormatter<'t, W, L, Extra>
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
        let use_color = self.use_color();
        let green = if use_color { GREEN } else { "" };
        let yellow = if use_color { YELLOW } else { "" };
        let cyan = if use_color { CYAN } else { "" };
        let red = if use_color { RED } else { "" };
        let reset = if use_color { RESET } else { "" };

        let target = &mut self.common.target;
        let write_res = match data.status {
            TestStatus::Passed => write!(target, "{green}.{reset}"),
            TestStatus::Ignored { .. } => write!(target, "{yellow}i{reset}"),
            TestStatus::Other(..) => write!(target, "{cyan}o{reset}"),
            TestStatus::Failed(..) | TestStatus::TimedOut => {
                if self.last_ok {
                    writeln!(
                        self.common.target,
                        " {}/{}",
                        self.progress,
                        self.common.tests.len()
                    )?;
                }
                writeln!(self.common.target, "{} --- {red}FAILED{reset}", data.name)
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

impl<'t, W: io::Write, L, Extra: 't> TestListFormatter<'t, Extra>
    for TerseFormatter<'t, W, L, Extra>
{
    type Error = io::Error;

    type ListTest = TestName<'t>;
    fn fmt_list_test(&mut self, data: Self::ListTest) -> Result<(), Self::Error> {
        writeln!(self.common.target, "{}: test", data.0)
    }

    type InitListing = ();
    type BeginListing = ();
    type EndListing = ();
}

impl<'t, GroupKey, GroupCtx, W, L, Extra> GroupedTestFormatter<'t, Extra, GroupKey, GroupCtx>
    for TerseFormatter<'t, W, L, Extra>
where
    GroupKey: 't,
    GroupCtx: 't,
    W: io::Write + SupportsColor + Send,
    L: Send + Display,
    Extra: 't + Sync,
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

    type GroupedRunOutcomes = fto::GroupedRunOutcomes;
    fn fmt_grouped_run_outcomes(
        &mut self,
        data: Self::GroupedRunOutcomes,
    ) -> Result<(), Self::Error> {
        self.common.fmt_grouped_run_outcomes(data)
    }

    type GroupOutcomes = ();
}
