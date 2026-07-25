use std::{
    fmt::{self, Debug, Formatter},
    io::{self, Stdout},
};

use crate::{capture::OutputCapture, formatter::*, ignore::IgnoreStatus, outcome::TestStatus};

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
    fmt_grouped_run_start: BoxedDebugFormatter<DebugGroupedRunStart>,
    fmt_group_start: BoxedDebugFormatter<DebugGroupStart>,
    fmt_group_outcomes: BoxedDebugFormatter<DebugGroupOutcomes>,
    fmt_grouped_run_outcomes: BoxedDebugFormatter<DebugGroupedRunOutcomes>,
    fmt_init_listing: BoxedDebugFormatter<DebugInitListing<'t, Extra>>,
    fmt_begin_listing: BoxedDebugFormatter<DebugBeginListing>,
    fmt_list_test: BoxedDebugFormatter<DebugListTest<'t, Extra>>,
    fmt_end_listing: BoxedDebugFormatter<DebugEndListing>,
    fmt_list_groups: BoxedDebugFormatter<DebugListGroups>,
    fmt_list_group_start: BoxedDebugFormatter<DebugListGroupStart>,
    fmt_list_group_end: BoxedDebugFormatter<DebugListGroupEnd>,
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
            fmt_grouped_run_start: Box::new(|f, d| debug_fmt(f, d)),
            fmt_group_start: Box::new(|f, d| debug_fmt(f, d)),
            fmt_group_outcomes: Box::new(|f, d| debug_fmt(f, d)),
            fmt_grouped_run_outcomes: Box::new(|f, d| debug_fmt(f, d)),
            fmt_init_listing: Box::new(|f, d| debug_fmt(f, d)),
            fmt_begin_listing: Box::new(|f, d| debug_fmt(f, d)),
            fmt_list_test: Box::new(|f, d| debug_fmt(f, d)),
            fmt_end_listing: Box::new(|f, d| debug_fmt(f, d)),
            fmt_list_groups: Box::new(|f, d| debug_fmt(f, d)),
            fmt_list_group_start: Box::new(|f, d| debug_fmt(f, d)),
            fmt_list_group_end: Box::new(|f, d| debug_fmt(f, d)),
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
            fmt_grouped_run_start: self.fmt_grouped_run_start,
            fmt_group_start: self.fmt_group_start,
            fmt_group_outcomes: self.fmt_group_outcomes,
            fmt_grouped_run_outcomes: self.fmt_grouped_run_outcomes,
            fmt_init_listing: self.fmt_init_listing,
            fmt_begin_listing: self.fmt_begin_listing,
            fmt_list_test: self.fmt_list_test,
            fmt_end_listing: self.fmt_end_listing,
            fmt_list_groups: self.fmt_list_groups,
            fmt_list_group_start: self.fmt_list_group_start,
            fmt_list_group_end: self.fmt_list_group_end,
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
            fmt_grouped_run_start: Box::new(|_, _| Ok(())),
            fmt_group_start: Box::new(|_, _| Ok(())),
            fmt_group_outcomes: Box::new(|_, _| Ok(())),
            fmt_grouped_run_outcomes: Box::new(|_, _| Ok(())),
            fmt_init_listing: Box::new(|_, _| Ok(())),
            fmt_begin_listing: Box::new(|_, _| Ok(())),
            fmt_list_test: Box::new(|_, _| Ok(())),
            fmt_end_listing: Box::new(|_, _| Ok(())),
            fmt_list_groups: Box::new(|_, _| Ok(())),
            fmt_list_group_start: Box::new(|_, _| Ok(())),
            fmt_list_group_end: Box::new(|_, _| Ok(())),
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
        (DebugGroupedRunStart, fmt_grouped_run_start, with_grouped_run_start_formatter, with_default_grouped_run_start_formatter),
        (DebugGroupStart, fmt_group_start, with_group_start_formatter, with_default_group_start_formatter),
        (DebugGroupOutcomes, fmt_group_outcomes, with_group_outcomes_formatter, with_default_group_outcomes_formatter),
        (DebugGroupedRunOutcomes, fmt_grouped_run_outcomes, with_grouped_run_outcomes_formatter, with_default_grouped_run_outcomes_formatter),
        (DebugInitListing<'t, Extra>, fmt_init_listing, with_init_listing_formatter, with_default_init_listing_formatter),
        (DebugBeginListing, fmt_begin_listing, with_begin_listing_formatter, with_default_begin_listing_formatter),
        (DebugListTest<'t, Extra>, fmt_list_test, with_list_test_formatter, with_default_list_test_formatter),
        (DebugEndListing, fmt_end_listing, with_end_listing_formatter, with_default_end_listing_formatter),
        (DebugListGroups, fmt_list_groups, with_list_groups_formatter, with_default_list_groups_formatter),
        (DebugListGroupStart, fmt_list_group_start, with_list_group_start_formatter, with_default_list_group_start_formatter),
        (DebugListGroupEnd, fmt_list_group_end, with_list_group_end_formatter, with_default_list_group_end_formatter),
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

#[non_exhaustive]
#[derive(Debug)]
pub struct DebugGroupedRunStart {
    pub tests: usize,
    pub filtered: usize,
}

impl From<FmtGroupedRunStart> for DebugGroupedRunStart {
    fn from(value: FmtGroupedRunStart) -> Self {
        let FmtGroupedRunStart { tests, filtered } = value;
        Self { tests, filtered }
    }
}

#[non_exhaustive]
#[derive(Debug)]
pub struct DebugGroupStart {
    pub tests: usize,
    pub worker_count: usize,
    pub key: String,
    pub ctx: Option<String>,
}

impl<'g, GroupKey: Debug, GroupCtx: Debug> From<FmtGroupStart<'g, GroupKey, GroupCtx>>
    for DebugGroupStart
{
    fn from(value: FmtGroupStart<'g, GroupKey, GroupCtx>) -> Self {
        let FmtGroupStart {
            tests,
            worker_count,
            key,
            ctx,
        } = value;
        Self {
            tests,
            worker_count: worker_count.get(),
            key: format!("{key:?}"),
            ctx: ctx.map(|ctx| format!("{ctx:?}")),
        }
    }
}

#[non_exhaustive]
#[derive(Debug)]
pub struct DebugGroupOutcomes {
    pub outcomes: Vec<(String, TestOutcome)>,
    pub duration: Duration,
    pub key: String,
    pub ctx: Option<String>,
}

impl<'t, 'g, 'o, GroupKey: Debug, GroupCtx: Debug>
    From<FmtGroupOutcomes<'t, 'g, 'o, GroupKey, GroupCtx>> for DebugGroupOutcomes
{
    fn from(value: FmtGroupOutcomes<'t, 'g, 'o, GroupKey, GroupCtx>) -> Self {
        let FmtGroupOutcomes {
            outcomes,
            duration,
            key,
            ctx,
        } = value;
        Self {
            outcomes: outcomes
                .iter()
                .map(|(name, outcome)| (name.to_string(), outcome.into()))
                .collect(),
            duration,
            key: format!("{key:?}"),
            ctx: ctx.map(|ctx| format!("{ctx:?}")),
        }
    }
}

#[non_exhaustive]
#[derive(Debug)]
pub struct DebugGroupedRunOutcomes {
    pub outcomes: Vec<(String, Vec<(String, TestOutcome)>, Option<String>)>,
    pub duration: Duration,
}

impl<'t, 'o, GroupKey: Debug, GroupCtx: Debug>
    From<FmtGroupedRunOutcomes<'t, 'o, GroupKey, GroupCtx>> for DebugGroupedRunOutcomes
{
    fn from(value: FmtGroupedRunOutcomes<'t, 'o, GroupKey, GroupCtx>) -> Self {
        let FmtGroupedRunOutcomes { outcomes, duration } = value;
        Self {
            outcomes: outcomes
                .iter()
                .map(|(key, outcomes, ctx)| {
                    (
                        format!("{key:?}"),
                        outcomes
                            .iter()
                            .map(|(name, outcome)| (name.to_string(), outcome.into()))
                            .collect(),
                        ctx.as_ref().map(|ctx| format!("{ctx:?}")),
                    )
                })
                .collect(),
            duration,
        }
    }
}

#[non_exhaustive]
#[derive(Debug)]
pub struct DebugInitListing<'t, Extra> {
    pub tests: &'t [Test<Extra>],
}

impl<'t, Extra> From<FmtInitListing<'t, Extra>> for DebugInitListing<'t, Extra> {
    fn from(value: FmtInitListing<'t, Extra>) -> Self {
        let FmtInitListing { tests } = value;
        Self { tests }
    }
}

#[non_exhaustive]
#[derive(Debug)]
pub struct DebugBeginListing {
    pub tests: usize,
    pub filtered: usize,
}

impl From<FmtBeginListing> for DebugBeginListing {
    fn from(value: FmtBeginListing) -> Self {
        let FmtBeginListing { tests, filtered } = value;
        Self { tests, filtered }
    }
}

#[non_exhaustive]
#[derive(Debug)]
pub struct DebugListTest<'t, Extra> {
    pub meta: &'t TestMeta<Extra>,
    pub ignored: IgnoreStatus,
}

impl<'t, Extra> From<FmtListTest<'t, Extra>> for DebugListTest<'t, Extra> {
    fn from(value: FmtListTest<'t, Extra>) -> Self {
        let FmtListTest { meta, ignored } = value;
        Self { meta, ignored }
    }
}

#[non_exhaustive]
#[derive(Debug)]
pub struct DebugEndListing {
    pub active: usize,
    pub ignored: usize,
}

impl From<FmtEndListing> for DebugEndListing {
    fn from(value: FmtEndListing) -> Self {
        let FmtEndListing { active, ignored } = value;
        Self { active, ignored }
    }
}

#[non_exhaustive]
#[derive(Debug)]
pub struct DebugListGroups {
    pub groups: usize,
}

impl From<FmtListGroups> for DebugListGroups {
    fn from(value: FmtListGroups) -> Self {
        let FmtListGroups { groups } = value;
        Self { groups }
    }
}

#[non_exhaustive]
#[derive(Debug)]
pub struct DebugListGroupStart {
    pub tests: usize,
    pub key: String,
    pub ctx: Option<String>,
}

impl<'g, GroupKey: Debug, GroupCtx: Debug> From<FmtListGroupStart<'g, GroupKey, GroupCtx>>
    for DebugListGroupStart
{
    fn from(value: FmtListGroupStart<'g, GroupKey, GroupCtx>) -> Self {
        let FmtListGroupStart { tests, key, ctx } = value;
        Self {
            tests,
            key: format!("{key:?}"),
            ctx: ctx.map(|ctx| format!("{ctx:?}")),
        }
    }
}

#[non_exhaustive]
#[derive(Debug)]
pub struct DebugListGroupEnd {
    pub tests: usize,
    pub key: String,
    pub ctx: Option<String>,
}

impl<'g, GroupKey: Debug, GroupCtx: Debug> From<FmtListGroupEnd<'g, GroupKey, GroupCtx>>
    for DebugListGroupEnd
{
    fn from(value: FmtListGroupEnd<'g, GroupKey, GroupCtx>) -> Self {
        let FmtListGroupEnd { tests, key, ctx } = value;
        Self {
            tests,
            key: format!("{key:?}"),
            ctx: ctx.map(|ctx| format!("{ctx:?}")),
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

impl<'t, W, Extra, GroupKey, GroupCtx> GroupedTestFormatter<'t, Extra, GroupKey, GroupCtx>
    for DebugFormatter<'t, W, Extra>
where
    W: io::Write + Send,
    Extra: 't + Sync + Debug,
    GroupKey: 't + Debug,
    GroupCtx: 't + Debug,
{
    type GroupedRunStart = DebugGroupedRunStart;
    fn fmt_grouped_run_start(&mut self, data: Self::GroupedRunStart) -> Result<(), Self::Error> {
        write!(
            self.target,
            "{:?}",
            fmt::from_fn(|f| (self.fmt_grouped_run_start)(f, &data))
        )
    }

    type GroupStart = DebugGroupStart;
    fn fmt_group_start(&mut self, data: Self::GroupStart) -> Result<(), Self::Error> {
        write!(
            self.target,
            "{:?}",
            fmt::from_fn(|f| (self.fmt_group_start)(f, &data))
        )
    }

    type GroupOutcomes = DebugGroupOutcomes;
    fn fmt_group_outcomes(&mut self, data: Self::GroupOutcomes) -> Result<(), Self::Error> {
        write!(
            self.target,
            "{:?}",
            fmt::from_fn(|f| (self.fmt_group_outcomes)(f, &data))
        )
    }

    type GroupedRunOutcomes = DebugGroupedRunOutcomes;
    fn fmt_grouped_run_outcomes(
        &mut self,
        data: Self::GroupedRunOutcomes,
    ) -> Result<(), Self::Error> {
        write!(
            self.target,
            "{:?}",
            fmt::from_fn(|f| (self.fmt_grouped_run_outcomes)(f, &data))
        )
    }
}

impl<'t, W, Extra> TestListFormatter<'t, Extra> for DebugFormatter<'t, W, Extra>
where
    W: io::Write + Send,
    Extra: 't + Sync + Debug,
{
    type Error = io::Error;

    type InitListing = DebugInitListing<'t, Extra>;
    fn fmt_init_listing(&mut self, data: Self::InitListing) -> Result<(), Self::Error> {
        write!(
            self.target,
            "{:?}",
            fmt::from_fn(|f| (self.fmt_init_listing)(f, &data))
        )
    }

    type BeginListing = DebugBeginListing;
    fn fmt_begin_listing(&mut self, data: Self::BeginListing) -> Result<(), Self::Error> {
        write!(
            self.target,
            "{:?}",
            fmt::from_fn(|f| (self.fmt_begin_listing)(f, &data))
        )
    }

    type ListTest = DebugListTest<'t, Extra>;
    fn fmt_list_test(&mut self, data: Self::ListTest) -> Result<(), Self::Error> {
        write!(
            self.target,
            "{:?}",
            fmt::from_fn(|f| (self.fmt_list_test)(f, &data))
        )
    }

    type EndListing = DebugEndListing;
    fn fmt_end_listing(&mut self, data: Self::EndListing) -> Result<(), Self::Error> {
        write!(
            self.target,
            "{:?}",
            fmt::from_fn(|f| (self.fmt_end_listing)(f, &data))
        )
    }
}

impl<'t, W, Extra, GroupKey, GroupCtx> GroupedTestListFormatter<'t, Extra, GroupKey, GroupCtx>
    for DebugFormatter<'t, W, Extra>
where
    W: io::Write + Send,
    Extra: 't + Sync + Debug,
    GroupKey: 't + Debug,
    GroupCtx: 't + Debug,
{
    type ListGroups = DebugListGroups;
    fn fmt_list_groups(&mut self, data: Self::ListGroups) -> Result<(), Self::Error> {
        write!(
            self.target,
            "{:?}",
            fmt::from_fn(|f| (self.fmt_list_groups)(f, &data))
        )
    }

    type ListGroupStart = DebugListGroupStart;
    fn fmt_list_group_start(&mut self, data: Self::ListGroupStart) -> Result<(), Self::Error> {
        write!(
            self.target,
            "{:?}",
            fmt::from_fn(|f| (self.fmt_list_group_start)(f, &data))
        )
    }

    type ListGroupEnd = DebugListGroupEnd;
    fn fmt_list_group_end(&mut self, data: Self::ListGroupEnd) -> Result<(), Self::Error> {
        write!(
            self.target,
            "{:?}",
            fmt::from_fn(|f| (self.fmt_list_group_end)(f, &data))
        )
    }
}

#[non_exhaustive]
#[derive(Debug)]
pub struct TestOutcome {
    pub status: TestStatus,
    pub duration: Duration,
    pub output: OutputCapture,
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
