#![doc(html_logo_url = "https://raw.githubusercontent.com/cptpiepmatz/kitest/main/logo/logo.svg")]

//! <div style="float: right">
//!     <img width="200em" src="https://raw.githubusercontent.com/cptpiepmatz/kitest/main/logo/logo.svg" alt="logo">
//! </div>
//!
//! # Kitest
//!
//! Kitest is a toolkit to build your own test harness for `cargo test`.
//!
//! It provides everything you expect from the built-in Rust test harness out of the box,
//! while allowing you to replace or customize each part independently.
//! You only touch the pieces you care about and leave the rest to Kitest.
//!
//! With a bit of macro machinery, Kitest can act as a drop in replacement for the default test harness.
//! At the same time, it enables testing setups that go beyond what the built-in harness
//! supports.
//!
//! Kitest is designed as a foundation.
//! It is not meant to enforce a single testing style, but to let you build one that fits your needs.
//! Other crates can be built on top of it to provide fully packaged test harnesses or reusable components.
//!
//! ## What it looks like
//!
//! A regular test run using the default harness and formatter:
//!
#![doc = include_str!("../doc/html/regular.ansi.html")]
//!
//! Grouped tests with shared setup and teardown:
//!
#![doc = include_str!("../doc/html/group_by_flag.ansi.html")]
//!
//! ## What Kitest enables
//!
//! - A test harness comparable to the built-in one, ready to use by default
//! - Data driven testing, tests do not need to originate from Rust code
//! - Grouping tests and running them with shared preparation and teardown
//! - Preparing and tearing down the entire test suite, not just individual tests
//! - Replacing individual parts of the harness without rewriting everything
//!
//! Compared to other solutions:
//! - Unlike [`libtest-mimic`](https://crates.io/crates/libtest-mimic), every part of the harness
//!   can be replaced independently
//! - Unlike [`rstest`](https://crates.io/crates/rstest), preparation and teardown are not limited
//!   to individual tests
//!
//! ## When should I use Kitest?
//!
//! Kitest is a good fit if the built-in Rust test harness starts getting in your way.
//!
//! You might want to use Kitest if:
//!
//! - You want to customize how tests are discovered, filtered, or executed
//! - You need test suite level setup and teardown
//! - You want to group tests and run them with shared state or resources
//! - Your tests are data driven or come from non-Rust sources
//! - You want full control over test output and formatting
//!
//! Kitest is also useful if you are building tooling on top of Rust testing.
//! It provides the building blocks needed to construct higher level test frameworks without having
//! to reimplement a full harness from scratch.
//!
//! You may not need Kitest if:
//!
//! - The built-in test harness already does everything you need
//! - You only need per test setup and teardown, then `rstest` should be enough
//! - Your test setup heavily diverges from the typical behavior of `cargo test`
//!
//! Kitest focuses on flexibility and composability.
//! If you want a single opinionated testing style out of the box, a higher level framework built on
//! top of Kitest may be a better fit.
//!
//! ## Build your own test harness
//!
//! The main entry point is [`harness`]. It takes a list of tests and returns a [`TestHarness`]
//! configured with a set of default strategies. From there, we can swap out individual parts
//! by chaining setter calls on the harness.
//!
//! ### Tests and metadata
//!
//! A test is represented by [`Test<Extra>`](test::Test).
//! It combines two things:
//!
//! - The test function itself (how the test is executed)
//! - The test metadata stored in [`TestMeta<Extra>`](test::TestMeta)
//!
//! [`TestMeta<Extra>`](test::TestMeta) contains the things a harness typically needs to know about
//! a test, like its name, whether it is ignored (optionally with a reason), and whether it is
//! expected to panic.
//!
//! It also carries an `extra` field of type `Extra` for user defined metadata.
//! `Extra` is not a boxed trait object and it is not erased at runtime.
//! It is compiled statically, which keeps things fast and keeps the types precise.
//! Nearly everything in Kitest is generic over `Extra`, so our strategies can depend on metadata
//! however we like, without needing up- or downcasting.
//!
//! This means Kitest does not decide what "tags", "flags", or "categories" mean.
//! You decide that, and your harness strategies can use it directly.
//!
//! ### The test lifetime `'t`
//!
//! Most harness types are generic over a lifetime called `'t`.
//! This is the lifetime of the tests passed into [`harness`].
//! It is a core part of the design.
//!
//! `'t` is threaded through the harness and its strategies so that everything is allowed to borrow
//! from the original test list.
//! As long as a strategy only needs to look at tests or metadata, it can borrow instead of copying.
//! This avoids a lot of unnecessary allocations and makes it cheap to build higher level logic
//! around the test list.
//!
//! In other words, `'t` is "the lifetime of the tests", and all harness components effectively live
//! inside that boundary.
//!
//! The harness still expects the full list of tests up front.
//! After that, the harness can:
//!
//! - filter tests (remove them from the run)
//! - ignore tests (keep them in the list, but do not execute them)
//!
//! But it cannot add new tests dynamically once the harness is created.
//!
//! This does not mean tests must be hardcoded.
//! The initial list can be created from anything:
//! reading files, scanning directories, querying a server, generating cases, and so on.
//!
//! ### Harness strategies
//!
//! A [`TestHarness`] is composed of multiple strategies.
//! Each strategy is responsible for one part of how tests are handled.
//! We can keep the defaults and replace only what we need.
//!
//! The default harness is assembled roughly like this:
//!
//! - **[Filter](filter::TestFilter)**: decides which tests participate in the run at all (for example, by name)
//! - **[Ignore](ignore::TestIgnore)**: decides whether a participating test is executed or reported as ignored
//! - **[Panic handler](panic::TestPanicHandler)**: executes the test and converts panics into a test status
//! - **[Runner](runner::TestRunner)**: schedules tests, usually in parallel, and collects outcomes
//! - **[Formatter](formatter::TestFormatter)**: prints progress and results in a `cargo test` style format
//!
//! If you want to group tests, you can explicitly opt into grouping by calling
//! [`with_grouper`](TestHarness::with_grouper) on a [`TestHarness`].
//! This turns it into a [`GroupedTestHarness`].
//!
//! From that point on, grouping-specific strategies become available and can be configured
//! independently from the non-grouped harness.
//!
//! A grouped harness adds a few more strategy points:
//!
//! - **[Grouper](group::TestGrouper)**: assigns tests to groups and can attach per-group context
//! - **[Group runner](group::TestGroupRunner)**: runs groups and can stop early depending on the group's outcome
//! - **[Grouped formatter](formatter::GroupedTestFormatter)**: prints group level events and outcomes
//!
//! This explicit transition keeps the basic harness simple, while making grouping a deliberate
//! choice.
//! Once grouping is enabled, all test execution happens through groups, which makes it
//! possible to share setup and teardown logic and control execution flow at the group level.
//!
//! ## Example
//!
//! This example shows how to build a custom test harness for your use case.
//!
//! First, Cargo needs to be told not to use the built-in test harness.
//! For unit tests, this can be done like this:
//!
#![doc = include_str!("../doc/html/lib.ansi.html")]
//!
//! And for integration tests like this:
//!
#![doc = include_str!("../doc/html/test.ansi.html")]
//!
//! By setting `harness` to `false`, we tell Cargo to skip the built-in harness.
//! Instead, it expects a custom `main` function that runs the tests.
//!
//! ```
//! use std::process::Termination;
//! use kitest::prelude::*;
//! #
//! # type MyFilter = kitest::filter::NoFilter;
//! # type MyRunner = kitest::runner::SimpleRunner<
//! #     kitest::capture::DefaultPanicHookProvider,
//! #     kitest::runner::scope::NoScopeFactory
//! # >;
//!
//! fn main() -> impl Termination {
//!     // However you collect your tests. This can be static or data driven.
//!     # let collect_tests = || Vec::<Test>::new();
//!     let tests = collect_tests();
//!
//!     // At this point, you could parse command line arguments and
//!     // construct different harnesses depending on them.
//!     // The harness also supports a list mode via `list`.
//!
//!     kitest::harness(&tests)
//!         .with_filter(MyFilter::new())
//!         .with_runner(MyRunner::new())
//!         .run()
//!         .report()
//! }
//! ```
//!
//! ## Argument parsing and configuration
//!
//! Kitest does not provide argument parsing.
//!
//! Crates like [`clap`](https://crates.io/crates/clap),
//! [`lexopt`](https://crates.io/crates/lexopt),
//! [`bpaf`](https://crates.io/crates/bpaf),
//! or [`argh`](https://crates.io/crates/argh)
//! already handle command line parsing well and integrate naturally with a custom `main` function.
//!
//! The intended workflow is:
//!
//! 1. Parse command line arguments in `main`.
//! 3. Collect tests.
//! 4. Decide which harness configuration to build.
//! 5. Run that harness.
//!
//! Depending on the parsed arguments, you might:
//!
//! - Switch between different formatters
//! - Enable or disable grouping
//! - Run in list mode instead of executing tests
//! - Choose different filters or runners
//!
//! Kitest is optimized for compiling a concrete harness configuration.
//! Strategy types are part of the harness type, which keeps everything static and efficient.
//! This also means strategy implementations are not swapped dynamically at runtime.
//!
//! If different configurations are possible, define them explicitly and select one
//! based on the parsed arguments:
//!
//! ```
//! # use kitest::prelude::*;
//! # use std::process::Termination;
//! #
//! # enum Mode { Run, List }
//! # struct Args { mode: Mode }
//! # fn parse() -> Args { Args { mode: Mode::Run } }
//! # fn collect() -> Vec<Test> { Vec::new() }
//! #
//! let args = parse();
//! let tests = collect();
//!
//! match args.mode {
//!     Mode::Run => {
//!         kitest::harness(&tests).run().report();
//!     },
//!     Mode::List => {
//!         kitest::harness(&tests).list();
//!     }
//! }
//! ```
//!
//! Argument parsing lives outside of Kitest.
//! Kitest focuses only on building and running test harnesses.

pub mod capture;
pub mod formatter;
pub mod outcome;
pub mod test;

mod strategy;
pub use strategy::*;

mod harness;
pub use harness::*;

mod report;
pub use report::*;

mod whatever;
pub use whatever::*;

/// Prelude containing everything you need to build a [`Test`](test::Test).
pub mod prelude {
    #[doc(no_inline)]
    pub use super::{
        ignore::IgnoreStatus,
        origin,
        panic::PanicExpectation,
        test::{Test, TestFn, TestFnHandle, TestMeta, TestOrigin, TestResult},
    };

    #[doc(no_inline)]
    pub use std::borrow::Cow;
}

mod util;

#[cfg(any(test, doctest))]
mod test_support;
