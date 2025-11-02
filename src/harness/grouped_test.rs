use std::{marker::PhantomData, ops::ControlFlow, panic::RefUnwindSafe, sync::Arc, time::Instant};

use crate::{
    GroupedTestReport,
    filter::{FilteredTests, TestFilter},
    formatter::*,
    group::{TestGroupRunner, TestGrouper, TestGroups},
    harness::FmtErrors,
    ignore::{IgnoreStatus, TestIgnore},
    outcome::TestStatus,
    panic::TestPanicHandler,
    runner::TestRunner,
    test::Test,
};

#[derive(Debug)]
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
    GroupKey: 't,
    GroupCtx: 't,
    Filter: TestFilter<Extra>,
    Grouper: TestGrouper<Extra, GroupKey, GroupCtx>,
    Groups: TestGroups<'t, Extra, GroupKey>,
    Ignore: TestIgnore<Extra> + Send + Sync + 't,
    GroupRunner: TestGroupRunner<'t, Extra, GroupKey, GroupCtx>,
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
        fmt_errors.push_on_error(
            FmtRunInit { tests: self.tests }.fmt(|data| formatter.fmt_run_init(data)),
        );

        let FilteredTests {
            tests,
            filtered_out: filtered,
        } = self.filter.filter(self.tests);
        tests.for_each(|test| self.groups.add(self.grouper.group(test), test));

        fmt_errors.push_on_error(
            FmtGroupedRunStart {
                tests: self.groups.len(),
                filtered,
            }
            .fmt(|data| formatter.fmt_grouped_run_start(data)),
        );
        let (grouped_outcomes, mut formatter, mut fmt_errors) = std::thread::scope(move |scope| {
            // TODO: prefer getting only the MAX value and not the total count of tests for the worker_count estimation
            let (ftx, frx) =
                crossbeam_channel::bounded(self.runner.worker_count(self.groups.len()).get());
            let fmt_thread = scope.spawn(move || {
                while let Ok(fmt_data) = frx.recv() {
                    fmt_errors.push_on_error(match fmt_data {
                        FmtGroupedTestData::Start(data) => formatter
                            .fmt_group_start(data)
                            .map_err(|err| (FormatError::GroupStart, err)),
                        FmtGroupedTestData::Test(FmtTestData::Ignored(data)) => formatter
                            .fmt_test_ignored(data)
                            .map_err(|err| (FormatError::TestIgnored, err)),
                        FmtGroupedTestData::Test(FmtTestData::Start(data)) => formatter
                            .fmt_test_start(data)
                            .map_err(|err| (FormatError::TestStart, err)),
                        FmtGroupedTestData::Test(FmtTestData::Outcome(data)) => formatter
                            .fmt_test_outcome(data)
                            .map_err(|err| (FormatError::TestOutcome, err)),
                        FmtGroupedTestData::Outcome(data) => formatter
                            .fmt_group_outcomes(data)
                            .map_err(|err| (FormatError::GroupOutcomes, err)),
                    });
                }
                (formatter, fmt_errors)
            });

            let ignore = Arc::new(self.ignore);
            let panic_handler = Arc::new(self.panic_handler);
            let runner = Arc::new(self.runner);

            let group_runs = self.groups.into_groups().scan(
                ControlFlow::Continue(()),
                |control_flow, (key, tests)| {
                    if *control_flow == ControlFlow::Break(()) {
                        return None;
                    }

                    let now = Instant::now();

                    let ignore = Arc::clone(&ignore);
                    let panic_handler = Arc::clone(&panic_handler);
                    let runner = Arc::clone(&runner);
                    let ftx = ftx.clone();
                    let ctx = self.grouper.group_ctx(&key);

                    let _ = ftx.send(FmtGroupedTestData::Start(
                        FmtGroupStart {
                            tests: tests.len(),
                            worker_count: runner.worker_count(tests.len()),
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
                                        let reason = match ignore.ignore(meta) {
                                            IgnoreStatus::Run => {
                                                let _ = ftx.send(FmtGroupedTestData::Test(
                                                    FmtTestData::Start(
                                                        FmtTestStart { meta }.into(),
                                                    ),
                                                ));
                                                return panic_handler.handle(|| test.call(), meta);
                                            }
                                            IgnoreStatus::Ignore => None,
                                            IgnoreStatus::IgnoreWithReason(reason) => Some(reason),
                                        };
                                        let _ = ftx.send(FmtGroupedTestData::Test(
                                            FmtTestData::Ignored(
                                                FmtTestIgnored {
                                                    meta,
                                                    reason: reason.as_ref(),
                                                }
                                                .into(),
                                            ),
                                        ));
                                        TestStatus::Ignored { reason }
                                    },
                                    meta,
                                )
                            });

                            runner
                                .run(test_runs, scope)
                                .inspect(|(meta, outcome)| {
                                    let _ =
                                        ftx.send(FmtGroupedTestData::Test(FmtTestData::Outcome(
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

                    let outcomes = match outcomes {
                        ControlFlow::Continue(outcomes) => outcomes,
                        ControlFlow::Break(outcomes) => {
                            *control_flow = ControlFlow::Break(());
                            outcomes
                        }
                    };

                    Some((outcomes, now.elapsed(), key, ctx))
                },
            );

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
        fmt_errors.push_on_error(
            FmtGroupedRunOutcomes {
                outcomes: &grouped_outcomes,
                duration,
            }
            .fmt(|data| formatter.fmt_grouped_run_outcomes(data)),
        );

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
    pub fn list(mut self) -> Vec<(FormatError, Formatter::Error)> {
        let mut formatter = self.formatter;
        let mut fmt_errors = Vec::new();
        fmt_errors.push_on_error(
            FmtInitListing { tests: self.tests }.fmt(|data| formatter.fmt_init_listing(data)),
        );

        let FilteredTests {
            tests,
            filtered_out: filtered,
        } = self.filter.filter(self.tests);
        fmt_errors.push_on_error(
            FmtBeginListing {
                tests: tests.len(),
                filtered,
            }
            .fmt(|data| formatter.fmt_begin_listing(data)),
        );

        tests.for_each(|test| self.groups.add(self.grouper.group(test), test));
        let groups = self.groups.into_groups();
        fmt_errors.push_on_error(
            FmtListGroups {
                groups: groups.len(),
            }
            .fmt(|data| formatter.fmt_list_groups(data)),
        );

        let mut active_count = 0;
        let mut ignore_count = 0;
        for (key, tests) in groups {
            let ctx = self.grouper.group_ctx(&key);
            let tests_len = tests.len();

            fmt_errors.push_on_error(
                FmtListGroupStart {
                    tests: tests_len,
                    key: &key,
                    ctx,
                }
                .fmt(|data| formatter.fmt_list_group_start(data)),
            );

            for test in tests {
                let ignored = self.ignore.ignore(test);
                match &ignored {
                    IgnoreStatus::Run => active_count += 1,
                    IgnoreStatus::Ignore | IgnoreStatus::IgnoreWithReason(_) => ignore_count += 1,
                }
                fmt_errors.push_on_error(
                    FmtListTest {
                        meta: test,
                        ignored,
                    }
                    .fmt(|data| formatter.fmt_list_test(data)),
                );
            }

            fmt_errors.push_on_error(
                FmtListGroupEnd {
                    tests: tests_len,
                    key: &key,
                    ctx,
                }
                .fmt(|data| formatter.fmt_list_group_end(data)),
            );
        }

        fmt_errors.push_on_error(
            FmtEndListing {
                active: active_count,
                ignored: ignore_count,
            }
            .fmt(|data| formatter.fmt_end_listing(data)),
        );

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

    pub fn with_group_runner<WithGroupRunner: TestGroupRunner<'t, Extra, GroupKey, GroupCtx>>(
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
