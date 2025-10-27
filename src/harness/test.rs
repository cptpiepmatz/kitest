use std::{marker::PhantomData, panic::RefUnwindSafe, sync::Arc, time::Instant};

use crate::{
    GroupedTestHarness, TestReport,
    filter::{FilteredTests, TestFilter},
    formatter::*,
    group::{SimpleGroupRunner, TestGroupHashMap, TestGrouper},
    ignore::TestIgnore,
    outcome::TestStatus,
    panic_handler::TestPanicHandler,
    runner::TestRunner,
    test::Test,
};

use super::{FmtErrors, named_fmt};

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
        fmt_errors.push_on_error(named_fmt!(
            formatter.fmt_run_init(FmtRunInitData { tests: self.tests }.into())
        ));

        let FilteredTests { tests, filtered } = self.filter.filter(self.tests);
        fmt_errors.push_on_error(named_fmt!(
            formatter.fmt_run_start(
                FmtRunStart {
                    active: tests.len(),
                    filtered
                }
                .into()
            )
        ));

        let ignore = Arc::new(self.ignore);
        let panic_handler = Arc::new(self.panic_handler);

        let (outcomes, mut formatter, mut fmt_errors) = std::thread::scope(move |scope| {
            let (ftx, frx) =
                crossbeam_channel::bounded(4 * self.runner.worker_count(tests.len()).get());
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
    ) -> impl IntoIterator<IntoIter = impl ExactSizeIterator<Item = (&'static str, Formatter::Error)>>
    {
        let mut formatter = self.formatter;
        let mut fmt_errors = Vec::new();
        fmt_errors.push_on_error(named_fmt!(
            formatter.fmt_init_listing(FmtInitListing { tests: self.tests }.into())
        ));

        let FilteredTests { tests, filtered } = self.filter.filter(self.tests);
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
            group_runner: SimpleGroupRunner::default(),
            panic_handler: self.panic_handler,
            runner: self.runner,
            formatter: self.formatter,
        }
    }
}
