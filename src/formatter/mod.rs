use std::io;

use crate::meta::{TestMeta, TestResult};

pub struct Report;

pub struct GroupReport;

pub struct TestGroupResult;

pub trait TestFormatter<Extra> {
    fn fmt_run_init(&mut self) -> io::Result<()> {
        Ok(())
    }

    fn fmt_run_start(&mut self, tests: &[&TestMeta<Extra>], filtered: usize) -> io::Result<()> {
        let _ = (tests, filtered);
        Ok(())
    }

    fn fmt_test_ignored(&mut self, meta: &TestMeta<Extra>, reason: &str) -> io::Result<()> {
        let _ = (meta, reason);
        Ok(())
    }

    fn fmt_test_start(&mut self, meta: &TestMeta<Extra>) -> io::Result<()> {
        let _ = meta;
        Ok(())
    }

    fn fmt_test_result(&mut self, meta: &TestMeta<Extra>, result: &TestResult) -> io::Result<()> {
        let _ = (meta, result);
        Ok(())
    }

    fn fmt_run_report(&mut self, report: &Report) -> io::Result<()> {
        let _ = report;
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

    fn fmt_grouped_run_report(&mut self, report: &GroupReport) -> io::Result<()> {
        let _ = report;
        Ok(())
    }
}
