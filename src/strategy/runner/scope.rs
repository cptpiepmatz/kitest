//! Per test scoping hooks for runners.
//!
//! This module defines a small abstraction for running lifecycle hooks before
//! and after each test, without having to replace the entire runner
//! implementation.

use crate::{outcome::TestOutcome, test::TestMeta};

/// Per test lifecycle hooks for runners.
///
/// Test scoping allows adding "before test" and "after test" hooks to runners
/// like [`DefaultRunner`](super::DefaultRunner) and [`SimpleRunner`](super::SimpleRunner) without
/// having to replace the entire runner implementation.
///
/// A scope instance is created for a single test and is used for both the
/// [`before_test`](Self::before_test) and [`after_test`](Self::after_test) call of that test.
/// The provided [`TestMeta`] can be used to prepare the environment in a precise way
/// (for example based on the test name or extra metadata).
///
/// If a scope needs to provide data to the test body, prefer using thread locals
/// ([`thread_local!`](std::thread_local)).
/// This keeps data isolated per worker thread and avoids races when multiple tests run at the
/// same time.
pub trait TestScope<'t, Extra> {
    /// Called right before the test is executed.
    fn before_test(&mut self, meta: &'t TestMeta<Extra>) {
        let _ = meta;
    }

    /// Called right after the test finished executing.
    fn after_test(&mut self, meta: &'t TestMeta<Extra>, outcome: &TestOutcome) {
        let _ = (meta, outcome);
    }
}

/// Factory for creating [`TestScope`] instances.
///
/// The factory is used to create one scope instance per test. The same scope
/// instance is then used for both `before_test` and `after_test`.
pub trait TestScopeFactory<'t, Extra> {
    /// The scope type produced by this factory.
    ///
    /// The returned scope may borrow from the factory.
    type Scope<'f>: TestScope<'t, Extra> + 'f
    where
        't: 'f,
        Self: 'f;

    /// Create a new scope instance for a single test.
    fn make_scope<'f>(&'f self) -> Self::Scope<'f>
    where
        't: 'f;
}

/// A [`TestScope`] implementation that performs no work.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct NoScope;

impl<'t, Extra> TestScope<'t, Extra> for NoScope {}

/// A [`TestScopeFactory`] that always produces [`NoScope`].
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct NoScopeFactory;

impl<'t, Extra> TestScopeFactory<'t, Extra> for NoScopeFactory {
    type Scope<'f>
        = NoScope
    where
        't: 'f,
        Self: 'f;

    fn make_scope<'f>(&'f self) -> Self::Scope<'f>
    where
        't: 'f,
    {
        NoScope
    }
}
