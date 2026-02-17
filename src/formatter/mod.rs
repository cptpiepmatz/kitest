//! Formatting output for kitest.
//!
//! A formatter defines what gets formatted and, depending on the implementation,
//! what gets printed to the console.
//!
//! Formatting is event based.
//! The harness emits events while it filters, ignores, and runs tests.
//! Each event uses a lightweight `Fmt*` data type that borrows from the harness as much as
//! possible.
//!
//! Each formatter method has an associated `type` that must be constructible from the `Fmt*` event
//! via `From`.
//! This lets a formatter decide what it wants to clone for its formatter thread, while keeping the
//! harness side cheap.
//!
//! The main traits are [`TestFormatter`] and [`GroupedTestFormatter`].
//! A grouped formatter builds on the regular formatter, so [`GroupedTestFormatter`] extends
//! [`TestFormatter`].

use std::{borrow::Cow, num::NonZeroUsize, time::Duration};

use crate::{
    GroupedTestOutcomes, TestOutcomes,
    ignore::IgnoreStatus,
    outcome::TestOutcome,
    test::{Test, TestMeta},
};

pub mod common;

pub mod no;
pub mod pretty;
pub mod terse;

macro_rules! discard {
    ($data:expr) => {{
        let _ = $data;
        Ok(())
    }};
}

#[derive(Debug)]
pub(crate) enum FmtTestData<I, S, O> {
    Ignored(I),
    Start(S),
    Outcome(O),
}

#[derive(Debug)]
pub(crate) enum FmtGroupedTestData<I, S, O, GS, GO> {
    Test(FmtTestData<I, S, O>),
    Start(GS),
    Outcome(GO),
}

#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct FmtRunInit<'t, Extra> {
    pub tests: &'t [Test<Extra>],
}

#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct FmtRunStart {
    pub active: usize,
    pub filtered: usize,
}

#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct FmtTestIgnored<'t, 'r, Extra> {
    pub meta: &'t TestMeta<Extra>,
    pub reason: Option<&'r Cow<'static, str>>,
}

#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct FmtTestStart<'t, Extra> {
    pub meta: &'t TestMeta<Extra>,
}

#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct FmtTestOutcome<'t, 'o, Extra> {
    pub meta: &'t TestMeta<Extra>,
    pub outcome: &'o TestOutcome,
}

#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct FmtRunOutcomes<'t, 'o> {
    pub outcomes: &'o TestOutcomes<'t>,
    pub filtered_out: usize,
    pub duration: Duration,
}

/// A formatter for a normal (non grouped) test run.
///
/// The harness calls these methods as it progresses through a run.
/// Each method receives a formatter specific data type, which must be constructible from the
/// corresponding `Fmt*` event type.
/// This is used to keep cloning under the formatter's control.
///
/// The associated `Error` type is collected during the run and returned at the end.
pub trait TestFormatter<'t, Extra: 't>: Send {
    /// Formatter specific error type.
    type Error: Send + 't;

    type RunInit: From<FmtRunInit<'t, Extra>> + Send;
    /// Called before anything happens, even before filtering.
    ///
    /// The harness provides [`FmtRunInit`], which includes the full, unfiltered test list.
    fn fmt_run_init(&mut self, data: Self::RunInit) -> Result<(), Self::Error> {
        discard!(data)
    }

    type RunStart: From<FmtRunStart> + Send;
    /// Called after filtering is done and the run is about to start.
    ///
    /// The harness provides [`FmtRunStart`], which includes counts for active and
    /// filtered tests.
    fn fmt_run_start(&mut self, data: Self::RunStart) -> Result<(), Self::Error> {
        discard!(data)
    }

    type TestIgnored: for<'r> From<FmtTestIgnored<'t, 'r, Extra>> + Send;
    /// Called for a test that is ignored.
    ///
    /// The harness provides [`FmtTestIgnored`], including the test metadata and an
    /// optional ignore reason.
    fn fmt_test_ignored(&mut self, data: Self::TestIgnored) -> Result<(), Self::Error> {
        discard!(data)
    }

    type TestStart: From<FmtTestStart<'t, Extra>> + Send;
    /// Called when a test is about to start executing.
    ///
    /// The harness provides [`FmtTestStart`], which includes the test metadata.
    fn fmt_test_start(&mut self, data: Self::TestStart) -> Result<(), Self::Error> {
        discard!(data)
    }

    type TestOutcome: for<'o> From<FmtTestOutcome<'t, 'o, Extra>> + Send;
    /// Called when a test has finished executing.
    ///
    /// The harness provides [`FmtTestOutcome`], which includes the test metadata and
    /// a reference to the produced [`TestOutcome`].
    fn fmt_test_outcome(&mut self, data: Self::TestOutcome) -> Result<(), Self::Error> {
        discard!(data)
    }

    type RunOutcomes: for<'o> From<FmtRunOutcomes<'t, 'o>> + Send;
    /// Called at the end of the run with all outcomes.
    ///
    /// This is only used for non grouped runs. Grouped runs end with
    /// [`GroupedTestFormatter::fmt_grouped_run_outcomes`] instead.
    ///
    /// The harness provides [`FmtRunOutcomes`], including all outcomes, total duration,
    /// and the number of filtered out tests.
    fn fmt_run_outcomes(&mut self, data: Self::RunOutcomes) -> Result<(), Self::Error> {
        discard!(data)
    }
}

#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct FmtGroupedRunStart {
    pub tests: usize,
    pub filtered: usize,
}

#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct FmtGroupStart<'g, GroupKey, GroupCtx = ()> {
    pub tests: usize,
    pub worker_count: NonZeroUsize,
    pub key: &'g GroupKey,
    pub ctx: Option<&'g GroupCtx>,
}

#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct FmtGroupOutcomes<'t, 'g, 'o, GroupKey, GroupCtx = ()> {
    pub outcomes: &'o TestOutcomes<'t>,
    pub duration: Duration,
    pub key: &'g GroupKey,
    pub ctx: Option<&'g GroupCtx>,
}

#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct FmtGroupedRunOutcomes<'t, 'o, GroupKey, GroupCtx> {
    pub outcomes: &'o GroupedTestOutcomes<'t, GroupKey, GroupCtx>,
    pub duration: Duration,
}

/// A formatter for grouped test runs.
///
/// Grouped formatting wraps a run with extra group level events. During grouped
/// execution, the harness still calls the regular [`TestFormatter`] per test events
/// (`fmt_test_start`, `fmt_test_outcome`, etc.), and adds group start and end
/// events around them.
///
/// The grouped run ends with [`Self::fmt_grouped_run_outcomes`], which replaces
/// [`TestFormatter::fmt_run_outcomes`].
pub trait GroupedTestFormatter<'t, Extra: 't, GroupKey: 't, GroupCtx: 't = ()>:
    TestFormatter<'t, Extra>
{
    type GroupedRunStart: From<FmtGroupedRunStart> + Send;
    /// Called after filtering is done and the grouped run is about to start.
    ///
    /// The harness provides [`FmtGroupedRunStart`], which includes counts for total
    /// tests and filtered tests.
    fn fmt_grouped_run_start(&mut self, data: Self::GroupedRunStart) -> Result<(), Self::Error> {
        discard!(data)
    }

    type GroupStart: for<'g> From<FmtGroupStart<'g, GroupKey, GroupCtx>> + Send;
    /// Called when a group is about to start executing.
    ///
    /// The harness provides [`FmtGroupStart`], including the group key, optional
    /// group context, and the worker count used for that group.
    fn fmt_group_start(&mut self, data: Self::GroupStart) -> Result<(), Self::Error> {
        discard!(data)
    }

    type GroupOutcomes: for<'g, 'o> From<FmtGroupOutcomes<'t, 'g, 'o, GroupKey, GroupCtx>> + Send;
    /// Called at the end of a group with that group's outcomes.
    ///
    /// The harness provides [`FmtGroupOutcomes`], including the outcomes for that
    /// group, its duration, and the group key and context.
    fn fmt_group_outcomes(&mut self, data: Self::GroupOutcomes) -> Result<(), Self::Error> {
        discard!(data)
    }

    type GroupedRunOutcomes: for<'o> From<FmtGroupedRunOutcomes<'t, 'o, GroupKey, GroupCtx>> + Send;
    /// Called at the end of the grouped run with all group outcomes.
    ///
    /// This replaces [`TestFormatter::fmt_run_outcomes`] for grouped runs.
    ///
    /// The harness provides [`FmtGroupedRunOutcomes`], including all group outcomes
    /// and total duration.
    fn fmt_grouped_run_outcomes(
        &mut self,
        data: Self::GroupedRunOutcomes,
    ) -> Result<(), Self::Error> {
        discard!(data)
    }
}

#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct FmtInitListing<'t, Extra> {
    pub tests: &'t [Test<Extra>],
}

#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct FmtBeginListing {
    pub tests: usize,
    pub filtered: usize,
}

#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct FmtListTest<'t, Extra> {
    pub meta: &'t TestMeta<Extra>,
    pub ignored: IgnoreStatus,
}

#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct FmtEndListing {
    pub active: usize,
    pub ignored: usize,
}

/// A formatter for listing tests without executing them.
///
/// This works like [`TestFormatter`], but for [`harness.list()`](super::TestHarness::list).
/// The harness emits listing events, and the formatter decides how to present them.
///
/// Each method receives a formatter specific data type, which must be
/// constructible from the corresponding `Fmt*` event type via `From`. This keeps
/// cloning under the formatter's control.
///
/// The associated `Error` type is collected during listing and returned at the end.
pub trait TestListFormatter<'t, Extra: 't> {
    /// Formatter specific error type.
    type Error: 't;

    type InitListing: From<FmtInitListing<'t, Extra>>;
    /// Called before anything happens, even before filtering.
    ///
    /// The harness provides [`FmtInitListing`], which includes the full, unfiltered test list.
    fn fmt_init_listing(&mut self, data: Self::InitListing) -> Result<(), Self::Error> {
        discard!(data)
    }

    type BeginListing: From<FmtBeginListing>;
    /// Called after filtering is done and listing is about to start.
    ///
    /// The harness provides [`FmtBeginListing`], which includes counts for total and
    /// filtered tests.
    fn fmt_begin_listing(&mut self, data: Self::BeginListing) -> Result<(), Self::Error> {
        discard!(data)
    }

    type ListTest: From<FmtListTest<'t, Extra>>;
    /// Called for each test that is part of the listing.
    ///
    /// The harness provides [`FmtListTest`], including test metadata and the ignore decision.
    fn fmt_list_test(&mut self, data: Self::ListTest) -> Result<(), Self::Error> {
        discard!(data)
    }

    type EndListing: From<FmtEndListing>;
    /// Called at the end of listing.
    ///
    /// The harness provides [`FmtEndListing`], including the counts for active and ignored tests.
    fn fmt_end_listing(&mut self, data: Self::EndListing) -> Result<(), Self::Error> {
        discard!(data)
    }
}

#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct FmtListGroups {
    pub groups: usize,
}

#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct FmtListGroupStart<'g, GroupKey, GroupCtx> {
    pub tests: usize,
    pub key: &'g GroupKey,
    pub ctx: Option<&'g GroupCtx>,
}

#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct FmtListGroupEnd<'g, GroupKey, GroupCtx> {
    pub tests: usize,
    pub key: &'g GroupKey,
    pub ctx: Option<&'g GroupCtx>,
}

/// A formatter for listing grouped tests without executing them.
///
/// Grouped listing wraps listing with group level events. The harness still calls
/// the regular [`TestListFormatter`] events for the tests, and adds group events
/// around them.
pub trait GroupedTestListFormatter<'t, Extra: 't, GroupKey: 't, GroupCtx: 't>:
    TestListFormatter<'t, Extra>
{
    type ListGroups: From<FmtListGroups>;
    /// Called once the harness knows how many groups exist.
    ///
    /// This is called after filtering.
    fn fmt_list_groups(&mut self, data: Self::ListGroups) -> Result<(), Self::Error> {
        discard!(data)
    }

    type ListGroupStart: for<'g> From<FmtListGroupStart<'g, GroupKey, GroupCtx>>;
    /// Called at the beginning of each group.
    ///
    /// The harness provides [`FmtListGroupStart`], including the group key, optional
    /// group context, and the number of tests in the group.
    fn fmt_list_group_start(&mut self, data: Self::ListGroupStart) -> Result<(), Self::Error> {
        discard!(data)
    }

    type ListGroupEnd: for<'g> From<FmtListGroupEnd<'g, GroupKey, GroupCtx>>;
    /// Called at the end of each group.
    ///
    /// The harness provides [`FmtListGroupEnd`], including the group key, optional
    /// group context, and the number of tests in the group.
    fn fmt_list_group_end(&mut self, data: Self::ListGroupEnd) -> Result<(), Self::Error> {
        discard!(data)
    }
}

pub(crate) trait IntoFormatError: Sized {
    fn fmt<F, Data, Err>(self, f: F) -> Result<(), (FormatError, Err)>
    where
        F: FnMut(Data) -> Result<(), Err>,
        Data: From<Self>;
}

macro_rules! make_format_error {
    {$($name:ident$(<$($generic:tt),*>)?: $key:ident),* $(,)?} => {$(
        impl$(<$($generic),*>)? IntoFormatError for $name$(<$($generic),*>)? {
            fn fmt<F, Data, Err>(self, mut f: F) -> Result<(), (FormatError, Err)>
            where
                F: FnMut(Data) -> Result<(), Err>,
                Data: From<Self> {
                    f(self.into()).map_err(|err| (FormatError::$key, err))
                }
        })*

        #[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
        #[non_exhaustive]
        pub enum FormatError {
            $($key),*
        }
    };
}

make_format_error! {
    FmtRunInit<'t, Extra>: RunInit,
    FmtRunStart: RunStart,
    FmtTestIgnored<'t, 'r, Extra>: TestIgnored,
    FmtTestStart<'t, Extra>: TestStart,
    FmtTestOutcome<'t, 'o, Extra>: TestOutcome,
    FmtRunOutcomes<'t, 'o>: RunOutcomes,
    FmtGroupedRunStart: GroupedRunStart,
    FmtGroupStart<'g, GroupKey, GroupCtx>: GroupStart,
    FmtGroupOutcomes<'t, 'g, 'o, GroupKey, GroupCtx>: GroupOutcomes,
    FmtGroupedRunOutcomes<'t, 'o, GroupKey, GroupCtx>: GroupedRunOutcomes,
    FmtInitListing<'t, Extra>: InitListing,
    FmtBeginListing: BeginListing,
    FmtListTest<'t, Extra>: ListTest,
    FmtEndListing: EndListing,
    FmtListGroups: ListGroups,
    FmtListGroupStart<'g, GroupKey, GroupCtx>: ListGroupStart,
    FmtListGroupEnd<'g, GroupKey, GroupCtx>: ListGroupEnd,
}
