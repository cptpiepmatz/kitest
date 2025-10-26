use std::{
    collections::HashMap,
    hash::Hash,
    io,
    marker::PhantomData,
    panic::RefUnwindSafe,
    sync::Arc,
    time::{Duration, Instant},
};

use crate::{
    filter::{DefaultFilter, FilteredTests, TestFilter},
    formatter::{
        FmtBeginListing, FmtEndListing, FmtGroupOutcomes, FmtGroupStart, FmtGroupedRunOutcomes,
        FmtGroupedRunStart, FmtGroupedTestData, FmtInitListing, FmtListGroupEnd, FmtListGroupStart,
        FmtListGroups, FmtListTest, FmtRunInitData, FmtRunOutcomes, FmtRunStart, FmtTestData,
        FmtTestIgnored, FmtTestOutcome, FmtTestStart, GroupedTestFormatter,
        GroupedTestListFormatter, TestFormatter, TestListFormatter, pretty::PrettyFormatter,
    },
    group::{SimpleGroupRunner, TestGroupHashMap, TestGroupRunner, TestGrouper, TestGroups},
    ignore::{DefaultIgnore, TestIgnore},
    outcome::{TestOutcome, TestStatus},
    panic_handler::{DefaultPanicHandler, TestPanicHandler},
    runner::{DefaultRunner, TestRunner},
    test::Test,
};

pub mod filter;
pub mod formatter;
pub mod group;
pub mod ignore;
pub mod outcome;
pub mod panic_handler;
pub mod runner;
pub mod test;

mod harness;
pub use harness::*;

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
    't,
    Filter: TestFilter<Extra>,
    Runner: TestRunner<Extra>,
    Ignore: TestIgnore<Extra> + Send + Sync + 't,
    PanicHandler: TestPanicHandler<Extra> + Send + Sync + 't,
    Formatter: TestFormatter<'t, Extra> + 't,
    Extra: RefUnwindSafe + Sync + 't,
>(
    tests: &'t [Test<Extra>],
    filter: Filter,
    runner: Runner,
    ignore: Ignore,
    panic_handler: PanicHandler,
    mut formatter: Formatter,
) -> TestReport<'t, Formatter::Error> {
    let now = Instant::now();

    let mut fmt_errors = Vec::new();
    fmt_errors.push_on_error(named_fmt!(
        formatter.fmt_run_init(FmtRunInitData { tests }.into())
    ));

    let FilteredTests { tests, filtered } = filter.filter(tests);
    fmt_errors.push_on_error(named_fmt!(
        formatter.fmt_run_start(
            FmtRunStart {
                active: tests.len(),
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

        let test_runs = tests.into_iter().map(|test| {
            let meta = &test.meta;
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
                    panic_handler.handle(|| test.call(), meta)
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
                filtered_out: filtered,
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
    't,
    Filter: TestFilter<Extra>,
    Grouper: TestGrouper<Extra, GroupKey, GroupCtx>,
    Groups: TestGroups<'t, Extra, GroupKey>,
    GroupRunner: TestGroupRunner<Extra, GroupKey, GroupCtx>,
    Runner: TestRunner<Extra>,
    Ignore: TestIgnore<Extra> + Send + Sync + 't,
    PanicHandler: TestPanicHandler<Extra> + Send + Sync + 't,
    Formatter: GroupedTestFormatter<'t, Extra, GroupKey, GroupCtx> + 't,
    Extra: RefUnwindSafe + Sync + 't,
    GroupKey: Eq + Hash + 't,
    GroupCtx: 't,
>(
    tests: &'t [Test<Extra>],
    filter: Filter,
    mut grouper: Grouper,
    mut groups: Groups,
    group_runner: GroupRunner,
    runner: Runner,
    ignore: Ignore,
    panic_handler: PanicHandler,
    mut formatter: Formatter,
) -> GroupedTestReport<'t, GroupKey, Formatter::Error>
where
    <Formatter as GroupedTestFormatter<'t, Extra, GroupKey, GroupCtx>>::GroupStart: 't,
    <Formatter as GroupedTestFormatter<'t, Extra, GroupKey, GroupCtx>>::GroupOutcomes: 't,
{
    let now = Instant::now();

    let mut fmt_errors = Vec::new();
    fmt_errors.push_on_error(named_fmt!(
        formatter.fmt_run_init(FmtRunInitData { tests }.into())
    ));

    let FilteredTests { tests, filtered } = filter.filter(tests);
    tests.for_each(|test| groups.add(grouper.group(test), test));

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

        let group_runs = groups.into_groups().map(|(key, tests)| {
            let now = Instant::now();

            let ignore = Arc::clone(&ignore);
            let panic_handler = Arc::clone(&panic_handler);
            let runner = Arc::clone(&runner);
            let ftx = ftx.clone();
            let ctx = grouper.group_ctx(&key);

            let _ = ftx.send(FmtGroupedTestData::Start(
                FmtGroupStart {
                    tests: tests.len(),
                    key: &key,
                    ctx,
                }
                .into(),
            ));

            let outcomes = group_runner.run_group(
                move || {
                    let test_runs = tests.into_iter().map(|test| {
                        let meta = &test.meta;
                        let ignore = Arc::clone(&ignore);
                        let panic_handler = Arc::clone(&panic_handler);
                        let ftx = ftx.clone();

                        (
                            move || {
                                let (ignored, reason) = ignore.ignore(meta);
                                if ignored {
                                    let _ =
                                        ftx.send(FmtGroupedTestData::Test(FmtTestData::Ignored(
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
                                panic_handler.handle(|| test.call(), meta)
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
                },
                &key,
                ctx,
            );

            (outcomes, now.elapsed(), key, ctx)
        });

        let grouped_outcomes = group_runs
            .inspect(|(outcomes, duration, key, ctx)| {
                let _ = ftx.send(FmtGroupedTestData::Outcome(
                    FmtGroupOutcomes {
                        outcomes,
                        duration: *duration,
                        key,
                        ctx: *ctx,
                    }
                    .into(),
                ));
            })
            .map(|(outcomes, _, key, _)| (key, outcomes))
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

pub type TestOutcomes<'t> = HashMap<&'t str, TestOutcome, ahash::RandomState>;

#[non_exhaustive]
pub struct TestReport<'t, FmtError: 't> {
    pub outcomes: TestOutcomes<'t>,
    pub duration: Duration,
    pub fmt_errors: Vec<(&'static str, FmtError)>,
}

pub type GroupedTestOutcomes<'t, GroupKey> =
    HashMap<GroupKey, HashMap<&'t str, TestOutcome, ahash::RandomState>, ahash::RandomState>;

#[non_exhaustive]
pub struct GroupedTestReport<'t, GroupKey, FmtError: 't> {
    pub outcomes: GroupedTestOutcomes<'t, GroupKey>,
    pub duration: Duration,
    pub fmt_errors: Vec<(&'static str, FmtError)>,
}

pub fn list_tests<
    't,
    Filter: TestFilter<Extra>,
    Ignore: TestIgnore<Extra>,
    Formatter: TestListFormatter<'t, Extra>,
    Extra: Sync + 't,
>(
    tests: &'t [Test<Extra>],
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
    for test in tests {
        let ignored = ignore.ignore(test);
        match ignored.0 {
            true => ignore_count += 1,
            false => active_count += 1,
        }
        fmt_errors.push_on_error(named_fmt!(
            formatter.fmt_list_test(
                FmtListTest {
                    meta: test,
                    ignored
                }
                .into()
            )
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

pub fn list_grouped_tests<
    't,
    Filter: TestFilter<Extra>,
    Ignore: TestIgnore<Extra>,
    Grouper: TestGrouper<Extra, GroupKey, GroupCtx>,
    Groups: TestGroups<'t, Extra, GroupKey>,
    Formatter: GroupedTestListFormatter<'t, Extra, GroupKey, GroupCtx>,
    Extra: Sync + 't,
    GroupKey: 't,
    GroupCtx: 't,
>(
    tests: &'t [Test<Extra>],
    filter: Filter,
    ignore: Ignore,
    mut grouper: Grouper,
    mut groups: Groups,
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

    tests.for_each(|test| groups.add(grouper.group(test), test));
    let groups = groups.into_groups();
    fmt_errors.push_on_error(named_fmt!(
        formatter.fmt_list_groups(
            FmtListGroups {
                groups: groups.len()
            }
            .into()
        )
    ));

    let mut active_count = 0;
    let mut ignore_count = 0;
    for (key, tests) in groups {
        let ctx = grouper.group_ctx(&key);
        let tests_len = tests.len();

        fmt_errors.push_on_error(named_fmt!(
            formatter.fmt_list_group_start(
                FmtListGroupStart {
                    tests: tests_len,
                    key: &key,
                    ctx
                }
                .into()
            )
        ));

        for test in tests {
            let ignored = ignore.ignore(test);
            match ignored.0 {
                true => ignore_count += 1,
                false => active_count += 1,
            }
            fmt_errors.push_on_error(named_fmt!(
                formatter.fmt_list_test(
                    FmtListTest {
                        meta: test,
                        ignored
                    }
                    .into()
                )
            ));
        }

        fmt_errors.push_on_error(named_fmt!(
            formatter.fmt_list_group_end(
                FmtListGroupEnd {
                    tests: tests_len,
                    key: &key,
                    ctx
                }
                .into()
            )
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
