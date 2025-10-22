use std::{borrow::Cow, io, time::Duration};

use crate::{
    GroupedTestOutcomes, TestOutcomes,
    meta::{TestMeta, TestResult},
    outcome::TestOutcome,
};

pub enum FmtTestData<'m, 'o, Extra> {
    Ignored {
        meta: &'m TestMeta<Extra>,
        reason: Option<Cow<'static, str>>,
    },

    Start {
        meta: &'m TestMeta<Extra>,
    },

    Outcome {
        name: &'m str,
        outcome: &'o TestOutcome,
    },
}

pub struct TestGroupResult;

pub trait TestFormatter<Extra> {
    fn fmt_run_init(&mut self) -> io::Result<()> {
        Ok(())
    }

    fn fmt_run_start(&mut self, tests: &[&TestMeta<Extra>], filtered: usize) -> io::Result<()> {
        let _ = (tests, filtered);
        Ok(())
    }

    fn fmt_test_ignored(&mut self, meta: &TestMeta<Extra>, reason: Option<&str>) -> io::Result<()> {
        let _ = (meta, reason);
        Ok(())
    }

    fn fmt_test_start(&mut self, meta: &TestMeta<Extra>) -> io::Result<()> {
        let _ = meta;
        Ok(())
    }

    fn fmt_test_outcome(&mut self, name: &str, outcome: &TestOutcome) -> io::Result<()> {
        let _ = (name, outcome);
        Ok(())
    }

    fn fmt_run_outcomes(
        &mut self,
        outcomes: &TestOutcomes<'_>,
        duration: Duration,
    ) -> io::Result<()> {
        let _ = (outcomes, duration);
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
impl<Extra> TestFormatter<Extra> for NoFormatter {}
impl<GroupKey, Extra> GroupedTestFormatter<GroupKey, Extra> for NoFormatter {}
