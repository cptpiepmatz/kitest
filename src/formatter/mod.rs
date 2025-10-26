use std::{borrow::Cow, time::Duration};

use crate::{
    GroupedTestOutcomes, TestOutcomes,
    meta::{Test, TestMeta},
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

pub struct FmtRunInitData<'m, Extra> {
    pub tests: &'m [Test<Extra>],
}

pub struct FmtRunStart {
    pub active: usize,
    pub filtered: usize,
}

pub struct FmtTestIgnored<'m, 'r, Extra> {
    pub meta: &'m TestMeta<Extra>,
    pub reason: Option<&'r Cow<'static, str>>,
}

pub struct FmtTestStart<'m, Extra> {
    pub meta: &'m TestMeta<Extra>,
}

pub struct FmtTestOutcome<'m, 'o, Extra> {
    pub meta: &'m TestMeta<Extra>,
    pub outcome: &'o TestOutcome,
}

pub struct FmtRunOutcomes<'m, 'o> {
    pub outcomes: &'o TestOutcomes<'m>,
    pub filtered_out: usize,
    pub duration: Duration,
}

pub trait TestFormatter<'m, Extra: 'm>: Send {
    type Error: Send + 'm;

    type RunInit: From<FmtRunInitData<'m, Extra>> + Send;
    fn fmt_run_init(&mut self, data: Self::RunInit) -> Result<(), Self::Error> {
        discard!(data)
    }

    type RunStart: From<FmtRunStart> + Send;
    fn fmt_run_start(&mut self, data: Self::RunStart) -> Result<(), Self::Error> {
        discard!(data)
    }

    type TestIgnored: for<'r> From<FmtTestIgnored<'m, 'r, Extra>> + Send;
    fn fmt_test_ignored(&mut self, data: Self::TestIgnored) -> Result<(), Self::Error> {
        discard!(data)
    }

    type TestStart: From<FmtTestStart<'m, Extra>> + Send;
    fn fmt_test_start(&mut self, data: Self::TestStart) -> Result<(), Self::Error> {
        discard!(data)
    }

    type TestOutcome: for<'o> From<FmtTestOutcome<'m, 'o, Extra>> + Send;
    fn fmt_test_outcome(&mut self, data: Self::TestOutcome) -> Result<(), Self::Error> {
        discard!(data)
    }

    type RunOutcomes: for<'o> From<FmtRunOutcomes<'m, 'o>> + Send;
    fn fmt_run_outcomes(&mut self, data: Self::RunOutcomes) -> Result<(), Self::Error> {
        discard!(data)
    }
}

pub struct FmtGroupedRunStart {
    pub tests: usize,
    pub filtered: usize,
}

pub struct FmtGroupStart<'g, GroupKey> {
    pub key: &'g GroupKey,
    pub tests: usize,
}

pub struct FmtGroupOutcomes<'m, 'g, 'o, GroupKey> {
    pub key: &'g GroupKey,
    pub outcomes: &'o TestOutcomes<'m>,
    pub duration: Duration,
}

pub struct FmtGroupedRunOutcomes<'m, 'o, GroupKey> {
    pub outcomes: &'o GroupedTestOutcomes<'m, GroupKey>,
    pub duration: Duration,
}

pub trait GroupedTestFormatter<'m, GroupKey: 'm, Extra: 'm>: TestFormatter<'m, Extra> {
    type GroupedRunStart: From<FmtGroupedRunStart> + Send;
    fn fmt_grouped_run_start(&mut self, data: Self::GroupedRunStart) -> Result<(), Self::Error> {
        discard!(data)
    }

    type GroupStart: for<'g> From<FmtGroupStart<'g, GroupKey>> + Send;
    fn fmt_group_start(&mut self, data: Self::GroupStart) -> Result<(), Self::Error> {
        discard!(data)
    }

    type GroupOutcomes: for<'g, 'o> From<FmtGroupOutcomes<'m, 'g, 'o, GroupKey>> + Send;
    fn fmt_group_outcomes(&mut self, data: Self::GroupOutcomes) -> Result<(), Self::Error> {
        discard!(data)
    }

    type GroupedRunOutcomes: for<'o> From<FmtGroupedRunOutcomes<'m, 'o, GroupKey>> + Send;
    fn fmt_grouped_run_outcomes(
        &mut self,
        data: Self::GroupedRunOutcomes,
    ) -> Result<(), Self::Error> {
        discard!(data)
    }
}

pub struct FmtInitListing<'m, Extra> {
    pub tests: &'m [Test<Extra>],
}

pub struct FmtBeginListing {
    pub tests: usize,
    pub filtered: usize,
}

pub struct FmtListTest<'m, Extra> {
    pub meta: &'m TestMeta<Extra>,
    pub ignored: (bool, Option<Cow<'static, str>>),
}

pub struct FmtEndListing {
    pub active: usize,
    pub ignored: usize,
}

pub trait TestListFormatter<'m, Extra: 'm> {
    type Error: 'm;

    type InitListing: From<FmtInitListing<'m, Extra>>;
    fn fmt_init_listing(&mut self, data: Self::InitListing) -> Result<(), Self::Error> {
        discard!(data)
    }

    type BeginListing: From<FmtBeginListing>;
    fn fmt_begin_listing(&mut self, data: Self::BeginListing) -> Result<(), Self::Error> {
        discard!(data)
    }

    type ListTest: From<FmtListTest<'m, Extra>>;
    fn fmt_list_test(&mut self, data: Self::ListTest) -> Result<(), Self::Error> {
        discard!(data)
    }

    type EndListing: From<FmtEndListing>;
    fn fmt_end_listing(&mut self, data: Self::EndListing) -> Result<(), Self::Error> {
        discard!(data)
    }
}

pub struct FmtListGroupStart<'g, GroupKey> {
    pub key: &'g GroupKey,
    pub tests: usize,
}

pub struct FmtListGroupEnd<'g, GroupKey> {
    pub key: &'g GroupKey,
    pub tests: usize,
}

pub trait GroupedTestListFormatter<'m, GroupKey: 'm, Extra: 'm>:
    TestListFormatter<'m, Extra>
{
    type ListGroupStart: for<'g> From<FmtListGroupStart<'g, GroupKey>>;
    fn fmt_list_group_start(&mut self, data: Self::ListGroupStart) -> Result<(), Self::Error> {
        discard!(data)
    }

    type ListGroupEnd: for<'g> From<FmtListGroupEnd<'g, GroupKey>>;
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
    FmtRunInitData<'m, Extra>,
    FmtRunStart,
    FmtTestIgnored<'m, 'r, Extra>,
    FmtTestStart<'m, Extra>,
    FmtTestOutcome<'m, 'o, Extra>,
    FmtRunOutcomes<'m, 'o>,
    FmtGroupedRunStart,
    FmtGroupStart<'g, GroupKey>,
    FmtGroupOutcomes<'m, 'g, 'o, GroupKey>,
    FmtGroupedRunOutcomes<'m, 'o, GroupKey>,
    FmtInitListing<'m, Extra>,
    FmtBeginListing,
    FmtListTest<'m, Extra>,
    FmtEndListing,
    FmtListGroupStart<'g, GroupKey>,
    FmtListGroupEnd<'g, GroupKey>,
];

impl<'m, Extra: 'm> TestFormatter<'m, Extra> for NoFormatter {
    type Error = ();
    type RunInit = ();
    type RunStart = ();
    type TestIgnored = ();
    type TestStart = ();
    type TestOutcome = ();
    type RunOutcomes = ();
}

impl<'m, GroupKey: 'm, Extra: 'm> GroupedTestFormatter<'m, GroupKey, Extra> for NoFormatter {
    type GroupedRunStart = ();
    type GroupStart = ();
    type GroupOutcomes = ();
    type GroupedRunOutcomes = ();
}

impl<'m, Extra: 'm> TestListFormatter<'m, Extra> for NoFormatter {
    type Error = ();
    type InitListing = ();
    type BeginListing = ();
    type ListTest = ();
    type EndListing = ();
}

impl<'m, GroupKey: 'm, Extra: 'm> GroupedTestListFormatter<'m, GroupKey, Extra> for NoFormatter {
    type ListGroupStart = ();
    type ListGroupEnd = ();
}
