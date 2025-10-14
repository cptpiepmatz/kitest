use std::{
    borrow::Cow, collections::{BTreeMap, HashMap}, fmt::Display, hash::Hash, io, num::NonZeroUsize, path::Path,
    thread, time::Duration,
};

type FlexStr<'s> = Cow<'s, str>;

pub struct TestMeta<'m, Extra> {
    pub name: FlexStr<'m>,
    pub function: fn(),
    pub ignored: (bool, Option<FlexStr<'m>>),
    pub should_panic: (bool, Option<FlexStr<'m>>),
    pub extra: Extra,
}

struct TestIndex<'m, TestExtra>(HashMap<&'m str, &'m TestMeta<'m, TestExtra>>);

impl<'m, TestExtra> FromIterator<&'m TestMeta<'m, TestExtra>> for TestIndex<'m, TestExtra> {
    fn from_iter<T: IntoIterator<Item = &'m TestMeta<'m, TestExtra>>>(iter: T) -> Self {
        Self(
            iter.into_iter()
                .map(|meta| (meta.name.as_ref(), meta))
                .collect(),
        )
    }
}

struct TestGroups<'m, TestExtra, GroupKey, GroupCtx>(
    HashMap<GroupKey, (GroupCtx, Vec<&'m TestMeta<'m, TestExtra>>)>,
);

pub struct TestExecutor<'m, TestExtra, GroupByFn> {
    include_ignored: bool,
    ignored: bool,
    list: bool,
    exact: bool,
    quiet: bool,
    test_threads: NonZeroUsize,
    log_file: Option<Cow<'static, Path>>,
    skip: Vec<String>,
    filter: Vec<String>,

    index: TestIndex<'m, TestExtra>,
    group_by_fn: Option<GroupByFn>,
}

impl<'m, TestExtra, GroupByFn> TestExecutor<'m, TestExtra, GroupByFn> {
    pub fn new(tests: impl IntoIterator<Item = &'m TestMeta<'m, TestExtra>>) -> Self {
        Self {
            include_ignored: false,
            ignored: false,
            list: false,
            exact: false,
            quiet: false,
            test_threads: thread::available_parallelism().unwrap_or(NonZeroUsize::MIN),
            log_file: None,
            skip: Vec::new(),
            filter: Vec::new(),
            index: TestIndex::from_iter(tests),
            group_by_fn: None,
        }
    }

    pub fn include_ignored(mut self, value: bool) -> Self {
        self.include_ignored = value;
        self
    }

    pub fn ignored(mut self, value: bool) -> Self {
        self.ignored = value;
        self
    }

    pub fn list(mut self, value: bool) -> Self {
        self.list = value;
        self
    }

    pub fn exact(mut self, value: bool) -> Self {
        self.exact = value;
        self
    }

    pub fn quiet(mut self, value: bool) -> Self {
        self.quiet = value;
        self
    }

    pub fn test_threads(mut self, value: NonZeroUsize) -> Self {
        self.test_threads = value;
        self
    }

    pub fn log_file(mut self, value: impl Into<Option<Cow<'static, Path>>>) -> Self {
        self.log_file = value.into();
        self
    }

    pub fn skip(mut self, value: impl Into<Vec<String>>) -> Self {
        self.skip = value.into();
        self
    }

    pub fn filter(mut self, value: impl Into<Vec<String>>) -> Self {
        self.filter = value.into();
        self
    }

    pub fn group_by<F, GroupKey, GroupCtx>(self, f: F) -> TestExecutor<'m, TestExtra, F>
    where
        F: Fn(&TestMeta<'m, TestExtra>) -> (GroupKey, GroupCtx),
        GroupKey: Hash + Eq + Ord + Display,
    {
        TestExecutor {
            include_ignored: self.include_ignored,
            ignored: self.ignored,
            list: self.list,
            exact: self.exact,
            quiet: self.quiet,
            test_threads: self.test_threads,
            log_file: self.log_file,
            skip: self.skip,
            filter: self.filter,
            index: self.index,
            group_by_fn: Some(f),
        }
    }

    pub fn run<F, GroupKey, GroupCtx>(self, f: F) -> Conclusion<GroupKey>
    where
        F: Fn(&TestMeta<'m, TestExtra>, GroupCtx, GroupKey) -> Result<Pass, Fail>,
    {
        todo!()
    }
}

pub enum Pass {
    Ok,
    Ignored,
}

pub struct Fail;

pub struct Conclusion<GroupKey>(BTreeMap<GroupKey, ConclusionGroup>);

pub struct ConclusionGroup {
    pub filtered_out: u64,
    pub passed: u64,
    pub failed: u64,
    pub ignored: u64,
    pub duration: Duration,
}

impl<GroupKey> Conclusion<GroupKey> {
    pub fn exit(self) -> ! {
        todo!()
    }
}

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

pub trait ResultFormatter<GroupKey: Eq + Hash + Display> {
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
        conclusion: &Conclusion<GroupKey>,
    ) -> io::Result<()>;
}

