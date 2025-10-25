use std::{borrow::Cow, io, time::Duration};

use crate::{GroupedTestOutcomes, TestOutcomes, meta::TestMeta, outcome::TestOutcome};

macro_rules! discard {
    ($data:expr) => {{
        let _ = $data;
        Ok(())
    }}
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
    pub tests: &'m [TestMeta<Extra>],
}

pub struct FmtRunStartData {
    pub tests: usize,
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
    pub duration: Duration,
}

pub trait TestFormatter<'m, Extra: 'm>: Send {
    type RunInit: From<FmtRunInitData<'m, Extra>> + Send;
    fn fmt_run_init(&mut self, data: Self::RunInit) -> io::Result<()> {
        discard!(data)
    }

    type RunStart: From<FmtRunStartData> + Send;
    fn fmt_run_start(&mut self, data: Self::RunStart) -> io::Result<()> {
        discard!(data)
    }

    type TestIgnored: for<'r> From<FmtTestIgnored<'m, 'r, Extra>> + Send;
    fn fmt_test_ignored(&mut self, data: Self::TestIgnored) -> io::Result<()> {
        discard!(data)
    }

    type TestStart: From<FmtTestStart<'m, Extra>> + Send;
    fn fmt_test_start(&mut self, data: Self::TestStart) -> io::Result<()> {
        discard!(data)
    }

    type TestOutcome: for<'o> From<FmtTestOutcome<'m, 'o, Extra>> + Send;
    fn fmt_test_outcome(&mut self, data: Self::TestOutcome) -> io::Result<()> {
        discard!(data)
    }

    type RunOutcomes: for<'o> From<FmtRunOutcomes<'m, 'o>> + Send;
    fn fmt_run_outcomes(&mut self, data: Self::RunOutcomes) -> io::Result<()> {
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

pub struct FmtGroupOutcomes<'m, 'g, 'o,  GroupKey> {
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
    fn fmt_grouped_run_start(&mut self, data: Self::GroupedRunStart) -> io::Result<()> {
        discard!(data)
    }

    type GroupStart: for<'g> From<FmtGroupStart<'g, GroupKey>> + Send;
    fn fmt_group_start(&mut self, data: Self::GroupStart) -> io::Result<()> {
        discard!(data)
    }

    type GroupOutcomes: for<'g, 'o> From<FmtGroupOutcomes<'m, 'g, 'o, GroupKey>> + Send;
    fn fmt_group_outcomes(&mut self, data: Self::GroupOutcomes) -> io::Result<()> {
        discard!(data)
    }

    type GroupedRunOutcomes: for<'o> From<FmtGroupedRunOutcomes<'m, 'o, GroupKey>> + Send;
    fn fmt_grouped_run_outcomes(&mut self, data: Self::GroupedRunOutcomes) -> io::Result<()> {
        discard!(data)
    }
}

pub struct FmtBeginListing<'m, Extra> {
    pub tests: &'m [TestMeta<Extra>],
}

pub struct FmtListTest<'m, Extra> {
    pub meta: &'m TestMeta<Extra>,
    pub ignored: (bool, Option<&'m Cow<'static, str>>)
}

pub trait TestListFormatter {
    fn fmt_begin_listing(&mut self, data: ()) -> io::Result<()> {
        discard!(data)
    }

    fn fmt_list_test(&mut self, data: ()) -> io::Result<()> {
        discard!(data)
    }

    fn fmt_end_listing(&mut self, data: ()) -> io::Result<()> {
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
    FmtRunStartData,
    FmtTestIgnored<'m, 'r, Extra>,
    FmtTestStart<'m, Extra>,
    FmtTestOutcome<'m, 'o, Extra>,
    FmtRunOutcomes<'m, 'o>,
    FmtGroupedRunStart,
    FmtGroupStart<'g, GroupKey>,
    FmtGroupOutcomes<'m, 'g, 'o, GroupKey>,
    FmtGroupedRunOutcomes<'m, 'o, GroupKey>,
];

impl<'m, Extra: 'm> TestFormatter<'m, Extra> for NoFormatter {
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
