use std::{
    borrow::Cow,
    collections::{BTreeMap, HashMap},
    fmt::Display,
    hash::Hash,
    io,
    num::NonZeroUsize,
    path::Path,
    process::{self, ExitCode, Termination},
    thread,
    time::Duration,
};

use itertools::Itertools;

mod formatter;

type FlexStr<'s> = Cow<'s, str>;
pub type Result<T = Pass, E = Fail> = std::result::Result<T, E>;

pub enum TestFn {
    Plain(fn()),
    WithResult(fn() -> Result),
}

impl TestFn {
    pub const fn plain(f: fn()) -> Self {
        Self::Plain(f)
    }

    pub const fn with_result(f: fn() -> Result) -> Self {
        Self::WithResult(f)
    }
}

pub struct TestMeta<'m, Extra = ()> {
    pub name: FlexStr<'m>,
    pub function: TestFn,
    pub ignored: (bool, Option<FlexStr<'m>>),
    pub should_panic: (bool, Option<FlexStr<'m>>),
    pub extra: Extra,
}

impl<'m, Extra> TestMeta<'m, Extra> {
    pub fn run(&self) -> Result {
        match self.function {
            TestFn::Plain(f) => f(),
            TestFn::WithResult(f) => return f(),
        }

        Ok(Pass::Ok)
    }
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

impl<'m, TestExtra> TestExecutor<'m, TestExtra, fn(&TestMeta<'m, TestExtra>) -> ((), ())> {
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
}

impl<'m, TestExtra, GroupByFn> TestExecutor<'m, TestExtra, GroupByFn> {
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

    pub fn run<F>(self, f: F) -> Conclusion
    where
        F: Fn(&TestMeta<'m, TestExtra>) -> Result<Pass, Fail>,
    {
        #[derive(Default)]
        struct Fold {
            passed: u64,
            failed: u64,
            ignored: u64,
        }

        let results =
            self.index
                .0
                .values()
                .map(|meta| f(meta))
                .fold(Fold::default(), |mut acc, current| {
                    match current {
                        Ok(Pass::Ok) => acc.passed += 1,
                        Ok(Pass::Ignored) => acc.ignored += 1,
                        Err(_) => acc.failed += 1,
                    };

                    acc
                });

        Conclusion {
            filtered_out: 0,
            passed: results.passed,
            failed: results.failed,
            ignored: results.ignored,
            duration: Duration::default(),
        }
    }

    pub fn run_grouped<F, GroupKey, GroupCtx>(self, f: F) -> ConclusionGroups<GroupKey>
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

pub struct Conclusion {
    pub filtered_out: u64,
    pub passed: u64,
    pub failed: u64,
    pub ignored: u64,
    pub duration: Duration,
}

impl Conclusion {
    pub fn exit_code(self) -> impl Termination {
        match self.failed {
            0 => ExitCode::SUCCESS,
            _ => ExitCode::FAILURE,
        }
    }

    pub fn exit(self) -> ! {
        match self.failed {
            0 => process::exit(0),
            _ => process::exit(1),
        }
    }
}

pub struct ConclusionGroups<GroupKey>(BTreeMap<GroupKey, Conclusion>);

impl<GroupKey> ConclusionGroups<GroupKey> {
    pub fn exit(self) -> ! {
        match self.exit_code().report() {
            ExitCode::SUCCESS => process::exit(0),
            _ => process::exit(1),
        }
    }

    pub fn exit_code(self) -> impl Termination {
        for (_, conclusion) in self.0.into_iter() {
            if let exit_code @ ExitCode::FAILURE = conclusion.exit_code().report() {
                return exit_code;
            }
        }

        ExitCode::SUCCESS
    }

    pub fn filtered_out(&self) -> u64 {
        self.0.values().map(|g| g.filtered_out).sum()
    }

    pub fn passed(&self) -> u64 {
        self.0.values().map(|g| g.passed).sum()
    }

    pub fn failed(&self) -> u64 {
        self.0.values().map(|g| g.failed).sum()
    }

    pub fn ignored(&self) -> u64 {
        self.0.values().map(|g| g.ignored).sum()
    }

    pub fn duration(&self) -> Duration {
        self.0.values().map(|g| g.duration).sum()
    }
}
