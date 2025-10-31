use std::{num::NonZeroUsize, thread::Scope, time::Instant};

use crate::{outcome::{TestOutcome, TestOutcomeAttachments, TestStatus}, runner::TestRunner, test::TestMeta};

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
        F: (Fn() -> TestStatus) + Send + 's,
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
