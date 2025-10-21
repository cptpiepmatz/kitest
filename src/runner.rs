use std::{
    collections::HashMap,
    num::NonZeroUsize,
    thread::{self, Scope},
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
    ) -> impl Iterator<Item = (&'m str, TestOutcome)>
    where
        I: ExactSizeIterator<Item = (F, &'m TestMeta<Extra>)> + Send,
        F: (Fn() -> TestStatus) + Send,
        Extra: 'm + Sync,
        'm: 's;
}

// #[derive(Default)]
// pub struct SimpleRunner;

// impl<Extra> TestRunner<Extra> for SimpleRunner {
//     fn run<'m, I, F>(
//         &self,
//         tests: I,
//         _: &Scope<'_, 'm>,
//     ) -> impl Iterator<Item = (&'m str, TestOutcome)>
//     where
//         I: ExactSizeIterator<Item = (F, &'m TestMeta<Extra>)> + Send,
//         F: (Fn() -> TestStatus) + Send,
//         Extra: 'm + Sync,
//     {
//         tests.map(|(test, meta)| {
//             let now = Instant::now();
//             let status = test();
//             let duration = now.elapsed();
//             (
//                 meta.name.as_ref(),
//                 TestOutcome {
//                     status,
//                     duration,
//                     stdout: Vec::new(),
//                     stderr: Vec::new(),
//                     attachments: TestOutcomeAttachments::default(),
//                 },
//             )
//         })
//     }
// }

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
    F: (Fn() -> TestStatus) + Send,
    Extra: 'm + Sync,
    'm: 's,
{
    source: I,
    scope: &'s Scope<'s, 'm>,
    push_job: Sender<Option<(F, &'m TestMeta<Extra>)>>,
}

impl<'m, 's, I, F, Extra> DefaultRunnerIterator<'m, 's, I, F, Extra>
where
    I: Iterator<Item = (F, &'m TestMeta<Extra>)> + Send,
    F: (Fn() -> TestStatus) + Send + 's,
    Extra: 'm + Sync,
{
    fn new(worker_count: NonZeroUsize, mut iter: I, scope: &'s Scope<'s, 'm>) -> Self {
        let (itx, irx) = crossbeam_channel::bounded(worker_count.into());
        let (otx, orx) = crossbeam_channel::bounded(1);
        let workers = (0..worker_count.get())
            .map(|_| {
                let irx = irx.clone();
                let otx = otx.clone();
                itx.send(iter.next());
                scope.spawn(move || {
                    while let Ok(Some((f, meta))) = irx.recv() {
                        let now = Instant::now();
                        let status = f();
                        let duration = now.elapsed();
                        otx.send((meta.name.as_ref(), TestOutcome {
                            status,
                            duration,
                            stdout: Vec::new(),
                            stderr: Vec::new(),
                            attachments: TestOutcomeAttachments::default(),
                        }));
                    }
            })});

        todo!()
    }
}

// impl<'m, I, F, Extra> Iterator for DefaultRunnerIterator<'m, I, F, Extra>
// where
//     I: Iterator<Item = (F, &'m TestMeta<Extra>)>,
//     F: (Fn() -> TestStatus),
//     Extra: 'm,
// {
//     type Item = (&'m str, TestOutcome);

//     fn next(&mut self) -> Option<Self::Item> {
//         todo!()
//     }
// }

impl<Extra> TestRunner<Extra> for DefaultRunner {
    fn run<'m, I, F>(
        &self,
        tests: I,
        scope: &Scope<'_, 'm>,
    ) -> impl Iterator<Item = (&'m str, TestOutcome)>
    where
        I: ExactSizeIterator<Item = (F, &'m TestMeta<Extra>)> + Send,
        F: (Fn() -> TestStatus) + Send,
        Extra: 'm + Sync,
    {
        // TODO: proper iterator here
        std::iter::empty()
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
