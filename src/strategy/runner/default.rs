use std::{
    cmp,
    num::NonZeroUsize,
    thread::{Scope, ScopedJoinHandle},
    time::Instant,
};

use crate::{
    outcome::{TestOutcome, TestOutcomeAttachments, TestStatus},
    runner::TestRunner,
    test::TestMeta,
};

// TODO: add early aborting and keep going flag

#[derive(Debug)]
pub struct DefaultRunner {
    threads: NonZeroUsize,
}

impl Default for DefaultRunner {
    fn default() -> Self {
        Self {
            threads: std::thread::available_parallelism().unwrap_or(NonZeroUsize::MIN),
        }
    }
}

impl DefaultRunner {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_thread_count(self, count: NonZeroUsize) -> Self {
        Self { threads: count }
    }
}

struct DefaultRunnerIterator<'t, 's, I, F, Extra>
where
    I: Iterator<Item = (F, &'t TestMeta<Extra>)>,
    F: (Fn() -> TestStatus) + Send,
    Extra: 't,
{
    source: I,
    push_job: crossbeam_channel::Sender<Option<(F, &'t TestMeta<Extra>)>>,
    wait_job: crossbeam_channel::Receiver<(&'t TestMeta<Extra>, TestOutcome)>,
    _scope: &'s Scope<'s, 't>,
    _workers: Vec<ScopedJoinHandle<'s, ()>>,
}

impl<'t, 's, I, F, Extra: Sync> DefaultRunnerIterator<'t, 's, I, F, Extra>
where
    I: Iterator<Item = (F, &'t TestMeta<Extra>)>,
    F: (Fn() -> TestStatus) + Send + 's,
    Extra: 't,
{
    fn new(worker_count: NonZeroUsize, mut iter: I, scope: &'s Scope<'s, 't>) -> Self {
        let (itx, irx) = crossbeam_channel::bounded(worker_count.into());
        let (otx, orx) = crossbeam_channel::bounded(1);
        let workers = (0..worker_count.get())
            .map(|_| {
                let irx = irx.clone();
                let otx = otx.clone();
                itx.send(iter.next()).expect("open space in channel");
                scope.spawn(move || {
                    while let Ok(Some((f, meta))) = irx.recv() {
                        let now = Instant::now();
                        let status = f();
                        let duration = now.elapsed();
                        let send_outcome_res = otx.send((
                            meta,
                            TestOutcome {
                                status,
                                duration,
                                stdout: Vec::new(),
                                stderr: Vec::new(),
                                attachments: TestOutcomeAttachments::default(),
                            },
                        ));
                        if send_outcome_res.is_err() {
                            // If receiver dropped, the work is irrelevant anymore, drop silently.
                            return;
                        }
                    }
                })
            })
            .collect();

        Self {
            source: iter,
            push_job: itx,
            wait_job: orx,
            _scope: scope,
            _workers: workers,
        }
    }
}

impl<'t, 's, I, F, Extra> Iterator for DefaultRunnerIterator<'t, 's, I, F, Extra>
where
    I: Iterator<Item = (F, &'t TestMeta<Extra>)>,
    F: (Fn() -> TestStatus) + Send + 's,
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

impl<Extra: Sync> TestRunner<Extra> for DefaultRunner {
    fn run<'t, 's, I, F>(
        &self,
        tests: I,
        scope: &'s Scope<'s, 't>,
    ) -> impl Iterator<Item = (&'t TestMeta<Extra>, TestOutcome)>
    where
        I: ExactSizeIterator<Item = (F, &'t TestMeta<Extra>)>,
        F: (Fn() -> TestStatus) + Send + 's,
        Extra: 't,
    {
        let worker_count = <DefaultRunner as TestRunner<Extra>>::worker_count(self, tests.len());
        DefaultRunnerIterator::new(worker_count, tests, scope)
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
    fn thread_count_works() {
        let tests: Vec<_> = (0..4)
            .map(|idx| {
                test! {
                    name: format!("test_{idx}"),
                    func: || thread::sleep(Duration::from_millis(100))
                }
            })
            .collect();

        const FOUR: NonZeroUsize = NonZeroUsize::new(4).unwrap();
        let parallel = harness(&tests)
            .with_runner(DefaultRunner::default().with_thread_count(FOUR))
            .run();

        const ONE: NonZeroUsize = NonZeroUsize::new(1).unwrap();
        let serial = harness(&tests)
            .with_runner(DefaultRunner::default().with_thread_count(ONE))
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

        const FIFTY: NonZeroUsize = NonZeroUsize::new(50).unwrap();
        let max = harness(&tests)
            .with_runner(DefaultRunner::default().with_thread_count(FIFTY))
            .run();
        assert!(max.duration < Duration::from_millis(20) + PADDING);
    }
}
