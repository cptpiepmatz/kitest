use std::{collections::HashMap, time::Instant};

use rayon::{
    ThreadPool, ThreadPoolBuildError, ThreadPoolBuilder,
    iter::{ParallelBridge, ParallelIterator},
};

use crate::{
    meta::TestMeta,
    outcome::{TestOutcome, TestOutcomeAttachments, TestStatus},
};

pub trait TestRunner<Extra> {
    fn run<'m, I, F>(&self, tests: I) -> impl Iterator<Item = (&'m str, TestOutcome)>
    where
        I: ExactSizeIterator<Item = (F, &'m TestMeta<Extra>)> + Send,
        F: (Fn() -> TestStatus) + Send,
        Extra: 'm + Sync;
}

#[derive(Default)]
pub struct SimpleRunner;

impl<Extra> TestRunner<Extra> for SimpleRunner {
    fn run<'m, I, F>(&self, tests: I) -> impl Iterator<Item = (&'m str, TestOutcome)>
    where
        I: ExactSizeIterator<Item = (F, &'m TestMeta<Extra>)> + Send,
        F: (Fn() -> TestStatus) + Send,
        Extra: 'm + Sync,
    {
        tests.map(|(test, meta)| {
            let now = Instant::now();
            let status = test();
            let duration = now.elapsed();
            (
                meta.name.as_ref(),
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
}

pub struct DefaultRunner {
    thread_pool: Option<ThreadPool>,
}

impl DefaultRunner {
    pub fn new() -> Self {
        Self { thread_pool: None }
    }

    pub fn with_threads(self, threads: usize) -> Result<Self, ThreadPoolBuildError> {
        let thread_pool = ThreadPoolBuilder::new().num_threads(threads).build()?;
        Ok(Self {
            thread_pool: Some(thread_pool),
        })
    }

    pub fn with_thread_pool(self, thread_pool: ThreadPool) -> Self {
        Self {
            thread_pool: Some(thread_pool),
        }
    }
}

impl Default for DefaultRunner {
    fn default() -> Self {
        Self::new()
    }
}

impl<Extra> TestRunner<Extra> for DefaultRunner {
    fn run<'m, I, F>(&self, tests: I) -> impl Iterator<Item = (&'m str, TestOutcome)>
    where
        I: ExactSizeIterator<Item = (F, &'m TestMeta<Extra>)> + Send,
        F: (Fn() -> TestStatus) + Send,
        Extra: 'm + Sync,
    {
        let run = || {
            rayon_par_bridge::par_bridge(
                1,
                tests.par_bridge().map(|(test, meta)| {
                    let now = Instant::now();
                    let status = test();
                    let duration = now.elapsed();
                    (
                        meta.name.as_ref(),
                        TestOutcome {
                            status,
                            duration,
                            stdout: Vec::new(),
                            stderr: Vec::new(),
                            attachments: TestOutcomeAttachments::default(),
                        },
                    )
                }),
                |i| i.into_iter(),
            )
        };

        match &self.thread_pool {
            Some(thread_pool) => thread_pool.install(run),
            None => run(),
        }
    }
}

pub struct SmartRunner {
    threshold: usize,
    simple: SimpleRunner,
    default: DefaultRunner,
}

impl SmartRunner {
    pub fn new() -> Self {
        Self {
            threshold: 4,
            simple: SimpleRunner,
            default: DefaultRunner { thread_pool: None },
        }
    }

    pub fn with_threshold(self, threshold: usize) -> Self {
        Self { threshold, ..self }
    }

    pub fn with_threads(self, threads: usize) -> Result<Self, ThreadPoolBuildError> {
        let thread_pool = ThreadPoolBuilder::new().num_threads(threads).build()?;
        Ok(Self {
            default: DefaultRunner {
                thread_pool: Some(thread_pool),
            },
            ..self
        })
    }

    pub fn with_thread_pool(self, thread_pool: ThreadPool) -> Self {
        Self {
            default: DefaultRunner {
                thread_pool: Some(thread_pool),
            },
            ..self
        }
    }
}

impl Default for SmartRunner {
    fn default() -> Self {
        Self::new()
    }
}

enum SmartRunnerIterator<IS, ID> {
    Simple(IS),
    Default(ID),
}

impl<'m, IS, ID> Iterator for SmartRunnerIterator<IS, ID>
where
    IS: Iterator<Item = (&'m str, TestOutcome)>,
    ID: Iterator<Item = (&'m str, TestOutcome)>,
{
    type Item = (&'m str, TestOutcome);

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            SmartRunnerIterator::Simple(i) => i.next(),
            SmartRunnerIterator::Default(i) => i.next(),
        }
    }
}

impl<Extra> TestRunner<Extra> for SmartRunner {
    fn run<'m, I, F>(&self, tests: I) -> impl Iterator<Item = (&'m str, TestOutcome)>
    where
        I: ExactSizeIterator<Item = (F, &'m TestMeta<Extra>)> + Send,
        F: (Fn() -> TestStatus) + Send,
        Extra: 'm + Sync,
    {
        match tests.len() <= self.threshold {
            true => SmartRunnerIterator::Simple(<SimpleRunner as TestRunner<Extra>>::run(
                &self.simple,
                tests,
            )),
            false => SmartRunnerIterator::Default(<DefaultRunner as TestRunner<Extra>>::run(
                &self.default,
                tests,
            )),
        }
    }
}
