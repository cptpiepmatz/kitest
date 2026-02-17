use std::{num::NonZeroUsize, thread::Scope, time::Instant};

use crate::{
    capture::{
        CapturePanicHookGuard, DefaultPanicHookProvider, OutputCapture, PanicHookProvider,
        TEST_OUTPUT_CAPTURE,
    },
    outcome::{TestOutcome, TestOutcomeAttachments, TestStatus},
    runner::{
        TestRunner,
        scope::{NoScopeFactory, TestScope, TestScopeFactory},
    },
    test::TestMeta,
};

/// A simple [`TestRunner`] that runs tests on the current thread.
///
/// This is a simpler alternative to [`DefaultRunner`](super::DefaultRunner).
/// It executes tests sequentially, in the order they are provided, and does not spawn any extra
/// threads.
///
/// This is handy in tests and other situations where deterministic ordering is
/// useful, while still keeping the same behavior around timing and output capture.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SimpleRunner<PanicHookProvider, TestScopeFactory> {
    panic_hook_provider: PanicHookProvider,
    test_scope_factory: TestScopeFactory,
}

impl Default for SimpleRunner<DefaultPanicHookProvider, NoScopeFactory> {
    fn default() -> Self {
        Self {
            panic_hook_provider: DefaultPanicHookProvider,
            test_scope_factory: NoScopeFactory,
        }
    }
}

impl<PanicHookProvider, TestScopeFactory> SimpleRunner<PanicHookProvider, TestScopeFactory> {
    /// Create a simple runner using the default panic hook provider.
    ///
    /// This is the same as `SimpleRunner::default()`.
    pub fn new() -> SimpleRunner<DefaultPanicHookProvider, NoScopeFactory> {
        SimpleRunner::default()
    }

    /// Replace the panic hook provider used for output capture.
    pub fn with_panic_hook_provider<WithPanicHookProvider>(
        self,
        panic_hook_provider: WithPanicHookProvider,
    ) -> SimpleRunner<WithPanicHookProvider, TestScopeFactory> {
        SimpleRunner {
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
    ) -> SimpleRunner<PanicHookProvider, WithTestScopeFactory> {
        SimpleRunner {
            panic_hook_provider: self.panic_hook_provider,
            test_scope_factory,
        }
    }
}

impl<'t, P, T, Extra> TestRunner<'t, Extra> for SimpleRunner<P, T>
where
    P: PanicHookProvider,
    T: TestScopeFactory<'t, Extra>,
{
    fn run<'s, I, F>(
        &self,
        tests: I,
        _: &'s Scope<'s, 't>,
    ) -> impl Iterator<Item = (&'t TestMeta<Extra>, TestOutcome)>
    where
        I: ExactSizeIterator<Item = (F, &'t TestMeta<Extra>)>,
        F: (Fn() -> TestStatus) + Send + 's,
        Extra: 't,
    {
        let panic_hook = CapturePanicHookGuard::install(self.panic_hook_provider.provide());
        tests.map(move |(test, meta)| {
            // keep a ref in here so that the panic_hook only gets dropped after this iterator is done
            let _panic_hook = &panic_hook;

            let mut test_scope = self.test_scope_factory.make_scope();
            test_scope.before_test(meta);

            let now = Instant::now();
            let status = test();
            let duration = now.elapsed();
            let output = TEST_OUTPUT_CAPTURE.with_borrow_mut(OutputCapture::take);

            let outcome = TestOutcome {
                status,
                duration,
                output,
                attachments: TestOutcomeAttachments::default(),
            };

            test_scope.after_test(meta, &outcome);
            (meta, outcome)
        })
    }

    fn worker_count(&self, _: usize) -> NonZeroUsize {
        const { NonZeroUsize::new(1).unwrap() }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::*;

    #[test]
    fn run_all_ok_tests() {
        let tests = &[test! {}, test! {}, test! {}];

        let report = harness(tests).with_runner(SimpleRunner::default()).run();
        assert_eq!(report.outcomes.len(), tests.len());
    }

    #[test]
    fn run_linear() {
        let tests = &[
            test! {name: "first"},
            test! {name: "second"},
            test! {name: "third"},
        ];

        let report = harness(tests).with_runner(SimpleRunner::default()).run();
        let test_names: Vec<_> = report.outcomes.into_iter().map(|(key, _)| key).collect();
        let [first, second, third] = test_names.as_slice() else {
            panic!("invalid amount of test outcomes")
        };

        assert_eq!(*first, "first");
        assert_eq!(*second, "second");
        assert_eq!(*third, "third");
    }
}
