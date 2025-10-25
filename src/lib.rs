use std::{
    collections::HashMap,
    hash::Hash,
    sync::Arc,
    time::{Duration, Instant},
};

use crate::{
    filter::{FilteredTests, TestFilter},
    formatter::{
        FmtBeginListing, FmtEndListing, FmtGroupOutcomes, FmtGroupStart, FmtGroupedRunOutcomes,
        FmtGroupedRunStart, FmtGroupedTestData, FmtInitListing, FmtListTest, FmtRunInitData,
        FmtRunOutcomes, FmtRunStartData, FmtTestData, FmtTestIgnored, FmtTestOutcome, FmtTestStart,
        GroupedTestFormatter, TestFormatter, TestListFormatter,
    },
    group::{TestGroupRunner, TestGrouper, TestGroups},
    ignore::TestIgnore,
    meta::TestMeta,
    outcome::{TestOutcome, TestStatus},
    panic_handler::TestPanicHandler,
    runner::TestRunner,
};

pub mod filter;
pub mod formatter;
pub mod group;
pub mod ignore;
pub mod meta;
pub mod outcome;
pub mod panic_handler;
pub mod runner;

trait FmtErrors<E> {
    fn push_on_error<T>(&mut self, data: (&'static str, Result<T, E>));
}

impl<E> FmtErrors<E> for Vec<(&'static str, E)> {
    fn push_on_error<T>(&mut self, (name, res): (&'static str, Result<T, E>)) {
        if let Err(err) = res {
            self.push((name, err));
        }
    }
}

macro_rules! named_fmt {
    ($fmt:ident.$method:ident($expr:expr)) => {
        (stringify!($method), $fmt.$method($expr))
    };
}

pub fn run_tests<
    'm,
    Filter: TestFilter<Extra>,
    Runner: TestRunner<Extra>,
    Ignore: TestIgnore<Extra> + Send + Sync + 'm,
    PanicHandler: TestPanicHandler<Extra> + Send + Sync + 'm,
    Formatter: TestFormatter<'m, Extra> + 'm,
    Extra: Sync + 'm,
>(
    tests: &'m [TestMeta<Extra>],
    filter: Filter,
    runner: Runner,
    ignore: Ignore,
    panic_handler: PanicHandler,
    mut formatter: Formatter,
) -> TestReport<'m, Formatter::Error> {
    let now = Instant::now();

    let mut fmt_errors = Vec::new();
    fmt_errors.push_on_error(named_fmt!(
        formatter.fmt_run_init(FmtRunInitData { tests }.into())
    ));

    let FilteredTests { tests, filtered } = filter.filter(tests);
    fmt_errors.push_on_error(named_fmt!(
        formatter.fmt_run_start(
            FmtRunStartData {
                tests: tests.len(),
                filtered
            }
            .into()
        )
    ));

    let ignore = Arc::new(ignore);
    let panic_handler = Arc::new(panic_handler);

    let (outcomes, mut formatter, mut fmt_errors) = std::thread::scope(move |scope| {
        let (ftx, frx) = crossbeam_channel::bounded(4 * runner.worker_count(tests.len()).get());
        let fmt_thread = scope.spawn(move || {
            while let Ok(fmt_data) = frx.recv() {
                fmt_errors.push_on_error(match fmt_data {
                    FmtTestData::Ignored(data) => named_fmt!(formatter.fmt_test_ignored(data)),
                    FmtTestData::Start(data) => named_fmt!(formatter.fmt_test_start(data)),
                    FmtTestData::Outcome(data) => named_fmt!(formatter.fmt_test_outcome(data)),
                });
            }
            (formatter, fmt_errors)
        });

        let test_runs = tests.into_iter().map(|meta| {
            let ignore = Arc::clone(&ignore);
            let panic_handler = Arc::clone(&panic_handler);
            let ftx = ftx.clone();

            (
                move || {
                    let (ignored, reason) = ignore.ignore(meta);
                    if ignored {
                        let _ = ftx.send(FmtTestData::Ignored(
                            FmtTestIgnored {
                                meta,
                                reason: reason.as_ref(),
                            }
                            .into(),
                        ));
                        return TestStatus::Ignored { reason };
                    }

                    let _ = ftx.send(FmtTestData::Start(FmtTestStart { meta }.into()));
                    panic_handler.handle(meta)
                },
                meta,
            )
        });

        let outcomes = runner
            .run(test_runs, scope)
            .inspect(|(meta, outcome)| {
                let _ = ftx.send(FmtTestData::Outcome(
                    FmtTestOutcome {
                        meta: *meta,
                        outcome,
                    }
                    .into(),
                ));
            })
            .map(|(meta, outcome)| (meta.name.as_ref(), outcome))
            .collect();

        drop(ftx);
        let (formatter, fmt_errors) = fmt_thread
            .join()
            .expect("format thread should join without issues");

        (outcomes, formatter, fmt_errors)
    });

    let duration = now.elapsed();
    fmt_errors.push_on_error(named_fmt!(
        formatter.fmt_run_outcomes(
            FmtRunOutcomes {
                outcomes: &outcomes,
                duration
            }
            .into()
        )
    ));

    TestReport {
        outcomes,
        duration,
        fmt_errors,
    }
}

#[allow(clippy::too_many_arguments)]
pub fn run_grouped_tests<
    'm,
    Filter: TestFilter<Extra>,
    Grouper: TestGrouper<GroupKey, Extra>,
    Groups: TestGroups<'m, GroupKey, Extra>,
    GroupRunner: TestGroupRunner<GroupKey, Extra>,
    Runner: TestRunner<Extra>,
    Ignore: TestIgnore<Extra> + Send + Sync + 'm,
    PanicHandler: TestPanicHandler<Extra> + Send + Sync + 'm,
    Formatter: GroupedTestFormatter<'m, GroupKey, Extra> + 'm,
    GroupKey: Eq + Hash + 'm,
    Extra: Sync + 'm,
>(
    tests: &'m [TestMeta<Extra>],
    filter: Filter,
    grouper: Grouper,
    mut groups: Groups,
    group_runner: GroupRunner,
    runner: Runner,
    ignore: Ignore,
    panic_handler: PanicHandler,
    mut formatter: Formatter,
) -> GroupedTestReport<'m, GroupKey, Formatter::Error>
where
    <Formatter as GroupedTestFormatter<'m, GroupKey, Extra>>::GroupStart: 'm,
    <Formatter as GroupedTestFormatter<'m, GroupKey, Extra>>::GroupOutcomes: 'm,
{
    let now = Instant::now();

    let mut fmt_errors = Vec::new();
    fmt_errors.push_on_error(named_fmt!(
        formatter.fmt_run_init(FmtRunInitData { tests }.into())
    ));

    let FilteredTests { tests, filtered } = filter.filter(tests);
    tests.for_each(|meta| groups.add(grouper.group(meta), meta));

    fmt_errors.push_on_error(named_fmt!(
        formatter.fmt_grouped_run_start(
            FmtGroupedRunStart {
                tests: groups.len(),
                filtered
            }
            .into()
        )
    ));
    let (grouped_outcomes, mut formatter, mut fmt_errors) = std::thread::scope(move |scope| {
        // TODO: prefer getting only the MAX value and not the total count of tests for the worker_count estimation
        let (ftx, frx) = crossbeam_channel::bounded(runner.worker_count(groups.len()).get());
        let fmt_thread = scope.spawn(move || {
            while let Ok(fmt_data) = frx.recv() {
                fmt_errors.push_on_error(match fmt_data {
                    FmtGroupedTestData::Start(data) => named_fmt!(formatter.fmt_group_start(data)),
                    FmtGroupedTestData::Test(data) => match data {
                        FmtTestData::Ignored(data) => named_fmt!(formatter.fmt_test_ignored(data)),
                        FmtTestData::Start(data) => named_fmt!(formatter.fmt_test_start(data)),
                        FmtTestData::Outcome(data) => named_fmt!(formatter.fmt_test_outcome(data)),
                    },
                    FmtGroupedTestData::Outcome(data) => {
                        named_fmt!(formatter.fmt_group_outcomes(data))
                    }
                });
            }
            (formatter, fmt_errors)
        });

        let ignore = Arc::new(ignore);
        let panic_handler = Arc::new(panic_handler);
        let runner = Arc::new(runner);

        let group_runs = groups.into_iter().map(|(key, tests)| {
            let now = Instant::now();

            let ignore = Arc::clone(&ignore);
            let panic_handler = Arc::clone(&panic_handler);
            let runner = Arc::clone(&runner);
            let ftx = ftx.clone();

            let _ = ftx.send(FmtGroupedTestData::Start(
                FmtGroupStart {
                    key: &key,
                    tests: tests.len(),
                }
                .into(),
            ));

            let outcomes = group_runner.run_group(&key, move || {
                let test_runs = tests.into_iter().map(|meta| {
                    let ignore = Arc::clone(&ignore);
                    let panic_handler = Arc::clone(&panic_handler);
                    let ftx = ftx.clone();

                    (
                        move || {
                            let (ignored, reason) = ignore.ignore(meta);
                            if ignored {
                                let _ = ftx.send(FmtGroupedTestData::Test(FmtTestData::Ignored(
                                    FmtTestIgnored {
                                        meta,
                                        reason: reason.as_ref(),
                                    }
                                    .into(),
                                )));
                                return TestStatus::Ignored { reason };
                            };

                            let _ = ftx.send(FmtGroupedTestData::Test(FmtTestData::Start(
                                FmtTestStart { meta }.into(),
                            )));
                            panic_handler.handle(meta)
                        },
                        meta,
                    )
                });

                runner
                    .run(test_runs, scope)
                    .inspect(|(meta, outcome)| {
                        let _ = ftx.send(FmtGroupedTestData::Test(FmtTestData::Outcome(
                            FmtTestOutcome {
                                meta: *meta,
                                outcome,
                            }
                            .into(),
                        )));
                    })
                    .map(|(meta, outcome)| (meta.name.as_ref(), outcome))
                    .collect()
            });

            (key, outcomes, now.elapsed())
        });

        let grouped_outcomes = group_runs
            .inspect(|(key, outcomes, duration)| {
                let _ = ftx.send(FmtGroupedTestData::Outcome(
                    FmtGroupOutcomes {
                        key,
                        outcomes,
                        duration: *duration,
                    }
                    .into(),
                ));
            })
            .map(|(key, outcomes, _)| (key, outcomes))
            .collect();

        drop(ftx);
        let (formatter, fmt_errors) = fmt_thread
            .join()
            .expect("format thread should join without issues");

        (grouped_outcomes, formatter, fmt_errors)
    });

    let duration = now.elapsed();
    fmt_errors.push_on_error(named_fmt!(
        formatter.fmt_grouped_run_outcomes(
            FmtGroupedRunOutcomes {
                outcomes: &grouped_outcomes,
                duration
            }
            .into()
        )
    ));

    GroupedTestReport {
        outcomes: grouped_outcomes,
        duration,
        fmt_errors,
    }
}

pub type TestOutcomes<'m> = HashMap<&'m str, TestOutcome, ahash::RandomState>;

#[non_exhaustive]
pub struct TestReport<'m, FmtError: 'm> {
    pub outcomes: TestOutcomes<'m>,
    pub duration: Duration,
    pub fmt_errors: Vec<(&'static str, FmtError)>,
}

pub type GroupedTestOutcomes<'m, GroupKey> =
    HashMap<GroupKey, HashMap<&'m str, TestOutcome, ahash::RandomState>, ahash::RandomState>;

#[non_exhaustive]
pub struct GroupedTestReport<'m, GroupKey, FmtError: 'm> {
    pub outcomes: GroupedTestOutcomes<'m, GroupKey>,
    pub duration: Duration,
    pub fmt_errors: Vec<(&'static str, FmtError)>,
}

pub fn list_tests<
    'm,
    Filter: TestFilter<Extra>,
    Ignore: TestIgnore<Extra>,
    Formatter: TestListFormatter<'m, Extra>,
    Extra: Sync + 'm,
>(
    tests: &'m [TestMeta<Extra>],
    filter: Filter,
    ignore: Ignore,
    mut formatter: Formatter,
) -> Vec<(&'static str, Formatter::Error)> {
    let mut fmt_errors = Vec::new();
    fmt_errors.push_on_error(named_fmt!(
        formatter.fmt_init_listing(FmtInitListing { tests }.into())
    ));

    let FilteredTests { tests, filtered } = filter.filter(tests);
    fmt_errors.push_on_error(named_fmt!(
        formatter.fmt_begin_listing(
            FmtBeginListing {
                tests: tests.len(),
                filtered
            }
            .into()
        )
    ));

    let mut active_count = 0;
    let mut ignore_count = 0;
    for meta in tests {
        let ignored = ignore.ignore(meta);
        match ignored.0 {
            true => ignore_count += 1,
            false => active_count += 1,
        }
        fmt_errors.push_on_error(named_fmt!(
            formatter.fmt_list_test(FmtListTest { meta, ignored }.into())
        ));
    }

    fmt_errors.push_on_error(named_fmt!(
        formatter.fmt_end_listing(
            FmtEndListing {
                active: active_count,
                ignored: ignore_count
            }
            .into()
        )
    ));

    fmt_errors
}

#[test]
fn foo() {}

#[test]
fn bar() {}

#[test]
#[ignore = "for a reason"]
fn ignored() {}
