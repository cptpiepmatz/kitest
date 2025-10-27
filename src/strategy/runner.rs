use std::{
    cmp,
    num::NonZeroUsize,
    thread::{Scope, ScopedJoinHandle},
    time::Instant,
};

use crate::{
    outcome::{TestOutcome, TestOutcomeAttachments, TestStatus},
    test::TestMeta,
};

pub trait TestRunner<Extra> {
    fn run<'t, 's, I, F>(
        &self,
        tests: I,
        scope: &'s Scope<'s, 't>,
    ) -> impl Iterator<Item = (&'t TestMeta<Extra>, TestOutcome)>
    where
        I: ExactSizeIterator<Item = (F, &'t TestMeta<Extra>)>,
        F: (FnOnce() -> TestStatus) + Send + 's,
        Extra: 't;

    fn worker_count(&self, tests_count: usize) -> NonZeroUsize;
}

#[derive(Debug, Default)]
pub struct SimpleRunner;

impl<Extra> TestRunner<Extra> for SimpleRunner {
    fn run<'t, 's, I, F>(
        &self,
        tests: I,
        _: &'s Scope<'s, 't>,
    ) -> impl Iterator<Item = (&'t TestMeta<Extra>, TestOutcome)>
    where
        I: ExactSizeIterator<Item = (F, &'t TestMeta<Extra>)>,
        F: (FnOnce() -> TestStatus) + Send + 's,
        Extra: 't,
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
    F: (FnOnce() -> TestStatus) + Send,
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
    F: (FnOnce() -> TestStatus) + Send + 's,
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
    F: (FnOnce() -> TestStatus) + Send + 's,
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
        F: (FnOnce() -> TestStatus) + Send + 's,
        Extra: 't,
    {
        let worker_count = <DefaultRunner as TestRunner<Extra>>::worker_count(self, tests.len());
        DefaultRunnerIterator::new(worker_count, tests, scope)
    }

    fn worker_count(&self, test_count: usize) -> NonZeroUsize {
        NonZeroUsize::new(cmp::min(self.threads.get(), test_count)).unwrap_or(NonZeroUsize::MIN)
    }
}

#[derive(Debug)]
pub struct SmartRunner {
    threshold: usize,
    simple: SimpleRunner,
    default: DefaultRunner,
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

enum SmartRunnerIterator<IS, ID> {
    Simple(IS),
    Default(ID),
}

impl<'t, IS, ID, Extra> Iterator for SmartRunnerIterator<IS, ID>
where
    IS: Iterator<Item = (&'t TestMeta<Extra>, TestOutcome)>,
    ID: Iterator<Item = (&'t TestMeta<Extra>, TestOutcome)>,
    Extra: 't,
{
    type Item = (&'t TestMeta<Extra>, TestOutcome);

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            SmartRunnerIterator::Simple(i) => i.next(),
            SmartRunnerIterator::Default(i) => i.next(),
        }
    }
}

impl<Extra: Sync> TestRunner<Extra> for SmartRunner {
    fn run<'t, 's, I, F>(
        &self,
        tests: I,
        scope: &'s Scope<'s, 't>,
    ) -> impl Iterator<Item = (&'t TestMeta<Extra>, TestOutcome)>
    where
        I: ExactSizeIterator<Item = (F, &'t TestMeta<Extra>)>,
        F: (FnOnce() -> TestStatus) + Send + 's,
        Extra: 't,
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
