use std::{num::NonZeroUsize, thread::Scope};

use crate::{outcome::{TestOutcome, TestStatus}, runner::{DefaultRunner, SimpleRunner, TestRunner}, test::TestMeta};

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
        self.default = self.default.with_thread_count(threads);
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
        F: (Fn() -> TestStatus) + Send + 's,
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
