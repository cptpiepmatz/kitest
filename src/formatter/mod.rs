use std::{
    borrow::Cow,
    io,
    marker::PhantomData,
    ops::{Deref, DerefMut},
    time::Duration,
};

use crate::{
    GroupedTestOutcomes, TestOutcomes,
    group::TestGroups,
    meta::{TestMeta, TestResult},
    outcome::TestOutcome,
};

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

pub struct TestGroupResult;

pub struct FmtRunInitData<'m, Extra> {
    pub tests: &'m [TestMeta<Extra>],
}

pub struct FmtRunStartData {
    pub tests: usize,
    pub filtered: usize,
}

pub struct FmtTestIgnored<'m, 'r, Extra> {
    pub meta: &'m TestMeta<Extra>,
    pub reason: Option<&'r str>,
}

pub struct FmtTestStart<'m, Extra> {
    pub meta: &'m TestMeta<Extra>,
}

pub struct FmtTestOutcome<'m, 'o, Extra> {
    pub meta: &'m TestMeta<Extra>,
    pub outcome: &'o TestOutcome,
}

pub struct FmtRunOutcomes<'m> {
    pub outcomes: &'m TestOutcomes<'m>,
    pub duration: Duration,
}

pub trait TestFormatter<Extra>: Send {
    type RunInit: for<'m> From<FmtRunInitData<'m, Extra>> + Send;
    fn fmt_run_init(&mut self, data: Self::RunInit) -> io::Result<()> {
        let _ = data;
        Ok(())
    }

    type RunStart: From<FmtRunStartData> + Send;
    fn fmt_run_start(&mut self, data: Self::RunStart) -> io::Result<()> {
        let _ = data;
        Ok(())
    }

    type TestIgnored: for<'m, 'r> From<FmtTestIgnored<'m, 'r, Extra>> + Send;
    fn fmt_test_ignored(&mut self, data: Self::TestIgnored) -> io::Result<()> {
        let _ = data;
        Ok(())
    }

    type TestStart: for<'m> From<FmtTestStart<'m, Extra>> + Send;
    fn fmt_test_start(&mut self, data: Self::TestStart) -> io::Result<()> {
        let _ = data;
        Ok(())
    }

    type TestOutcome: for<'m, 'o> From<FmtTestOutcome<'m, 'o, Extra>> + Send;
    fn fmt_test_outcome(&mut self, data: Self::TestOutcome) -> io::Result<()> {
        let _ = data;
        Ok(())
    }

    type RunOutcomes: for<'m> From<FmtRunOutcomes<'m>> + Send;
    fn fmt_run_outcomes(&mut self, data: Self::RunOutcomes) -> io::Result<()> {
        let _ = data;
        Ok(())
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

pub struct FmtGroupOutcomes<'g, 'm, GroupKey> {
    pub key: &'g GroupKey,
    pub outcomes: &'m TestOutcomes<'m>,
    pub duration: Duration,
}

pub struct FmtGroupedRunOutcomes<'m, GroupKey> {
    pub outcomes: &'m GroupedTestOutcomes<'m, GroupKey>,
    pub duration: Duration,
}

pub trait GroupedTestFormatter<GroupKey, Extra>: TestFormatter<Extra> {
    type GroupedRunStart: From<FmtGroupedRunStart> + Send;
    fn fmt_grouped_run_start(&mut self, data: Self::GroupedRunStart) -> io::Result<()> {
        let _ = data;
        Ok(())
    }

    type GroupStart: for<'g> From<FmtGroupStart<'g, GroupKey>> + Send;
    fn fmt_group_start(&mut self, data: Self::GroupStart) -> io::Result<()> {
        let _ = data;
        Ok(())
    }

    type GroupOutcomes: for<'g, 'm> From<FmtGroupOutcomes<'g, 'm, GroupKey>> + Send;
    fn fmt_group_outcomes(&mut self, data: Self::GroupOutcomes) -> io::Result<()> {
        let _ = data;
        Ok(())
    }

    type GroupedRunOutcomes: for<'m> From<FmtGroupedRunOutcomes<'m, GroupKey>> + Send;
    fn fmt_grouped_run_outcomes(&mut self, data: Self::GroupedRunOutcomes) -> io::Result<()> {
        let _ = data;
        Ok(())
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
    FmtRunOutcomes<'m>,

    FmtGroupedRunStart,
    FmtGroupStart<'g, GroupKey>,
    FmtGroupOutcomes<'g, 'm, GroupKey>,
    FmtGroupedRunOutcomes<'m, GroupKey>,
];

impl<Extra> TestFormatter<Extra> for NoFormatter {
    type RunInit = ();
    type RunStart = ();
    type TestIgnored = ();
    type TestStart = ();
    type TestOutcome = ();
    type RunOutcomes = ();
}

impl<GroupKey, Extra> GroupedTestFormatter<GroupKey, Extra> for NoFormatter {
    type GroupedRunStart = ();
    type GroupStart = ();
    type GroupOutcomes = ();
    type GroupedRunOutcomes = ();
}
