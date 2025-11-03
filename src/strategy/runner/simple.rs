use std::{num::NonZeroUsize, ops::ControlFlow, thread::Scope, time::Instant};

use crate::{
    outcome::{TestOutcome, TestOutcomeAttachments, TestStatus},
    runner::TestRunner,
    test::TestMeta,
    util::IteratorExt,
};

#[derive(Debug, Default)]
pub struct SimpleRunner {
    pub keep_going: bool,
}

impl SimpleRunner {
    pub fn with_keep_going(self, keep_going: bool) -> Self {
        Self { keep_going }
    }
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
        tests.map_until_inclusive(|(test, meta)| {
            let now = Instant::now();
            let status = test();
            let duration = now.elapsed();

            let outcome = TestOutcome {
                status,
                duration,
                stdout: Vec::new(),
                stderr: Vec::new(),
                attachments: TestOutcomeAttachments::default(),
            };

            match (self.keep_going, outcome.is_bad()) {
                (false, true) => ControlFlow::Break((meta, outcome)),
                (true, _) | (false, false) => ControlFlow::Continue((meta, outcome)),
            }
        })
    }

    fn worker_count(&self, _: usize) -> NonZeroUsize {
        const { NonZeroUsize::new(1).unwrap() }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ignore::DefaultIgnore, test_support::*};

    #[test]
    fn run_all_ok_tests() {
        let tests = &[test! {}, test! {}, test! {}];

        let report = harness(tests).with_runner(SimpleRunner::default()).run();
        assert_eq!(report.outcomes.len(), tests.len());
    }

    #[test]
    fn abort_only_after_failed_test() {
        let tests = &[
            test! {name: "ok"},
            test! {name: "ignored", ignore: true},
            test! {name: "fail", func: || Err(())},
            test! {name: "never"},
        ];

        let report = harness(tests)
            .with_ignore(DefaultIgnore::default())
            .with_runner(SimpleRunner { keep_going: false })
            .run();
        assert_eq!(report.outcomes.len(), 3);
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
