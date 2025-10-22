use std::{borrow::Cow, io, ops::{Deref, DerefMut}, time::Duration};

use crate::{
    GroupedTestOutcomes, TestOutcomes,
    meta::{TestMeta, TestResult},
    outcome::TestOutcome,
};

pub enum FmtTestData<I, S, O> {
    Ignored(I),
    Start(S),
    Outcome(O),
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

pub trait GroupedTestFormatter<GroupKey, Extra>: TestFormatter<Extra> {
    fn fmt_grouped_run_start(&mut self) -> io::Result<()> {
        Ok(())
    }

    fn fmt_group_start(&mut self, key: &GroupKey, tests: &[&TestMeta<Extra>]) -> io::Result<()> {
        let _ = (key, tests);
        Ok(())
    }

    fn fmt_group_result(
        &mut self,
        key: &GroupKey,
        tests: &[&TestMeta<Extra>],
        result: &TestGroupResult,
    ) -> io::Result<()> {
        let _ = (key, tests, result);
        Ok(())
    }

    fn fmt_grouped_run_outcomes(
        &mut self,
        outcomes: &GroupedTestOutcomes<'_, GroupKey>,
        duration: Duration,
    ) -> io::Result<()> {
        let _ = (outcomes, duration);
        Ok(())
    }
}

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
    FmtRunOutcomes<'m>
];

impl<Extra> TestFormatter<Extra> for NoFormatter {
    type RunInit = ();
    type RunStart = ();
    type TestIgnored = ();
    type TestStart = ();
    type TestOutcome = ();
    type RunOutcomes = ();
}

impl<GroupKey, Extra> GroupedTestFormatter<GroupKey, Extra> for NoFormatter {}
