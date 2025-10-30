use std::{marker::PhantomData, panic::RefUnwindSafe, sync::Arc, time::Instant};

use crate::{
    GroupedTestHarness, TestReport,
    filter::{FilteredTests, TestFilter},
    formatter::*,
    group::{SimpleGroupRunner, TestGroupHashMap, TestGrouper},
    harness::FmtErrors,
    ignore::{IgnoreStatus, TestIgnore},
    outcome::TestStatus,
    panic_handler::TestPanicHandler,
    runner::TestRunner,
    test::Test,
};

#[derive(Debug)]
pub struct TestHarness<'t, Extra, Filter, Ignore, PanicHandler, Runner, Formatter> {
    pub(crate) tests: &'t [Test<Extra>],
    pub(crate) filter: Filter,
    pub(crate) ignore: Ignore,
    pub(crate) panic_handler: PanicHandler,
    pub(crate) runner: Runner,
    pub(crate) formatter: Formatter,
}

impl<
    't,
    Extra: RefUnwindSafe + Sync,
    Filter: TestFilter<Extra>,
    Ignore: TestIgnore<Extra> + Send + Sync + 't,
    PanicHandler: TestPanicHandler<Extra> + Send + Sync + 't,
    Runner: TestRunner<Extra>,
    Formatter: TestFormatter<'t, Extra> + 't,
> TestHarness<'t, Extra, Filter, Ignore, PanicHandler, Runner, Formatter>
{
    pub fn run(self) -> TestReport<'t, Formatter::Error> {
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
        fmt_errors.push_on_error(
            FmtRunStart {
                active: tests.len(),
                filtered,
            }
            .fmt(|data| formatter.fmt_run_start(data)),
        );

        let ignore = Arc::new(self.ignore);
        let panic_handler = Arc::new(self.panic_handler);

        let (outcomes, mut formatter, mut fmt_errors) = std::thread::scope(move |scope| {
            let (ftx, frx) =
                crossbeam_channel::bounded(4 * self.runner.worker_count(tests.len()).get());
            let fmt_thread = scope.spawn(move || {
                while let Ok(fmt_data) = frx.recv() {
                    fmt_errors.push_on_error(match fmt_data {
                        FmtTestData::Ignored(data) => formatter
                            .fmt_test_ignored(data)
                            .map_err(|err| (FormatError::TestIgnored, err)),
                        FmtTestData::Start(data) => formatter
                            .fmt_test_start(data)
                            .map_err(|err| (FormatError::TestStart, err)),
                        FmtTestData::Outcome(data) => formatter
                            .fmt_test_outcome(data)
                            .map_err(|err| (FormatError::TestOutcome, err)),
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
                        let reason = match ignore.ignore(meta) {
                            IgnoreStatus::Run => {
                                let _ = ftx.send(FmtTestData::Start(FmtTestStart { meta }.into()));
                                return panic_handler.handle(|| test.call(), meta);
                            }
                            IgnoreStatus::Ignore => None,
                            IgnoreStatus::IgnoreWithReason(cow) => Some(cow),
                        };

                        let _ = ftx.send(FmtTestData::Ignored(
                            FmtTestIgnored {
                                meta,
                                reason: reason.as_ref(),
                            }
                            .into(),
                        ));

                        TestStatus::Ignored { reason }
                    },
                    meta,
                )
            });

            let outcomes = self
                .runner
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
        fmt_errors.push_on_error(
            FmtRunOutcomes {
                outcomes: &outcomes,
                filtered_out: filtered,
                duration,
            }
            .fmt(|data| formatter.fmt_run_outcomes(data)),
        );

        TestReport {
            outcomes,
            duration,
            fmt_errors,
        }
    }
}

impl<
    't,
    Extra,
    Filter: TestFilter<Extra>,
    Ignore: TestIgnore<Extra>,
    PanicHandler,
    Runner,
    Formatter: TestListFormatter<'t, Extra>,
> TestHarness<'t, Extra, Filter, Ignore, PanicHandler, Runner, Formatter>
{
    pub fn list(
        self,
    ) -> impl IntoIterator<IntoIter = impl ExactSizeIterator<Item = (FormatError, Formatter::Error)>>
    {
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

        let mut active_count = 0;
        let mut ignore_count = 0;
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
            FmtEndListing {
                active: active_count,
                ignored: ignore_count,
            }
            .fmt(|data| formatter.fmt_end_listing(data)),
        );

        fmt_errors
    }
}

impl<'t, Extra, Filter, Ignore, PanicHandler, Runner, Formatter>
    TestHarness<'t, Extra, Filter, Ignore, PanicHandler, Runner, Formatter>
{
    pub fn with_ignore<WithIgnore: TestIgnore<Extra>>(
        self,
        ignore: WithIgnore,
    ) -> TestHarness<'t, Extra, Filter, WithIgnore, PanicHandler, Runner, Formatter> {
        TestHarness {
            tests: self.tests,
            filter: self.filter,
            ignore,
            panic_handler: self.panic_handler,
            runner: self.runner,
            formatter: self.formatter,
        }
    }

    pub fn with_filter<WithFilter: TestFilter<Extra>>(
        self,
        filter: WithFilter,
    ) -> TestHarness<'t, Extra, WithFilter, Ignore, PanicHandler, Runner, Formatter> {
        TestHarness {
            tests: self.tests,
            filter,
            ignore: self.ignore,
            panic_handler: self.panic_handler,
            runner: self.runner,
            formatter: self.formatter,
        }
    }

    pub fn with_panic_handler<WithPanicHandler: TestPanicHandler<Extra>>(
        self,
        panic_handler: WithPanicHandler,
    ) -> TestHarness<'t, Extra, Filter, Ignore, WithPanicHandler, Runner, Formatter> {
        TestHarness {
            tests: self.tests,
            filter: self.filter,
            ignore: self.ignore,
            panic_handler,
            runner: self.runner,
            formatter: self.formatter,
        }
    }

    pub fn with_runner<WithRunner: TestRunner<Extra>>(
        self,
        runner: WithRunner,
    ) -> TestHarness<'t, Extra, Filter, Ignore, PanicHandler, WithRunner, Formatter> {
        TestHarness {
            tests: self.tests,
            filter: self.filter,
            ignore: self.ignore,
            panic_handler: self.panic_handler,
            runner,
            formatter: self.formatter,
        }
    }

    pub fn with_formatter<WithFormatter>(
        self,
        formatter: WithFormatter,
    ) -> TestHarness<'t, Extra, Filter, Ignore, PanicHandler, Runner, WithFormatter> {
        TestHarness {
            tests: self.tests,
            filter: self.filter,
            ignore: self.ignore,
            panic_handler: self.panic_handler,
            runner: self.runner,
            formatter,
        }
    }

    pub fn with_grouper<WithGrouper: TestGrouper<Extra, GroupKey, GroupCtx>, GroupKey, GroupCtx>(
        self,
        grouper: WithGrouper,
    ) -> GroupedTestHarness<
        't,
        Extra,
        GroupKey,
        GroupCtx,
        Filter,
        WithGrouper,
        TestGroupHashMap<'t, Extra, GroupKey>,
        Ignore,
        SimpleGroupRunner,
        PanicHandler,
        Runner,
        Formatter,
    > {
        GroupedTestHarness {
            tests: self.tests,
            _group_key: PhantomData,
            _group_ctx: PhantomData,
            filter: self.filter,
            grouper,
            groups: TestGroupHashMap::default(),
            ignore: self.ignore,
            group_runner: SimpleGroupRunner,
            panic_handler: self.panic_handler,
            runner: self.runner,
            formatter: self.formatter,
        }
    }
}
