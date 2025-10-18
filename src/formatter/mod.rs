use std::{fmt::Display, hash::Hash, io, time::Duration};

use crate::{ConclusionGroups, TestMeta};

mod pretty;

pub struct StartData {
    pub scheduled: u64,
    pub filtered: u64,
}

pub enum ColorConfig {
    Auto,
    Always,
    Never,
}

pub enum Status {
    Passed,
    Failed { msg: Option<String> },
    Ignored,
    Error { msg: Option<String> }, // harness error, timeout, etc.
}

pub struct TestOutcome<'a> {
    pub status: Status,
    pub duration: Option<Duration>,
    pub stdout: Option<&'a [u8]>,
    pub stderr: Option<&'a [u8]>,
}

pub trait TestFormatter<GroupKey: Eq + Hash + Display> {
    fn fmt_start(
        &mut self,
        w: &mut dyn io::Write,
        color: &ColorConfig,
        data: StartData,
    ) -> io::Result<()>;

    fn fmt_test_started<'m, E>(
        &mut self,
        w: &mut dyn io::Write,
        color: &ColorConfig,
        group: &GroupKey,
        meta: &TestMeta<'m, E>,
    ) -> io::Result<()> {
        let _ = (w, color, group, meta);
        Ok(())
    }

    fn fmt_test_finished<'m, 'o, E>(
        &mut self,
        w: &mut dyn io::Write,
        color: &ColorConfig,
        group: &GroupKey,
        meta: &TestMeta<'m, E>,
        outcome: &TestOutcome<'o>, // status, duration, stdout/stderr
    ) -> io::Result<()>;

    fn fmt_conclusion(
        &mut self,
        w: &mut dyn io::Write,
        color: &ColorConfig,
        conclusion: &ConclusionGroups<GroupKey>,
    ) -> io::Result<()>;
}
