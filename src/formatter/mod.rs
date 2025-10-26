use std::{borrow::Cow, time::Duration};

use crate::{
    GroupedTestOutcomes, TestOutcomes,
    test::{Test, TestMeta},
    outcome::TestOutcome,
};

mod common;
pub mod pretty;
pub mod terse;

macro_rules! discard {
    ($data:expr) => {{
        let _ = $data;
        Ok(())
    }};
}

pub(crate) enum FmtTestData<I, S, O> {
    Ignored(I),
    Start(S),
    Outcome(O),
}

pub(crate) enum FmtGroupedTestData<I, S, O, GS, GO> {
    Test(FmtTestData<I, S, O>),
    Start(GS),
    Outcome(GO),
}

pub struct FmtRunInitData<'t, Extra> {
    pub tests: &'t [Test<Extra>],
}

pub struct FmtRunStart {
    pub active: usize,
    pub filtered: usize,
}

pub struct FmtTestIgnored<'t, 'r, Extra> {
    pub meta: &'t TestMeta<Extra>,
    pub reason: Option<&'r Cow<'static, str>>,
}

pub struct FmtTestStart<'t, Extra> {
    pub meta: &'t TestMeta<Extra>,
}

pub struct FmtTestOutcome<'t, 'o, Extra> {
    pub meta: &'t TestMeta<Extra>,
    pub outcome: &'o TestOutcome,
}

pub struct FmtRunOutcomes<'t, 'o> {
    pub outcomes: &'o TestOutcomes<'t>,
    pub filtered_out: usize,
    pub duration: Duration,
}

pub trait TestFormatter<'t, Extra: 't>: Send {
    type Error: Send + 't;

    type RunInit: From<FmtRunInitData<'t, Extra>> + Send;
    fn fmt_run_init(&mut self, data: Self::RunInit) -> Result<(), Self::Error> {
        discard!(data)
    }

    type RunStart: From<FmtRunStart> + Send;
    fn fmt_run_start(&mut self, data: Self::RunStart) -> Result<(), Self::Error> {
        discard!(data)
    }

    type TestIgnored: for<'r> From<FmtTestIgnored<'t, 'r, Extra>> + Send;
    fn fmt_test_ignored(&mut self, data: Self::TestIgnored) -> Result<(), Self::Error> {
        discard!(data)
    }

    type TestStart: From<FmtTestStart<'t, Extra>> + Send;
    fn fmt_test_start(&mut self, data: Self::TestStart) -> Result<(), Self::Error> {
        discard!(data)
    }

    type TestOutcome: for<'o> From<FmtTestOutcome<'t, 'o, Extra>> + Send;
    fn fmt_test_outcome(&mut self, data: Self::TestOutcome) -> Result<(), Self::Error> {
        discard!(data)
    }

    type RunOutcomes: for<'o> From<FmtRunOutcomes<'t, 'o>> + Send;
    fn fmt_run_outcomes(&mut self, data: Self::RunOutcomes) -> Result<(), Self::Error> {
        discard!(data)
    }
}

pub struct FmtGroupedRunStart {
    pub tests: usize,
    pub filtered: usize,
}

pub struct FmtGroupStart<'g, GroupKey, GroupCtx = ()> {
    pub tests: usize,
    pub key: &'g GroupKey,
    pub ctx: Option<&'g GroupCtx>,
}

pub struct FmtGroupOutcomes<'t, 'g, 'o, GroupKey, GroupCtx = ()> {
    pub outcomes: &'o TestOutcomes<'t>,
    pub duration: Duration,
    pub key: &'g GroupKey,
    pub ctx: Option<&'g GroupCtx>,
}

pub struct FmtGroupedRunOutcomes<'t, 'o, GroupKey> {
    pub outcomes: &'o GroupedTestOutcomes<'t, GroupKey>,
    pub duration: Duration,
}

pub trait GroupedTestFormatter<'t, Extra: 't, GroupKey: 't, GroupCtx: 't = ()>:
    TestFormatter<'t, Extra>
{
    type GroupedRunStart: From<FmtGroupedRunStart> + Send;
    fn fmt_grouped_run_start(&mut self, data: Self::GroupedRunStart) -> Result<(), Self::Error> {
        discard!(data)
    }

    type GroupStart: for<'g> From<FmtGroupStart<'g, GroupKey, GroupCtx>> + Send;
    fn fmt_group_start(&mut self, data: Self::GroupStart) -> Result<(), Self::Error> {
        discard!(data)
    }

    type GroupOutcomes: for<'g, 'o> From<FmtGroupOutcomes<'t, 'g, 'o, GroupKey, GroupCtx>> + Send;
    fn fmt_group_outcomes(&mut self, data: Self::GroupOutcomes) -> Result<(), Self::Error> {
        discard!(data)
    }

    type GroupedRunOutcomes: for<'o> From<FmtGroupedRunOutcomes<'t, 'o, GroupKey>> + Send;
    fn fmt_grouped_run_outcomes(
        &mut self,
        data: Self::GroupedRunOutcomes,
    ) -> Result<(), Self::Error> {
        discard!(data)
    }
}

pub struct FmtInitListing<'t, Extra> {
    pub tests: &'t [Test<Extra>],
}

pub struct FmtBeginListing {
    pub tests: usize,
    pub filtered: usize,
}

pub struct FmtListTest<'t, Extra> {
    pub meta: &'t TestMeta<Extra>,
    pub ignored: (bool, Option<Cow<'static, str>>),
}

pub struct FmtEndListing {
    pub active: usize,
    pub ignored: usize,
}

pub trait TestListFormatter<'t, Extra: 't> {
    type Error: 't;

    type InitListing: From<FmtInitListing<'t, Extra>>;
    fn fmt_init_listing(&mut self, data: Self::InitListing) -> Result<(), Self::Error> {
        discard!(data)
    }

    type BeginListing: From<FmtBeginListing>;
    fn fmt_begin_listing(&mut self, data: Self::BeginListing) -> Result<(), Self::Error> {
        discard!(data)
    }

    type ListTest: From<FmtListTest<'t, Extra>>;
    fn fmt_list_test(&mut self, data: Self::ListTest) -> Result<(), Self::Error> {
        discard!(data)
    }

    type EndListing: From<FmtEndListing>;
    fn fmt_end_listing(&mut self, data: Self::EndListing) -> Result<(), Self::Error> {
        discard!(data)
    }
}

pub struct FmtListGroupStart<'g, GroupKey, GroupCtx> {
    pub tests: usize,
    pub key: &'g GroupKey,
    pub ctx: &'g GroupCtx,
}

pub struct FmtListGroupEnd<'g, GroupKey, GroupCtx> {
    pub tests: usize,
    pub key: &'g GroupKey,
    pub ctx: &'g GroupCtx,
}

pub trait GroupedTestListFormatter<'t, Extra: 't, GroupKey: 't, GroupCtx: 't>:
    TestListFormatter<'t, Extra>
{
    type ListGroupStart: for<'g> From<FmtListGroupStart<'g, GroupKey, GroupCtx>>;
    fn fmt_list_group_start(&mut self, data: Self::ListGroupStart) -> Result<(), Self::Error> {
        discard!(data)
    }

    type ListGroupEnd: for<'g> From<FmtListGroupEnd<'g, GroupKey, GroupCtx>>;
    fn fmt_list_group_end(&mut self, data: Self::ListGroupEnd) -> Result<(), Self::Error> {
        discard!(data)
    }
}

#[derive(Default)]
pub struct NoFormatter;

macro_rules! impl_unit_from {
    [$($name:ident$(<$($generic:tt),*>)?),* $(,)?] => {$(
        impl$(<$($generic),*>)? From<$name$(<$($generic),*>)?> for () {
            fn from(_: $name$(<$($generic),*>)?) -> () {}
        })*
    };
}

impl_unit_from![
    FmtRunInitData<'t, Extra>,
    FmtRunStart,
    FmtTestIgnored<'t, 'r, Extra>,
    FmtTestStart<'t, Extra>,
    FmtTestOutcome<'t, 'o, Extra>,
    FmtRunOutcomes<'t, 'o>,
    FmtGroupedRunStart,
    FmtGroupStart<'g, GroupKey, GroupCtx>,
    FmtGroupOutcomes<'t, 'g, 'o, GroupKey, GroupCtx>,
    FmtGroupedRunOutcomes<'t, 'o, GroupKey>,
    FmtInitListing<'t, Extra>,
    FmtBeginListing,
    FmtListTest<'t, Extra>,
    FmtEndListing,
    FmtListGroupStart<'g, GroupKey, GroupCtx>,
    FmtListGroupEnd<'g, GroupKey, GroupCtx>,
];

impl<'t, Extra: 't> TestFormatter<'t, Extra> for NoFormatter {
    type Error = ();
    type RunInit = ();
    type RunStart = ();
    type TestIgnored = ();
    type TestStart = ();
    type TestOutcome = ();
    type RunOutcomes = ();
}

impl<'t, Extra: 't, GroupKey: 't, GroupCtx: 't> GroupedTestFormatter<'t, Extra, GroupKey, GroupCtx>
    for NoFormatter
{
    type GroupedRunStart = ();
    type GroupStart = ();
    type GroupOutcomes = ();
    type GroupedRunOutcomes = ();
}

impl<'t, Extra: 't> TestListFormatter<'t, Extra> for NoFormatter {
    type Error = ();
    type InitListing = ();
    type BeginListing = ();
    type ListTest = ();
    type EndListing = ();
}

impl<'t, Extra: 't, GroupKey: 't, GroupCtx: 't>
    GroupedTestListFormatter<'t, Extra, GroupKey, GroupCtx> for NoFormatter
{
    type ListGroupStart = ();
    type ListGroupEnd = ();
}
