use std::{
    fmt::{self, Debug, Formatter},
    io::{self, Stdout},
};

use crate::{capture::OutputCapture, formatter::*, outcome::TestStatus};

type BoxedDebugFormatter<T> = Box<dyn (Fn(&mut Formatter, &T) -> fmt::Result) + Send>;

// TODO: explain in this, that this formatter should only be used for debugging as its operations
//       are pretty wasteful, for production (or at least runnings tests as they should), write a
//       proper test formatter that is using only what is needed

pub struct DebugFormatter<'t, W: io::Write, Extra> {
    target: W,

    fmt_run_init: BoxedDebugFormatter<DebugRunInit<'t, Extra>>,
    fmt_run_start: BoxedDebugFormatter<DebugRunStart>,
    fmt_test_ignored: BoxedDebugFormatter<DebugTestIgnored<'t, Extra>>,
    fmt_test_start: BoxedDebugFormatter<DebugTestStart<'t, Extra>>,
    fmt_test_outcome: BoxedDebugFormatter<DebugTestOutcome<'t, Extra>>,
    fmt_run_outcomes: BoxedDebugFormatter<DebugRunOutcomes>,
}

fn debug_fmt<T: Debug>(f: &mut Formatter, d: &T) -> fmt::Result {
    d.fmt(f)?;
    writeln!(f)
}

impl<'t, Extra: Debug> Default for DebugFormatter<'t, Stdout, Extra> {
    fn default() -> Self {
        Self {
            target: io::stdout(),

            fmt_run_init: Box::new(|f, d| debug_fmt(f, d)),
            fmt_run_start: Box::new(|f, d| debug_fmt(f, d)),
            fmt_test_ignored: Box::new(|f, d| debug_fmt(f, d)),
            fmt_test_start: Box::new(|f, d| debug_fmt(f, d)),
            fmt_test_outcome: Box::new(|f, d| debug_fmt(f, d)),
            fmt_run_outcomes: Box::new(|f, d| debug_fmt(f, d)),
        }
    }
}

macro_rules! with_formatter {
    {$(($ty:ty, $field:ident, $with:ident, $default:ident),)*} => {$(
        pub fn $with(
            self,
            f: impl (Fn(&mut Formatter, &$ty) -> fmt::Result) + Send + 'static,
        ) -> Self {
            Self {
                $field: Box::new(f),
                ..self
            }
        }

        pub fn $default(self) -> Self {
            Self {
                $field: Box::new(|f, d| debug_fmt(f, d)),
                ..self
            }
        }
    )*};
}

impl<'t, W: io::Write, Extra: Debug> DebugFormatter<'t, W, Extra> {
    pub fn with_target<WithTarget: io::Write>(
        self,
        target: WithTarget,
    ) -> DebugFormatter<'t, WithTarget, Extra> {
        DebugFormatter {
            target,
            fmt_run_init: self.fmt_run_init,
            fmt_run_start: self.fmt_run_start,
            fmt_test_ignored: self.fmt_test_ignored,
            fmt_test_start: self.fmt_test_start,
            fmt_test_outcome: self.fmt_test_outcome,
            fmt_run_outcomes: self.fmt_run_outcomes,
        }
    }

    pub fn with_no_formatter(self) -> Self {
        Self {
            target: self.target,

            fmt_run_init: Box::new(|_, _| Ok(())),
            fmt_run_start: Box::new(|_, _| Ok(())),
            fmt_test_ignored: Box::new(|_, _| Ok(())),
            fmt_test_start: Box::new(|_, _| Ok(())),
            fmt_test_outcome: Box::new(|_, _| Ok(())),
            fmt_run_outcomes: Box::new(|_, _| Ok(())),
        }
    }

    #[rustfmt::skip]
    with_formatter! {
        (DebugRunInit<'t, Extra>, fmt_run_init, with_run_init_formatter, with_default_run_init_formatter),
        (DebugRunStart, fmt_run_start, with_run_start_formatter, with_default_run_start_formatter),
        (DebugTestIgnored<'t, Extra>, fmt_test_ignored, with_test_ignored_formatter, with_default_test_ignored_formatter),
        (DebugTestStart<'t, Extra>, fmt_test_start, with_test_start_formatter, with_default_test_start_formatter),
        (DebugTestOutcome<'t, Extra>, fmt_test_outcome, with_test_outcome_formatter, with_default_test_outcome_formatter),
        (DebugRunOutcomes, fmt_run_outcomes, with_run_outcomes_formatter, with_default_run_outcomes_formatter),
    }
}

#[non_exhaustive]
#[derive(Debug)]
pub struct DebugRunInit<'t, Extra> {
    pub tests: &'t [Test<Extra>],
}

impl<'t, Extra> From<FmtRunInit<'t, Extra>> for DebugRunInit<'t, Extra> {
    fn from(value: FmtRunInit<'t, Extra>) -> Self {
        let FmtRunInit { tests } = value;
        Self { tests }
    }
}

#[non_exhaustive]
#[derive(Debug)]
pub struct DebugRunStart {
    pub active: usize,
    pub filtered: usize,
}

impl From<FmtRunStart> for DebugRunStart {
    fn from(value: FmtRunStart) -> Self {
        let FmtRunStart { active, filtered } = value;
        Self { active, filtered }
    }
}

#[non_exhaustive]
#[derive(Debug)]
pub struct DebugTestIgnored<'t, Extra> {
    pub meta: &'t TestMeta<Extra>,
    pub reason: Option<String>,
}

impl<'t, 'r, Extra> From<FmtTestIgnored<'t, 'r, Extra>> for DebugTestIgnored<'t, Extra> {
    fn from(value: FmtTestIgnored<'t, 'r, Extra>) -> Self {
        let FmtTestIgnored { meta, reason } = value;
        Self {
            meta,
            reason: reason.map(ToString::to_string),
        }
    }
}

#[non_exhaustive]
#[derive(Debug)]
pub struct DebugTestStart<'t, Extra> {
    pub meta: &'t TestMeta<Extra>,
}

impl<'t, Extra> From<FmtTestStart<'t, Extra>> for DebugTestStart<'t, Extra> {
    fn from(value: FmtTestStart<'t, Extra>) -> Self {
        let FmtTestStart { meta } = value;
        Self { meta }
    }
}

#[non_exhaustive]
#[derive(Debug)]
pub struct DebugTestOutcome<'t, Extra> {
    pub meta: &'t TestMeta<Extra>,
    pub outcome: TestOutcome,
}

impl<'t, 'o, Extra> From<FmtTestOutcome<'t, 'o, Extra>> for DebugTestOutcome<'t, Extra> {
    fn from(value: FmtTestOutcome<'t, 'o, Extra>) -> Self {
        let FmtTestOutcome { meta, outcome } = value;
        Self {
            meta,
            outcome: outcome.into(),
        }
    }
}

#[non_exhaustive]
#[derive(Debug)]
pub struct DebugRunOutcomes {
    pub outcomes: Vec<(String, TestOutcome)>,
    pub filtered_out: usize,
    pub duration: Duration,
}

impl<'t, 'o> From<FmtRunOutcomes<'t, 'o>> for DebugRunOutcomes {
    fn from(value: FmtRunOutcomes) -> Self {
        let FmtRunOutcomes {
            outcomes,
            filtered_out,
            duration,
        } = value;
        Self {
            outcomes: outcomes
                .into_iter()
                .map(|(name, outcome)| (name.to_string(), outcome.into()))
                .collect(),
            filtered_out,
            duration,
        }
    }
}

impl<'t, W, Extra> TestFormatter<'t, Extra> for DebugFormatter<'t, W, Extra>
where
    W: io::Write + Send,
    Extra: 't + Sync + Debug,
{
    type Error = io::Error;

    type RunInit = DebugRunInit<'t, Extra>;
    fn fmt_run_init(&mut self, data: Self::RunInit) -> Result<(), Self::Error> {
        write!(
            self.target,
            "{:?}",
            fmt::from_fn(|f| (self.fmt_run_init)(f, &data))
        )
    }

    type RunStart = DebugRunStart;
    fn fmt_run_start(&mut self, data: Self::RunStart) -> Result<(), Self::Error> {
        write!(
            self.target,
            "{:?}",
            fmt::from_fn(|f| (self.fmt_run_start)(f, &data))
        )
    }

    type TestIgnored = DebugTestIgnored<'t, Extra>;
    fn fmt_test_ignored(&mut self, data: Self::TestIgnored) -> Result<(), Self::Error> {
        write!(
            self.target,
            "{:?}",
            fmt::from_fn(|f| (self.fmt_test_ignored)(f, &data))
        )
    }

    type TestStart = DebugTestStart<'t, Extra>;
    fn fmt_test_start(&mut self, data: Self::TestStart) -> Result<(), Self::Error> {
        write!(
            self.target,
            "{:?}",
            fmt::from_fn(|f| (self.fmt_test_start)(f, &data))
        )
    }

    type TestOutcome = DebugTestOutcome<'t, Extra>;
    fn fmt_test_outcome(&mut self, data: Self::TestOutcome) -> Result<(), Self::Error> {
        write!(
            self.target,
            "{:?}",
            fmt::from_fn(|f| (self.fmt_test_outcome)(f, &data))
        )
    }

    type RunOutcomes = DebugRunOutcomes;
    fn fmt_run_outcomes(&mut self, data: Self::RunOutcomes) -> Result<(), Self::Error> {
        write!(
            self.target,
            "{:?}",
            fmt::from_fn(|f| (self.fmt_run_outcomes)(f, &data))
        )
    }
}

#[expect(dead_code, reason = "used for Debug")]
#[derive(Debug)]
pub struct TestOutcome {
    status: TestStatus,
    duration: Duration,
    output: OutputCapture,
}

impl From<&crate::outcome::TestOutcome> for TestOutcome {
    fn from(value: &crate::outcome::TestOutcome) -> Self {
        // outcome attachments are too arbitrary to keep them in the debug output
        let crate::outcome::TestOutcome {
            status,
            duration,
            output,
            attachments: _,
        } = value;
        Self {
            status: status.clone(),
            duration: duration.clone(),
            output: output.clone(),
        }
    }
}
