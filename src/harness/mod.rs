#![allow(clippy::type_complexity)]

use std::io;

use crate::{
    capture::DefaultPanicHookProvider,
    filter::DefaultFilter,
    formatter::{
        common::label::{FromGroupKey, GroupLabel},
        pretty::PrettyFormatter,
    },
    ignore::DefaultIgnore,
    panic::DefaultPanicHandler,
    runner::{DefaultRunner, scope::NoScopeFactory},
    test::Test,
};

mod test;
pub use test::TestHarness;

mod grouped_test;
pub use grouped_test::GroupedTestHarness;

/// Build a [`TestHarness`] from a list of tests.
///
/// This is the main entry point for running tests with Kitest.
/// It takes the full list of tests that *could* be executed and returns a harness configured with
/// Kitest's default strategies.
///
/// The returned harness can be customized by chaining `with_*` methods
/// (for example `with_filter`, `with_runner`, `with_formatter`).
/// You typically keep most defaults and replace only the parts you care about.
///
/// The `tests` slice is the complete universe of tests for this run.
/// The harness can filter and ignore tests, but it does not add new tests after creation.
/// The initial list can still be created in any way, including data driven discovery.
///
/// Calling [`with_grouper`](TestHarness::with_grouper) promotes the [`TestHarness`] into a
/// [`GroupedTestHarness`].
/// From that point on, tests are executed through groups instead of individually, and group
/// specific strategies can be configured.
///
/// `harness` works well with type inference: in normal usage you do not need to write out the full
/// `TestHarness<...>` type with all generic parameters. The compiler will infer it from the
/// strategies you keep or replace.
///
/// # Examples
///
/// ```
/// use std::process::Termination;
/// use kitest::prelude::*;
/// #
/// # fn collect_tests() -> Vec<Test> { vec![] }
///
/// fn main() -> impl Termination {
///     let tests = collect_tests();
///     kitest::harness(&tests)
///         .run()
///         .report()
/// }
/// ```
pub fn harness<'t, Extra>(
    tests: &'t [Test<Extra>],
) -> TestHarness<
    't,
    Extra,
    DefaultFilter,
    DefaultIgnore,
    DefaultPanicHandler,
    DefaultRunner<DefaultPanicHookProvider, NoScopeFactory>,
    PrettyFormatter<'t, io::Stdout, GroupLabel<FromGroupKey>, Extra>,
> {
    TestHarness {
        tests,
        filter: DefaultFilter::default(),
        ignore: DefaultIgnore::Default,
        panic_handler: DefaultPanicHandler,
        runner: DefaultRunner::default(),
        formatter: PrettyFormatter::default(),
    }
}

trait FmtErrors<E> {
    fn push_on_error<T>(&mut self, res: Result<T, E>);
}

impl<E> FmtErrors<E> for Vec<E> {
    fn push_on_error<T>(&mut self, res: Result<T, E>) {
        if let Err(err) = res {
            self.push(err);
        }
    }
}
