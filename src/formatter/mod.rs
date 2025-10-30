use std::{borrow::Cow, num::NonZeroUsize, time::Duration};

use crate::{
    GroupedTestOutcomes, TestOutcomes,
    ignore::IgnoreStatus,
    outcome::TestOutcome,
    test::{Test, TestMeta},
};

mod common;
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

pub trait TestFormatter<'t, Extra: 't>: Send {
    type Error: Send + 't;

    type RunInit: From<FmtRunInit<'t, Extra>> + Send;
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

pub trait GroupedTestListFormatter<'t, Extra: 't, GroupKey: 't, GroupCtx: 't>:
    TestListFormatter<'t, Extra>
{
    type ListGroups: From<FmtListGroups>;
    fn fmt_list_groups(&mut self, data: Self::ListGroups) -> Result<(), Self::Error> {
        discard!(data)
    }

    type ListGroupStart: for<'g> From<FmtListGroupStart<'g, GroupKey, GroupCtx>>;
    fn fmt_list_group_start(&mut self, data: Self::ListGroupStart) -> Result<(), Self::Error> {
        discard!(data)
    }

    type ListGroupEnd: for<'g> From<FmtListGroupEnd<'g, GroupKey, GroupCtx>>;
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
    FmtGroupedRunOutcomes<'t, 'o, GroupKey>: GroupedRunOutcomes,
    FmtInitListing<'t, Extra>: InitListing,
    FmtBeginListing: BeginListing,
    FmtListTest<'t, Extra>: ListTest,
    FmtEndListing: EndListing,
    FmtListGroups: ListGroups,
    FmtListGroupStart<'g, GroupKey, GroupCtx>: ListGroupStart,
    FmtListGroupEnd<'g, GroupKey, GroupCtx>: ListGroupEnd,
}
