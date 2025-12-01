use std::{num::NonZeroUsize, thread::Scope, time::Instant};

use crate::{
    capture::{
        CapturePanicHookGuard, DefaultPanicHookProvider, PanicHookProvider, TEST_OUTPUT_CAPTURE,
        TestOutputCapture,
    },
    outcome::{TestOutcome, TestOutcomeAttachments, TestStatus},
    runner::TestRunner,
    test::TestMeta,
};

#[derive(Debug)]
pub struct SimpleRunner<PanicHookProvider> {
    panic_hook_provider: PanicHookProvider,
}

impl Default for SimpleRunner<DefaultPanicHookProvider> {
    fn default() -> Self {
        Self {
            panic_hook_provider: DefaultPanicHookProvider,
        }
    }
}

impl<PanicHookProvider> SimpleRunner<PanicHookProvider> {
    pub fn new() -> SimpleRunner<DefaultPanicHookProvider> {
        SimpleRunner::default()
    }

    pub fn with_panic_hook_provider<WithPanicHookProvider>(
        self,
        panic_hook_provider: WithPanicHookProvider,
    ) -> SimpleRunner<WithPanicHookProvider> {
        SimpleRunner {
            panic_hook_provider,
        }
    }
}

impl<P, Extra> TestRunner<Extra> for SimpleRunner<P>
where
    P: PanicHookProvider,
{
    fn run<'t, 's, I, F>(
        &self,
        tests: I,
        _: &'s Scope<'s, 't>,
    ) -> impl Iterator<Item = (&'t TestMeta<Extra>, TestOutcome)>
    where
        I: ExactSizeIterator<Item = (F, &'t TestMeta<Extra>)>,
        F: (Fn() -> TestStatus) + Send + 's,
        Extra: 't,
    {
        let _panic_hook = CapturePanicHookGuard::install(self.panic_hook_provider.provide());
        tests.map(|(test, meta)| {
            let now = Instant::now();
            let status = test();
            let duration = now.elapsed();
            let TestOutputCapture { stdout, stderr } =
                TEST_OUTPUT_CAPTURE.with_borrow_mut(|capture| capture.take());

            let outcome = TestOutcome {
                status,
                duration,
                stdout,
                stderr,
                attachments: TestOutcomeAttachments::default(),
            };

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
