use std::{marker::PhantomData, panic::RefUnwindSafe, sync::Arc, time::Instant};

use crate::{
    GroupedTestHarness, TestReport,
    filter::{FilteredTests, TestFilter},
    formatter::*,
    group::{SimpleGroupRunner, TestGroupHashMap, TestGrouper},
    harness::FmtErrors,
    ignore::{IgnoreStatus, TestIgnore},
    outcome::TestStatus,
    panic::TestPanicHandler,
    runner::TestRunner,
    test::Test,
};

/// A configurable test harness.
///
/// [`TestHarness`] is the main operator of Kitest.
/// It holds the full test list and all strategies that define how a test run behaves:
/// [filtering](Self::with_filter), [ignoring](Self::with_ignore),
/// [panic handling](Self::with_panic_handler), [running](Self::with_runner), and
/// [formatting](Self::with_formatter).
///
/// A harness is lazy.
/// Constructing it does not do anything by itself.
/// To actually do work, call either [`run`](Self::run) to execute tests or
/// [`list`](Self::list) to list tests.
///
/// ## Configuration
///
/// A harness is configured by chaining `with_*` methods.
/// Each `with_*` call replaces exactly one strategy and returns a new `TestHarness` with an
/// updated generic type. This keeps the configuration type safe and avoids runtime indirection.
///
/// To enable grouping, call [`with_grouper`](Self::with_grouper).
/// This promotes the harness into a [`GroupedTestHarness`].
/// From that point on, tests are executed through groups and group specific strategies can be
/// configured.
///
/// ## Generics and type inference
///
/// The type of a fully configured harness can look intimidating in rustdoc because it carries
/// a generic parameter for each strategy.
/// In normal usage you rarely have to write these types explicitly.
/// Type inference will figure them out from the strategies you keep or replace.
///
/// ## Lifetimes
///
/// The lifetime parameter `'t` is the lifetime of the test slice stored in the harness.
/// All strategies are allowed to borrow from the tests through `'t`, which avoids unnecessary
/// allocations and copying.
#[derive(Debug, Clone)]
#[must_use = "test harnesses are lazy, you have to call either `run` or `list` to do something"]
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
    /// Execute the test harness and produce a [`TestReport`].
    ///
    /// This runs the full test pipeline:
    /// - filters tests
    /// - applies ignore rules
    /// - executes tests through the runner
    /// - captures output and panics
    /// - forwards events to the formatter
    ///
    /// The harness is consumed by this call. After running, the result is returned
    /// as a [`TestReport`], which can be converted into an exit status.
    ///
    /// Formatting errors are collected and included in the report instead of
    /// aborting the run early.
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
    /// List tests without executing them.
    ///
    /// This runs the harness in listing mode. Tests are filtered and ignored in the
    /// same way as during a normal run, but test functions are never executed.
    ///
    /// The formatter is notified of listing events and may print a test overview
    /// similar to `cargo test -- --list`.
    ///
    /// The harness is consumed by this call.
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
    /// Replace the ignore strategy.
    ///
    /// The ignore strategy decides whether a test is executed or reported as ignored,
    /// optionally with a reason.
    ///
    /// This does not remove tests from the harness. Ignored tests are still visible
    /// to formatters and listing mode.
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

    /// Replace the filter strategy.
    ///
    /// The filter strategy decides which tests participate in the run at all.
    /// Filtered tests are removed before execution and are not passed to the runner.
    ///
    /// Filtering happens before ignoring.
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

    /// Replace the panic handler.
    ///
    /// The panic handler is responsible for executing the test function and
    /// converting panics into a [`TestStatus`], taking metadata such as
    /// `should_panic` into account.
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

    /// Replace the test runner.
    ///
    /// The runner controls how tests are scheduled and executed, for example
    /// sequentially or in parallel.
    ///
    /// Runners receive prepared test closures and are responsible for driving
    /// execution and returning outcomes.
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

    /// Replace the formatter.
    ///
    /// The formatter receives structured events describing the test run and is
    /// responsible for producing output.
    ///
    /// This includes progress reporting, test results, and final summaries.
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

    /// Enable grouping and promote this harness into a [`GroupedTestHarness`].
    ///
    /// Calling this method switches the execution model from individual tests to
    /// test groups. Tests are assigned to groups using the provided [`TestGrouper`],
    /// and all execution happens through those groups.
    ///
    /// This is a type level transition. Once grouping is enabled, group specific
    /// strategies such as group runners and grouped formatters become available.
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
