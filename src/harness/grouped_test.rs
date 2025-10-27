use std::{hash::Hash, marker::PhantomData, panic::RefUnwindSafe, sync::Arc, time::Instant};

use crate::{
    GroupedTestReport,
    filter::{FilteredTests, TestFilter},
    formatter::*,
    group::{TestGroupRunner, TestGrouper, TestGroups},
    ignore::TestIgnore,
    outcome::TestStatus,
    panic_handler::TestPanicHandler,
    runner::TestRunner,
    test::Test,
};

use super::{FmtErrors, named_fmt};

pub struct GroupedTestHarness<
    't,
    Extra,
    GroupKey,
    GroupCtx,
    Filter,
    Grouper,
    Groups,
    Ignore,
    GroupRunner,
    PanicHandler,
    Runner,
    Formatter,
> {
    pub(crate) tests: &'t [Test<Extra>],
    pub(crate) _group_key: PhantomData<GroupKey>,
    pub(crate) _group_ctx: PhantomData<GroupCtx>,
    pub(crate) filter: Filter,
    pub(crate) grouper: Grouper,
    pub(crate) groups: Groups,
    pub(crate) ignore: Ignore,
    pub(crate) group_runner: GroupRunner,
    pub(crate) panic_handler: PanicHandler,
    pub(crate) runner: Runner,
    pub(crate) formatter: Formatter,
}

impl<
    't,
    Extra: RefUnwindSafe + Sync,
    GroupKey: Hash + Eq + 't,
    GroupCtx: 't,
    Filter: TestFilter<Extra>,
    Grouper: TestGrouper<Extra, GroupKey, GroupCtx>,
    Groups: TestGroups<'t, Extra, GroupKey>,
    Ignore: TestIgnore<Extra> + Send + Sync + 't,
    GroupRunner: TestGroupRunner<Extra, GroupKey, GroupCtx>,
    PanicHandler: TestPanicHandler<Extra> + Send + Sync + 't,
    Runner: TestRunner<Extra>,
    Formatter: GroupedTestFormatter<'t, Extra, GroupKey, GroupCtx> + 't,
>
    GroupedTestHarness<
        't,
        Extra,
        GroupKey,
        GroupCtx,
        Filter,
        Grouper,
        Groups,
        Ignore,
        GroupRunner,
        PanicHandler,
        Runner,
        Formatter,
    >
{
    pub fn run(mut self) -> GroupedTestReport<'t, GroupKey, Formatter::Error> {
        let now = Instant::now();

        let mut formatter = self.formatter;
        let mut fmt_errors = Vec::new();
        fmt_errors.push_on_error(named_fmt!(
            formatter.fmt_run_init(FmtRunInitData { tests: self.tests }.into())
        ));

        let FilteredTests { tests, filtered_out: filtered } = self.filter.filter(self.tests);
        tests.for_each(|test| self.groups.add(self.grouper.group(test), test));

        fmt_errors.push_on_error(named_fmt!(
            formatter.fmt_grouped_run_start(
                FmtGroupedRunStart {
                    tests: self.groups.len(),
                    filtered
                }
                .into()
            )
        ));
        let (grouped_outcomes, mut formatter, mut fmt_errors) = std::thread::scope(move |scope| {
            // TODO: prefer getting only the MAX value and not the total count of tests for the worker_count estimation
            let (ftx, frx) =
                crossbeam_channel::bounded(self.runner.worker_count(self.groups.len()).get());
            let fmt_thread = scope.spawn(move || {
                while let Ok(fmt_data) = frx.recv() {
                    fmt_errors.push_on_error(match fmt_data {
                        FmtGroupedTestData::Start(data) => {
                            named_fmt!(formatter.fmt_group_start(data))
                        }
                        FmtGroupedTestData::Test(data) => match data {
                            FmtTestData::Ignored(data) => {
                                named_fmt!(formatter.fmt_test_ignored(data))
                            }
                            FmtTestData::Start(data) => named_fmt!(formatter.fmt_test_start(data)),
                            FmtTestData::Outcome(data) => {
                                named_fmt!(formatter.fmt_test_outcome(data))
                            }
                        },
                        FmtGroupedTestData::Outcome(data) => {
                            named_fmt!(formatter.fmt_group_outcomes(data))
                        }
                    });
                }
                (formatter, fmt_errors)
            });

            let ignore = Arc::new(self.ignore);
            let panic_handler = Arc::new(self.panic_handler);
            let runner = Arc::new(self.runner);

            let group_runs = self.groups.into_groups().map(|(key, tests)| {
                let now = Instant::now();

                let ignore = Arc::clone(&ignore);
                let panic_handler = Arc::clone(&panic_handler);
                let runner = Arc::clone(&runner);
                let ftx = ftx.clone();
                let ctx = self.grouper.group_ctx(&key);

                let _ = ftx.send(FmtGroupedTestData::Start(
                    FmtGroupStart {
                        tests: tests.len(),
                        key: &key,
                        ctx,
                    }
                    .into(),
                ));

                let outcomes = self.group_runner.run_group(
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
                                        let _ = ftx.send(FmtGroupedTestData::Test(
                                            FmtTestData::Ignored(
                                                FmtTestIgnored {
                                                    meta,
                                                    reason: reason.as_ref(),
                                                }
                                                .into(),
                                            ),
                                        ));
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
}

impl<
    't,
    Extra,
    GroupKey: 't,
    GroupCtx: 't,
    Filter: TestFilter<Extra>,
    Grouper: TestGrouper<Extra, GroupKey, GroupCtx>,
    Groups: TestGroups<'t, Extra, GroupKey>,
    Ignore: TestIgnore<Extra>,
    GroupRunner,
    PanicHandler,
    Runner,
    Formatter: GroupedTestListFormatter<'t, Extra, GroupKey, GroupCtx>,
>
    GroupedTestHarness<
        't,
        Extra,
        GroupKey,
        GroupCtx,
        Filter,
        Grouper,
        Groups,
        Ignore,
        GroupRunner,
        PanicHandler,
        Runner,
        Formatter,
    >
{
    pub fn list(mut self) -> Vec<(&'static str, Formatter::Error)> {
        let mut formatter = self.formatter;
        let mut fmt_errors = Vec::new();
        fmt_errors.push_on_error(named_fmt!(
            formatter.fmt_init_listing(FmtInitListing { tests: self.tests }.into())
        ));

        let FilteredTests { tests, filtered_out: filtered } = self.filter.filter(self.tests);
        fmt_errors.push_on_error(named_fmt!(
            formatter.fmt_begin_listing(
                FmtBeginListing {
                    tests: tests.len(),
                    filtered
                }
                .into()
            )
        ));

        tests.for_each(|test| self.groups.add(self.grouper.group(test), test));
        let groups = self.groups.into_groups();
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
            let ctx = self.grouper.group_ctx(&key);
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
                let ignored = self.ignore.ignore(test);
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
}

impl<
    't,
    Extra,
    GroupKey,
    GroupCtx,
    Filter,
    Grouper,
    Groups,
    Ignore,
    GroupRunner,
    PanicHandler,
    Runner,
    Formatter,
>
    GroupedTestHarness<
        't,
        Extra,
        GroupKey,
        GroupCtx,
        Filter,
        Grouper,
        Groups,
        Ignore,
        GroupRunner,
        PanicHandler,
        Runner,
        Formatter,
    >
{
    pub fn with_filter<WithFilter: TestFilter<Extra>>(
        self,
        filter: WithFilter,
    ) -> GroupedTestHarness<
        't,
        Extra,
        GroupKey,
        GroupCtx,
        WithFilter,
        Grouper,
        Groups,
        Ignore,
        GroupRunner,
        PanicHandler,
        Runner,
        Formatter,
    > {
        GroupedTestHarness {
            tests: self.tests,
            _group_key: PhantomData,
            _group_ctx: PhantomData,
            filter,
            grouper: self.grouper,
            groups: self.groups,
            ignore: self.ignore,
            group_runner: self.group_runner,
            panic_handler: self.panic_handler,
            runner: self.runner,
            formatter: self.formatter,
        }
    }

    pub fn with_groups<WithGroups: TestGroups<'t, Extra, GroupKey>>(
        self,
        groups: WithGroups,
    ) -> GroupedTestHarness<
        't,
        Extra,
        GroupKey,
        GroupCtx,
        Filter,
        Grouper,
        WithGroups,
        Ignore,
        GroupRunner,
        PanicHandler,
        Runner,
        Formatter,
    > {
        GroupedTestHarness {
            tests: self.tests,
            _group_key: PhantomData,
            _group_ctx: PhantomData,
            filter: self.filter,
            grouper: self.grouper,
            groups,
            ignore: self.ignore,
            group_runner: self.group_runner,
            panic_handler: self.panic_handler,
            runner: self.runner,
            formatter: self.formatter,
        }
    }

    pub fn with_ignore<WithIgnore: TestIgnore<Extra>>(
        self,
        ignore: WithIgnore,
    ) -> GroupedTestHarness<
        't,
        Extra,
        GroupKey,
        GroupCtx,
        Filter,
        Grouper,
        Groups,
        WithIgnore,
        GroupRunner,
        PanicHandler,
        Runner,
        Formatter,
    > {
        GroupedTestHarness {
            tests: self.tests,
            _group_key: PhantomData,
            _group_ctx: PhantomData,
            filter: self.filter,
            grouper: self.grouper,
            groups: self.groups,
            ignore,
            group_runner: self.group_runner,
            panic_handler: self.panic_handler,
            runner: self.runner,
            formatter: self.formatter,
        }
    }

    pub fn with_group_runner<WithGroupRunner: TestGroupRunner<Extra, GroupKey, GroupCtx>>(
        self,
        group_runner: WithGroupRunner,
    ) -> GroupedTestHarness<
        't,
        Extra,
        GroupKey,
        GroupCtx,
        Filter,
        Grouper,
        Groups,
        Ignore,
        WithGroupRunner,
        PanicHandler,
        Runner,
        Formatter,
    > {
        GroupedTestHarness {
            tests: self.tests,
            _group_key: PhantomData,
            _group_ctx: PhantomData,
            filter: self.filter,
            grouper: self.grouper,
            groups: self.groups,
            ignore: self.ignore,
            group_runner,
            panic_handler: self.panic_handler,
            runner: self.runner,
            formatter: self.formatter,
        }
    }

    pub fn with_panic_handler<WithPanicHandler: TestPanicHandler<Extra>>(
        self,
        panic_handler: WithPanicHandler,
    ) -> GroupedTestHarness<
        't,
        Extra,
        GroupKey,
        GroupCtx,
        Filter,
        Grouper,
        Groups,
        Ignore,
        GroupRunner,
        WithPanicHandler,
        Runner,
        Formatter,
    > {
        GroupedTestHarness {
            tests: self.tests,
            _group_key: PhantomData,
            _group_ctx: PhantomData,
            filter: self.filter,
            grouper: self.grouper,
            groups: self.groups,
            ignore: self.ignore,
            group_runner: self.group_runner,
            panic_handler,
            runner: self.runner,
            formatter: self.formatter,
        }
    }

    pub fn with_runner<WithRunner: TestRunner<Extra>>(
        self,
        runner: WithRunner,
    ) -> GroupedTestHarness<
        't,
        Extra,
        GroupKey,
        GroupCtx,
        Filter,
        Grouper,
        Groups,
        Ignore,
        GroupRunner,
        PanicHandler,
        WithRunner,
        Formatter,
    > {
        GroupedTestHarness {
            tests: self.tests,
            _group_key: PhantomData,
            _group_ctx: PhantomData,
            filter: self.filter,
            grouper: self.grouper,
            groups: self.groups,
            ignore: self.ignore,
            group_runner: self.group_runner,
            panic_handler: self.panic_handler,
            runner,
            formatter: self.formatter,
        }
    }

    pub fn with_formatter<WithFormatter>(
        self,
        formatter: WithFormatter,
    ) -> GroupedTestHarness<
        't,
        Extra,
        GroupKey,
        GroupCtx,
        Filter,
        Grouper,
        Groups,
        Ignore,
        GroupRunner,
        PanicHandler,
        Runner,
        WithFormatter,
    > {
        GroupedTestHarness {
            tests: self.tests,
            _group_key: PhantomData,
            _group_ctx: PhantomData,
            filter: self.filter,
            grouper: self.grouper,
            groups: self.groups,
            ignore: self.ignore,
            group_runner: self.group_runner,
            panic_handler: self.panic_handler,
            runner: self.runner,
            formatter,
        }
    }
}
