use std::{num::NonZeroUsize, thread::Scope};

use crate::{
    capture::{DefaultPanicHookProvider, PanicHookProvider},
    outcome::{TestOutcome, TestStatus},
    runner::{DefaultRunner, SimpleRunner, TestRunner},
    test::TestMeta,
};

/// A [`TestRunner`] that picks between [`SimpleRunner`] and [`DefaultRunner`].
///
/// If the number of tests is at or below `threshold`, this runner uses
/// [`SimpleRunner`] (single threaded, in order).
/// Otherwise it uses [`DefaultRunner`] (worker pool).
///
/// This is useful for grouped runs where some groups may be very small.
/// For small batches it can be cheaper to run on the current thread than to
/// pay the overhead of scheduling work across threads.
/// The best choice depends on the workload, which is why this is not the default runner
/// implementation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SmartRunner<PanicHookProvider> {
    threshold: usize,
    simple: SimpleRunner<PanicHookProvider>,
    default: DefaultRunner<PanicHookProvider>,
}

impl Default for SmartRunner<DefaultPanicHookProvider> {
    fn default() -> Self {
        Self {
            threshold: 4,
            simple: SimpleRunner::default(),
            default: DefaultRunner::default(),
        }
    }
}

impl<P> SmartRunner<P> {
    /// Create a smart runner using the default panic hook provider.
    ///
    /// This is the same as `SmartRunner::default()`.
    pub fn new() -> SmartRunner<DefaultPanicHookProvider> {
        SmartRunner::default()
    }

    /// Set the maximum test count that will still use [`SimpleRunner`].
    ///
    /// This replaces the previous threshold.
    pub fn with_threshold(self, threshold: usize) -> Self {
        Self { threshold, ..self }
    }

    /// Override the worker thread count used by the internal [`DefaultRunner`].
    pub fn with_threads(mut self, threads: NonZeroUsize) -> Self {
        self.default = self.default.with_thread_count(threads);
        self
    }

    /// Replace the panic hook provider used for output capture.
    ///
    /// The provider is applied to both the internal [`SimpleRunner`] and
    /// [`DefaultRunner`].
    pub fn with_panic_hook_provider<WithPanicHookProvider: Clone>(
        self,
        panic_hook_provider: WithPanicHookProvider,
    ) -> SmartRunner<WithPanicHookProvider> {
        SmartRunner {
            threshold: self.threshold,
            simple: self
                .simple
                .with_panic_hook_provider(panic_hook_provider.clone()),
            default: self.default.with_panic_hook_provider(panic_hook_provider),
        }
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

impl<P: PanicHookProvider, Extra: Sync> TestRunner<Extra> for SmartRunner<P> {
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
            true => SmartRunnerIterator::Simple(<SimpleRunner<_> as TestRunner<Extra>>::run(
                &self.simple,
                tests,
                scope,
            )),
            false => SmartRunnerIterator::Default(<DefaultRunner<_> as TestRunner<Extra>>::run(
                &self.default,
                tests,
                scope,
            )),
        }
    }

    fn worker_count(&self, test_count: usize) -> NonZeroUsize {
        match test_count <= self.threshold {
            true => <SimpleRunner<_> as TestRunner<Extra>>::worker_count(&self.simple, test_count),
            false => {
                <DefaultRunner<_> as TestRunner<Extra>>::worker_count(&self.default, test_count)
            }
        }
    }
}
