use std::{collections::HashMap, num::NonZeroUsize, thread, time::Instant};

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

struct DefaultRunnerIterator<'m, I, F, Extra>
where
    I: Iterator<Item = (F, &'m TestMeta<Extra>)> + Send,
    F: (Fn() -> TestStatus) + Send,
    Extra: 'm + Sync,
{
    source: I,
}

impl<'m, I, F, Extra> DefaultRunnerIterator<'m, I, F, Extra>
where
    I: Iterator<Item = (F, &'m TestMeta<Extra>)> + Send,
    F: (Fn() -> TestStatus) + Send,
    Extra: 'm + Sync,
{
    fn new(worker_count: NonZeroUsize, mut iter: I) -> Self {
        let (itx, irx) = crossbeam_channel::bounded(worker_count.into());
        let workers = (0..worker_count.get())
            .map(|_| {
                let irx = irx.clone();
                itx.send(iter.next());
                thread::spawn(|| {
                    while let Some((f, meta)) = irx.recv().expect("sender not disconnected") {

                    }
            })});

        todo!()
    }
}

impl<'m, I, F, Extra> Iterator for DefaultRunnerIterator<'m, I, F, Extra>
where
    I: Iterator<Item = (F, &'m TestMeta<Extra>)>,
    F: (Fn() -> TestStatus),
    Extra: 'm,
{
    type Item = (&'m str, TestOutcome);

    fn next(&mut self) -> Option<Self::Item> {
        todo!()
    }
}

impl<Extra> TestRunner<Extra> for DefaultRunner {
    fn run<'m, I, F>(&self, tests: I) -> impl Iterator<Item = (&'m str, TestOutcome)>
    where
        I: ExactSizeIterator<Item = (F, &'m TestMeta<Extra>)> + Send,
        F: (Fn() -> TestStatus) + Send,
        Extra: 'm + Sync,
    {
        todo!()
    }
}

// pub struct SmartRunner {
//     threshold: usize,
//     simple: SimpleRunner,
//     default: DefaultRunner,
// }

// impl SmartRunner {
//     pub fn new() -> Self {
//         Self {
//             threshold: 4,
//             simple: SimpleRunner,
//             default: DefaultRunner { thread_pool: None },
//         }
//     }

//     pub fn with_threshold(self, threshold: usize) -> Self {
//         Self { threshold, ..self }
//     }

//     pub fn with_threads(self, threads: usize) -> Result<Self, ThreadPoolBuildError> {
//         let thread_pool = ThreadPoolBuilder::new().num_threads(threads).build()?;
//         Ok(Self {
//             default: DefaultRunner {
//                 thread_pool: Some(thread_pool),
//             },
//             ..self
//         })
//     }

//     pub fn with_thread_pool(self, thread_pool: ThreadPool) -> Self {
//         Self {
//             default: DefaultRunner {
//                 thread_pool: Some(thread_pool),
//             },
//             ..self
//         }
//     }
// }

// impl Default for SmartRunner {
//     fn default() -> Self {
//         Self::new()
//     }
// }

// enum SmartRunnerIterator<IS, ID> {
//     Simple(IS),
//     Default(ID),
// }

// impl<'m, IS, ID> Iterator for SmartRunnerIterator<IS, ID>
// where
//     IS: Iterator<Item = (&'m str, TestOutcome)>,
//     ID: Iterator<Item = (&'m str, TestOutcome)>,
// {
//     type Item = (&'m str, TestOutcome);

//     fn next(&mut self) -> Option<Self::Item> {
//         match self {
//             SmartRunnerIterator::Simple(i) => i.next(),
//             SmartRunnerIterator::Default(i) => i.next(),
//         }
//     }
// }

// impl<Extra> TestRunner<Extra> for SmartRunner {
//     fn run<'m, I, F>(&self, tests: I) -> impl Iterator<Item = (&'m str, TestOutcome)>
//     where
//         I: ExactSizeIterator<Item = (F, &'m TestMeta<Extra>)> + Send,
//         F: (Fn() -> TestStatus) + Send,
//         Extra: 'm + Sync,
//     {
//         match tests.len() <= self.threshold {
//             true => SmartRunnerIterator::Simple(<SimpleRunner as TestRunner<Extra>>::run(
//                 &self.simple,
//                 tests,
//             )),
//             false => SmartRunnerIterator::Default(<DefaultRunner as TestRunner<Extra>>::run(
//                 &self.default,
//                 tests,
//             )),
//         }
//     }
// }
