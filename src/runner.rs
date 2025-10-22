use std::{
    cmp,
    num::NonZeroUsize,
    thread::{Scope, ScopedJoinHandle},
    time::Instant,
};

use crate::{
    meta::TestMeta,
    outcome::{TestOutcome, TestOutcomeAttachments, TestStatus},
};

pub trait TestRunner<Extra> {
    fn run<'m, 's, I, F>(
        &self,
        tests: I,
        scope: &'s Scope<'s, 'm>,
    ) -> impl Iterator<Item = (&'m TestMeta<Extra>, TestOutcome)>
    where
        I: ExactSizeIterator<Item = (F, &'m TestMeta<Extra>)> + Send,
        F: (FnOnce() -> TestStatus) + Send + 's,
        Extra: 'm + Sync,
        'm: 's;

    fn worker_count(&self, tests_count: usize) -> NonZeroUsize;
}

#[derive(Default)]
pub struct SimpleRunner;

impl<Extra> TestRunner<Extra> for SimpleRunner {
    fn run<'m, 's, I, F>(
        &self,
        tests: I,
        _: &'s Scope<'s, 'm>,
    ) -> impl Iterator<Item = (&'m TestMeta<Extra>, TestOutcome)>
    where
        I: ExactSizeIterator<Item = (F, &'m TestMeta<Extra>)> + Send,
        F: (FnOnce() -> TestStatus) + Send + 's,
        Extra: 'm + Sync,
        'm: 's,
    {
        tests.map(|(test, meta)| {
            let now = Instant::now();
            let status = test();
            let duration = now.elapsed();
            (
                meta,
                TestOutcome {
                    status,
                    duration,
                    stdout: Vec::new(),
                    stderr: Vec::new(),
                    attachments: TestOutcomeAttachments::default(),
                },
            )
        })
    }

    fn worker_count(&self, _: usize) -> NonZeroUsize {
        const { NonZeroUsize::new(1).unwrap() }
    }
}

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

struct DefaultRunnerIterator<'m, 's, I, F, Extra>
where
    I: Iterator<Item = (F, &'m TestMeta<Extra>)> + Send,
    F: (FnOnce() -> TestStatus) + Send,
    Extra: 'm + Sync,
    'm: 's,
{
    source: I,
    push_job: crossbeam_channel::Sender<Option<(F, &'m TestMeta<Extra>)>>,
    wait_job: crossbeam_channel::Receiver<(&'m TestMeta<Extra>, TestOutcome)>,
    _scope: &'s Scope<'s, 'm>,
    _workers: Vec<ScopedJoinHandle<'s, ()>>,
}

impl<'m, 's, I, F, Extra> DefaultRunnerIterator<'m, 's, I, F, Extra>
where
    I: Iterator<Item = (F, &'m TestMeta<Extra>)> + Send,
    F: (FnOnce() -> TestStatus) + Send + 's,
    Extra: 'm + Sync,
    'm: 's,
{
    fn new(worker_count: NonZeroUsize, mut iter: I, scope: &'s Scope<'s, 'm>) -> Self {
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

impl<'m, 's, I, F, Extra> Iterator for DefaultRunnerIterator<'m, 's, I, F, Extra>
where
    I: Iterator<Item = (F, &'m TestMeta<Extra>)> + Send,
    F: (FnOnce() -> TestStatus) + Send + 's,
    Extra: 'm + Sync,
    'm: 's,
{
    type Item = (&'m TestMeta<Extra>, TestOutcome);

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

impl<Extra> TestRunner<Extra> for DefaultRunner {
    fn run<'m, 's, I, F>(
        &self,
        tests: I,
        scope: &'s Scope<'s, 'm>,
    ) -> impl Iterator<Item = (&'m TestMeta<Extra>, TestOutcome)>
    where
        I: ExactSizeIterator<Item = (F, &'m TestMeta<Extra>)> + Send,
        F: (FnOnce() -> TestStatus) + Send + 's,
        Extra: 'm + Sync,
        'm: 's,
    {
        let worker_count = <DefaultRunner as TestRunner<Extra>>::worker_count(self, tests.len());
        DefaultRunnerIterator::new(worker_count, tests, scope)
    }

    fn worker_count(&self, test_count: usize) -> NonZeroUsize {
        NonZeroUsize::new(cmp::min(self.threads.get(), test_count)).unwrap_or(NonZeroUsize::MIN)
    }
}

pub struct SmartRunner {
    threshold: usize,
    simple: SimpleRunner,
    default: DefaultRunner,
}

impl SmartRunner {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_threshold(self, threshold: usize) -> Self {
        Self { threshold, ..self }
    }

    pub fn with_threads(mut self, threads: NonZeroUsize) -> Self {
        self.default.threads = threads;
        self
    }
}

impl Default for SmartRunner {
    fn default() -> Self {
        Self {
            threshold: 4,
            simple: SimpleRunner,
            default: DefaultRunner::default(),
        }
    }
}

enum SmartRunnerIterator<IS, ID> {
    Simple(IS),
    Default(ID),
}

impl<'m, IS, ID, Extra> Iterator for SmartRunnerIterator<IS, ID>
where
    IS: Iterator<Item = (&'m TestMeta<Extra>, TestOutcome)>,
    ID: Iterator<Item = (&'m TestMeta<Extra>, TestOutcome)>,
    Extra: 'm,
{
    type Item = (&'m TestMeta<Extra>, TestOutcome);

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            SmartRunnerIterator::Simple(i) => i.next(),
            SmartRunnerIterator::Default(i) => i.next(),
        }
    }
}

impl<Extra> TestRunner<Extra> for SmartRunner {
    fn run<'m, 's, I, F>(
        &self,
        tests: I,
        scope: &'s Scope<'s, 'm>,
    ) -> impl Iterator<Item = (&'m TestMeta<Extra>, TestOutcome)>
    where
        I: ExactSizeIterator<Item = (F, &'m TestMeta<Extra>)> + Send,
        F: (FnOnce() -> TestStatus) + Send + 's,
        Extra: 'm + Sync,
        'm: 's,
    {
        match tests.len() <= self.threshold {
            true => SmartRunnerIterator::Simple(<SimpleRunner as TestRunner<Extra>>::run(
                &self.simple,
                tests,
                scope,
            )),
            false => SmartRunnerIterator::Default(<DefaultRunner as TestRunner<Extra>>::run(
                &self.default,
                tests,
                scope,
            )),
        }
    }

    fn worker_count(&self, test_count: usize) -> NonZeroUsize {
        match test_count <= self.threshold {
            true => <SimpleRunner as TestRunner<Extra>>::worker_count(&self.simple, test_count),
            false => <DefaultRunner as TestRunner<Extra>>::worker_count(&self.default, test_count),
        }
    }
}
