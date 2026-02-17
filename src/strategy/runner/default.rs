use std::{
    cmp,
    fmt::Debug,
    num::NonZeroUsize,
    sync::Arc,
    thread::{self, Scope, ScopedJoinHandle},
    time::Instant,
};

use crate::{
    capture::{
        CapturePanicHookGuard, DefaultPanicHookProvider, OutputCapture, PanicHook,
        PanicHookProvider, TEST_OUTPUT_CAPTURE,
    },
    outcome::{TestOutcome, TestOutcomeAttachments, TestStatus},
    runner::{
        TestRunner,
        scope::{NoScopeFactory, TestScope, TestScopeFactory},
    },
    test::TestMeta,
};

// TODO: add early aborting and keep going flag

/// The default [`TestRunner`] implementation used by the default test harness.
///
/// The behavior is meant to feel similar to the built in Rust test harness:
/// tests are executed on a worker pool, outcomes are collected as they finish,
/// and the order of results is not tied to the input order.
///
/// This runner uses multiple threads.
/// By default, the thread count is based on [`std::thread::available_parallelism`], but it can be
/// overridden.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DefaultRunner<PanicHookProvider, TestScopeFactory> {
    threads: NonZeroUsize,
    panic_hook_provider: PanicHookProvider,
    test_scope_factory: Arc<TestScopeFactory>,
}

impl Default for DefaultRunner<DefaultPanicHookProvider, NoScopeFactory> {
    fn default() -> Self {
        Self {
            threads: std::thread::available_parallelism().unwrap_or(NonZeroUsize::MIN),
            panic_hook_provider: DefaultPanicHookProvider,
            test_scope_factory: Arc::new(NoScopeFactory),
        }
    }
}

impl<PanicHookProvider, TestScopeFactory> DefaultRunner<PanicHookProvider, TestScopeFactory> {
    /// Create a default runner using the default panic hook provider.
    ///
    /// This is the same as `DefaultRunner::default()`.
    pub fn new() -> DefaultRunner<DefaultPanicHookProvider, NoScopeFactory> {
        DefaultRunner::default()
    }

    /// Override the number of worker threads used by the runner.
    ///
    /// This replaces the previous thread count.
    pub fn with_thread_count(self, count: NonZeroUsize) -> Self {
        Self {
            threads: count,
            ..self
        }
    }

    /// Replace the panic hook provider used for output capture.
    ///
    /// The runner is generic over a [`PanicHookProvider`] so we can swap out the
    /// output capture panic hook behavior without replacing the whole runner.
    pub fn with_panic_hook_provider<WithPanicHookProvider>(
        self,
        panic_hook_provider: WithPanicHookProvider,
    ) -> DefaultRunner<WithPanicHookProvider, TestScopeFactory> {
        DefaultRunner {
            threads: self.threads,
            panic_hook_provider,
            test_scope_factory: self.test_scope_factory,
        }
    }

    /// Replace the [`TestScopeFactory`] used by this runner.
    ///
    /// This allows injecting per test lifecycle hooks without replacing the entire runner.
    /// The factory produces one scope instance per test, and that scope instance is used for both
    /// [`before_test`](TestScope::before_test) and [`after_test`](TestScope::after_test).
    ///
    /// This replaces the previous scope factory.
    pub fn with_test_scope_factory<WithTestScopeFactory>(
        self,
        test_scope_factory: WithTestScopeFactory,
    ) -> DefaultRunner<PanicHookProvider, WithTestScopeFactory> {
        DefaultRunner {
            threads: self.threads,
            panic_hook_provider: self.panic_hook_provider,
            test_scope_factory: Arc::new(test_scope_factory),
        }
    }
}

struct DefaultRunnerIterator<'t, 's, I, F, T, Extra>
where
    I: Iterator<Item = (F, &'t TestMeta<Extra>)>,
    F: (Fn() -> TestStatus) + Send,
    T: TestScopeFactory<'t, Extra>,
    Extra: 't,
{
    source: I,
    push_job: crossbeam_channel::Sender<Option<(F, &'t TestMeta<Extra>)>>,
    wait_job: crossbeam_channel::Receiver<(&'t TestMeta<Extra>, TestOutcome)>,
    _scope: &'s Scope<'s, 't>,
    _workers: Vec<ScopedJoinHandle<'s, ()>>,
    _panic_hook: CapturePanicHookGuard,
    _test_scope_factory: Arc<T>,
}

impl<'t, 's, I, F, T, Extra: Sync> DefaultRunnerIterator<'t, 's, I, F, T, Extra>
where
    I: Iterator<Item = (F, &'t TestMeta<Extra>)>,
    F: (Fn() -> TestStatus) + Send + 's,
    T: TestScopeFactory<'t, Extra> + Send + Sync + 'static,
    Extra: 't,
{
    fn new(
        worker_count: NonZeroUsize,
        mut iter: I,
        scope: &'s Scope<'s, 't>,
        panic_hook: PanicHook,
        test_scope_factory: Arc<T>,
    ) -> Self {
        let (itx, irx) = crossbeam_channel::bounded(worker_count.into());
        let (otx, orx) = crossbeam_channel::bounded(1);
        let workers = (0..worker_count.get())
            .map(|idx| {
                let irx = irx.clone();
                let otx = otx.clone();
                let test_scope_factory = test_scope_factory.clone();
                itx.send(iter.next()).expect("open space in channel");
                thread::Builder::new()
                    .name(format!("kitest-worker-{idx}"))
                    .spawn_scoped(scope, move || {
                        while let Ok(Some((f, meta))) = irx.recv() {
                            let mut test_scope = test_scope_factory.make_scope();
                            test_scope.before_test(meta);

                            let now = Instant::now();
                            let status = f();
                            let duration = now.elapsed();
                            let output = TEST_OUTPUT_CAPTURE.with_borrow_mut(OutputCapture::take);
                            let outcome = TestOutcome {
                                status,
                                duration,
                                output,
                                attachments: TestOutcomeAttachments::default(),
                            };

                            test_scope.after_test(meta, &outcome);
                            let send_outcome_res = otx.send((meta, outcome));
                            if send_outcome_res.is_err() {
                                // If receiver dropped, the work is irrelevant anymore, drop silently.
                                return;
                            }
                        }
                    })
                    .expect("name has no null byte")
            })
            .collect();

        Self {
            source: iter,
            push_job: itx,
            wait_job: orx,
            _scope: scope,
            _workers: workers,
            _panic_hook: CapturePanicHookGuard::install(panic_hook),
            _test_scope_factory: test_scope_factory,
        }
    }
}

impl<'t, 's, I, F, T, Extra> Iterator for DefaultRunnerIterator<'t, 's, I, F, T, Extra>
where
    I: Iterator<Item = (F, &'t TestMeta<Extra>)>,
    F: (Fn() -> TestStatus) + Send + 's,
    T: TestScopeFactory<'t, Extra>,
    Extra: 't,
{
    type Item = (&'t TestMeta<Extra>, TestOutcome);

    fn next(&mut self) -> Option<Self::Item> {
        let out = self.wait_job.recv().ok();
        let next_job = self.source.next();
        if let Err(crossbeam_channel::SendError(Some((_, meta)))) = self.push_job.send(next_job) {
            // At the end we'll only send `None` values to signal workers to stop.
            // If sending `None` fails, that's fine — it just means all workers have exited.
            // But if we fail to send a real job, it means no workers are alive,
            // which should never happen.
            panic!("no worker available for job {}", meta.name);
        }
        out
    }
}

impl<'t, P, T, Extra> TestRunner<'t, Extra> for DefaultRunner<P, T>
where
    T: TestScopeFactory<'t, Extra> + Send + Sync + 'static,
    P: PanicHookProvider,
    Extra: Sync,
{
    fn run<'s, I, F>(
        &self,
        tests: I,
        scope: &'s Scope<'s, 't>,
    ) -> impl Iterator<Item = (&'t TestMeta<Extra>, TestOutcome)>
    where
        I: ExactSizeIterator<Item = (F, &'t TestMeta<Extra>)>,
        F: (Fn() -> TestStatus) + Send + 's,
        Extra: 't,
    {
        let worker_count =
            <DefaultRunner<_, _> as TestRunner<Extra>>::worker_count(self, tests.len());
        DefaultRunnerIterator::new(
            worker_count,
            tests,
            scope,
            self.panic_hook_provider.provide(),
            self.test_scope_factory.clone(),
        )
    }

    fn worker_count(&self, test_count: usize) -> NonZeroUsize {
        NonZeroUsize::new(cmp::min(self.threads.get(), test_count)).unwrap_or(NonZeroUsize::MIN)
    }
}

#[cfg(test)]
mod tests {
    use std::{thread, time::Duration};

    use super::*;
    use crate::test_support::*;

    #[test]
    #[cfg_attr(all(ci, target_os = "macos"), ignore = "too slow on macos")]
    fn run_tests_in_parallel() {
        let tests = &[
            test! {name: "a", func: || thread::sleep(Duration::from_millis(100))},
            test! {name: "b", func: || thread::sleep(Duration::from_millis(50))},
            test! {name: "c", func: || thread::sleep(Duration::from_millis(200))},
            test! {name: "d", func: || thread::sleep(Duration::from_millis(10))},
        ];

        let report = harness(tests).with_runner(DefaultRunner::default()).run();

        let order = report
            .outcomes
            .iter()
            .fold(String::new(), |s, (name, _)| s + name);
        assert_eq!(order, "dbac");

        assert!(report.duration < Duration::from_millis(300));
    }

    #[test]
    #[cfg_attr(all(ci, target_os = "macos"), ignore = "too slow on macos")]
    fn thread_count_works() {
        let tests: Vec<_> = (0..4)
            .map(|idx| {
                test! {
                    name: format!("test_{idx}"),
                    func: || thread::sleep(Duration::from_millis(100))
                }
            })
            .collect();

        let parallel = harness(&tests)
            .with_runner(DefaultRunner::default().with_thread_count(nonzero!(4)))
            .run();

        let serial = harness(&tests)
            .with_runner(DefaultRunner::default().with_thread_count(nonzero!(1)))
            .run();

        assert!(parallel.duration < Duration::from_millis(200));
        assert!(parallel.duration < serial.duration);
        assert!(serial.duration >= Duration::from_millis(400));
    }

    #[test]
    #[cfg_attr(all(ci, target_os = "macos"), ignore = "too slow on macos")]
    fn expected_execution_time() {
        const PADDING: Duration = Duration::from_millis(50);

        let tests: Vec<_> = (0..50)
            .map(|_| test! {func: || thread::sleep(Duration::from_millis(20))})
            .collect();

        let default = harness(&tests).with_runner(DefaultRunner::default()).run();
        let expected_duration = Duration::from_millis(
            ((50.0 / thread::available_parallelism().unwrap().get() as f64) * 20.0) as u64,
        );
        assert!(default.duration < expected_duration + PADDING);

        let max = harness(&tests)
            .with_runner(DefaultRunner::default().with_thread_count(nonzero!(50)))
            .run();
        assert!(max.duration < Duration::from_millis(20) + PADDING);
    }
}
