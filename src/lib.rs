use std::{
    collections::HashMap,
    hash::Hash,
    io,
    sync::Arc,
    time::{Duration, Instant},
};

use crate::{
    filter::{FilterDecision, TestFilter},
    formatter::{FmtTestData, TestFormatter},
    group::{TestGroupRunner, TestGrouper, TestGroups},
    ignore::TestIgnore,
    meta::TestMeta,
    outcome::{TestOutcome, TestStatus},
    panic_handler::TestPanicHandler,
    runner::TestRunner,
};
use itertools::Itertools;

pub mod filter;
pub mod formatter;
pub mod group;
pub mod ignore;
pub mod meta;
pub mod outcome;
pub mod panic_handler;
pub mod runner;

fn apply_filter<'m, Iter, Filter, Extra>(
    tests: Iter,
    mut filter: Filter,
) -> (Vec<&'m TestMeta<Extra>>, usize)
where
    Iter: Iterator<Item = &'m TestMeta<Extra>>,
    Filter: TestFilter<Extra>,
{
    let mut filtered = 0usize;
    let mut kept = Vec::new();

    let mut iter = tests.into_iter();

    if filter.skip_filtering() {
        kept = iter.collect_vec();
        return (kept, filtered);
    }

    while let Some(meta) = iter.next() {
        match filter.filter(&meta) {
            FilterDecision::Keep => {
                kept.push(meta);
            }
            FilterDecision::Exclude => {
                filtered += 1;
            }
            FilterDecision::KeepAndDone => {
                kept.push(meta);
                filtered += iter.count(); // everything remaining is filtered out
                break;
            }
            FilterDecision::ExcludeAndDone => {
                filtered += 1 + iter.count(); // this one plus the rest
                break;
            }
        }
    }

    (kept, filtered)
}

pub fn run_tests<
    'm,
    Iter: Iterator<Item = &'m TestMeta<Extra>>,
    Filter: TestFilter<Extra>,
    Runner: TestRunner<Extra>,
    Ignore: TestIgnore<Extra> + Send + Sync + 'm,
    PanicHandler: TestPanicHandler<Extra> + Send + Sync + 'm,
    Formatter: TestFormatter<Extra>,
    Extra: 'm + Sync,
>(
    tests: Iter,
    filter: Filter,
    runner: Runner,
    ignore: Ignore,
    panic_handler: PanicHandler,
    mut formatter: Formatter,
) -> TestReport<'m> {
    let mut fmt_errors = Vec::new();
    macro_rules! try_fmt {
        ($fmt:expr) => {
            if let Err(err) = $fmt {
                fmt_errors.push((stringify!($fmt), err));
            }
        };
    }

    let now = Instant::now();

    try_fmt!(formatter.fmt_run_init());
    let (tests, filtered) = apply_filter(tests, filter);
    try_fmt!(formatter.fmt_run_start(&tests, filtered));

    let ignore = Arc::new(ignore);
    let panic_handler = Arc::new(panic_handler);

    let outcomes = std::thread::scope(move |scope| {
        let (ftx, frx) = crossbeam_channel::unbounded();
        scope.spawn(move || while let Ok(fmt_data) = frx.recv() {});

        let test_runs = tests.into_iter().map(|meta| {
            let ignore = Arc::clone(&ignore);
            let panic_handler = Arc::clone(&panic_handler);
            let ftx = ftx.clone();

            (
                move || {
                    let (ignored, reason) = ignore.ignore(meta);
                    if ignored {
                        let _ = ftx.send(FmtTestData::Ignored {
                            meta,
                            reason: reason.clone(),
                        });
                        return TestStatus::Ignored { reason };
                    }

                    let _ = ftx.send(FmtTestData::Start { meta });
                    let test_status = panic_handler.handle(meta);
                    test_status
                },
                meta,
            )
        });

        runner
            .run(test_runs, scope)
            .inspect(|(name, outcome)| {
                // let _ = ftx.send(FmtTestData::Outcome { name, outcome });
            })
            .map(|(meta, outcome)| (meta.name.as_ref(), outcome))
            .collect()
    });

    println!("got report");
    let duration = now.elapsed();
    try_fmt!(formatter.fmt_run_outcomes(&outcomes, duration));

    TestReport {
        outcomes,
        duration,
        fmt_errors,
    }
}

fn apply_grouped_filter<
    'm,
    Iter: Iterator<Item = &'m TestMeta<Extra>>,
    Filter: TestFilter<Extra>,
    Grouper: TestGrouper<GroupKey, Extra>,
    Groups: TestGroups<'m, GroupKey, Extra>,
    GroupKey,
    Extra: 'm,
>(
    mut tests: Iter,
    mut filter: Filter,
    grouper: Grouper,
    mut groups: Groups,
) -> (Groups, usize) {
    let mut filtered = 0;

    if filter.skip_filtering() {
        tests.for_each(|meta| groups.add(grouper.group(meta), meta));
        return (groups, filtered);
    }

    while let Some(meta) = tests.next() {
        match filter.filter(meta) {
            FilterDecision::Keep => groups.add(grouper.group(meta), meta),
            FilterDecision::Exclude => filtered += 1,
            FilterDecision::KeepAndDone => {
                groups.add(grouper.group(meta), meta);
                filtered += tests.count();
                return (groups, filtered);
            }
            FilterDecision::ExcludeAndDone => {
                filtered += 1 + tests.count();
                return (groups, filtered);
            }
        }
    }

    (groups, filtered)
}

pub fn run_grouped_tests<
    'm,
    Iter: Iterator<Item = &'m TestMeta<Extra>>,
    Filter: TestFilter<Extra>,
    Grouper: TestGrouper<GroupKey, Extra>,
    Groups: TestGroups<'m, GroupKey, Extra>,
    GroupRunner: TestGroupRunner<GroupKey, Extra>,
    Runner: TestRunner<Extra>,
    Ignore: TestIgnore<Extra> + Send + Sync + 'm,
    PanicHandler: TestPanicHandler<Extra> + Send + Sync + 'm,
    GroupKey: Eq + Hash,
    Extra: 'm + Sync,
>(
    tests: Iter,
    filter: Filter,
    grouper: Grouper,
    groups: Groups,
    group_runner: GroupRunner,
    runner: Runner,
    ignore: Ignore,
    panic_handler: PanicHandler,
) -> GroupedTestReport<'m, GroupKey> {
    let mut fmt_errors = Vec::new();

    let now = Instant::now();

    let (groups, filtered) = apply_grouped_filter(tests, filter, grouper, groups);

    // ftm_grouped_start(&groups: impl Groups, filtered: usize)

    let outcomes = std::thread::scope(move |scope| {
        let ignore = Arc::new(ignore);
        let panic_handler = Arc::new(panic_handler);
        let runner = Arc::new(runner);

        let group_runs = groups.into_iter().map(|(key, tests)| {
            let ignore = Arc::clone(&ignore);
            let panic_handler = Arc::clone(&panic_handler);
            let runner = Arc::clone(&runner);

            let report = group_runner.run_group(&key, move || {
                let test_runs = tests.into_iter().map(|meta| {
                    let ignore = Arc::clone(&ignore);
                    let panic_handler = Arc::clone(&panic_handler);

                    (
                        move || {
                            let (ignored, reason) = ignore.ignore(meta);
                            if ignored {
                                // fmt_ignored(meta: &TestMeta, reason: &str)
                                return TestStatus::Ignored { reason };
                            };

                            // fmt_start_test(meta: &TestMeta)
                            let test_status = panic_handler.handle(meta);
                            // fmt_test_result(meta: &TestMeta, result: &TestResult)

                            test_status
                        },
                        meta,
                    )
                });

                runner
                    .run(test_runs, scope)
                    .map(|(meta, outcome)| (meta.name.as_ref(), outcome))
                    .collect()
            });

            (key, report)
        });

        group_runs.collect()
    });

    let duration = now.elapsed();
    // fmt_grouped_report()

    GroupedTestReport {
        outcomes,
        duration,
        fmt_errors,
    }
}

pub type TestOutcomes<'m> = HashMap<&'m str, TestOutcome, ahash::RandomState>;
pub struct TestReport<'m> {
    outcomes: TestOutcomes<'m>,
    duration: Duration,
    fmt_errors: Vec<(&'static str, io::Error)>,
}

pub type GroupedTestOutcomes<'m, GroupKey> =
    HashMap<GroupKey, HashMap<&'m str, TestOutcome, ahash::RandomState>, ahash::RandomState>;
pub struct GroupedTestReport<'m, GroupKey> {
    outcomes: GroupedTestOutcomes<'m, GroupKey>,
    duration: Duration,
    fmt_errors: Vec<(&'static str, io::Error)>,
}

#[test]
fn foo() {}

#[test]
fn bar() {}
