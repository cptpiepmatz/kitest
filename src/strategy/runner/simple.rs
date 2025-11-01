use std::{num::NonZeroUsize, thread::Scope, time::Instant};

use crate::{
    outcome::{TestOutcome, TestOutcomeAttachments, TestStatus},
    runner::TestRunner,
    test::TestMeta,
};

#[derive(Debug, Default)]
pub struct SimpleRunner {
    pub keep_going: bool,
}

impl<Extra> TestRunner<Extra> for SimpleRunner {
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
        tests
            .map(|(test, meta)| {
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
            .scan(true, |keep, (meta, outcome)| {
                if !self.keep_going && !*keep {
                    return None;
                };
                *keep = outcome.passed();
                Some((meta, outcome))
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
    fn abort_after_failed_test() {
        let tests = &[
            test! {name: "ok"},
            test! {name: "fail", func: || Err(())},
            test! {name: "never"},
        ];

        let report = harness(tests)
            .with_runner(SimpleRunner { keep_going: false })
            .run();
        assert_eq!(report.outcomes.len(), 2);
    }

    #[test]
    fn keep_going_after_failed_test() {
        let tests = &[
            test! {name: "ok"},
            test! {name: "fail", func: || Err(())},
            test! {name: "still here"},
        ];

        let report = harness(tests)
            .with_runner(SimpleRunner { keep_going: true })
            .run();
        assert_eq!(report.outcomes.len(), 3);
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
