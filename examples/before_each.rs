use std::{cell::RefCell, num::NonZeroUsize, process::Termination};

use kitest::{
    prelude::*,
    runner::{
        DefaultRunner,
        scope::{TestScope, TestScopeFactory},
    },
};

thread_local! {
    static BUF: RefCell<Vec<u32>> = const { RefCell::new(Vec::new()) };
}

fn tests() -> Vec<Test> {
    Vec::from_iter((0..1).map(|n| {
        Test::new(
            TestFnHandle::Owned(Box::new(move || {
                BUF.with_borrow_mut(|buf| {
                    assert!(buf.is_empty());
                    buf.push(n);
                });
            })),
            TestMeta {
                name: format!("push_{n}").into(),
                ignore: IgnoreStatus::Run,
                should_panic: PanicExpectation::ShouldNotPanic,
                origin: origin!(),
                extra: (),
            },
        )
    }))
}

struct ClearBuf;

impl<'t> TestScopeFactory<'t, ()> for ClearBuf {
    type Scope<'f>
        = Self
    where
        't: 'f,
        Self: 'f;

    fn make_scope<'f>(&'f self) -> Self::Scope<'f>
    where
        't: 'f,
    {
        ClearBuf
    }
}

impl<'t> TestScope<'t, ()> for ClearBuf {
    fn before_test(&mut self, _: &'t TestMeta<()>) {
        BUF.with_borrow_mut(|buf| buf.clear());
    }
}

fn main() -> impl Termination {
    let tests = tests().leak();

    let runner = DefaultRunner::default()
        .with_thread_count(const { NonZeroUsize::new(2).unwrap() })
        .with_test_scope_factory(ClearBuf);

    kitest::harness(tests).with_runner(runner).run()
}
