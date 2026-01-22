//! Test definitions.
//!
//! This module defines what a test is in Kitest.
//!
//! The central type is [`Test`]. It represents a single executable test together with
//! its associated metadata. All test harnesses in Kitest operate on collections of
//! [`Test`] values.

use std::{
    borrow::Cow,
    fmt::{Debug, Display},
    ops::Deref,
    panic::RefUnwindSafe,
};

use crate::{Whatever, ignore::IgnoreStatus, panic::PanicExpectation};

/// A single test case.
///
/// [`Test`] is the main test apparatus in Kitest. This is the type a harness operates on.
/// A harness takes a slice of tests and uses their metadata and execution handle to run,
/// list, filter, ignore, group, and report them.
///
/// A `Test` consists of:
///
/// - a [`TestFnHandle`] that knows how to execute the test
/// - a [`TestMeta<Extra>`] value that describes the test
///
/// ## Use case specific metadata with `Extra`
///
/// The `Extra` type parameter is user defined metadata attached to the test.
/// Nearly everything in Kitest is generic over `Extra`, so strategies can use this data without
/// runtime casts.
///
/// All default strategies work with any `Extra` type.
/// Custom strategies may choose to require a specific `Extra` type, for example a flag enum or a
/// struct with tags.
///
/// If no custom metadata is needed, `Extra` can be `()`.
/// Kitest treats this as the "no extra data" case, which keeps types easy to write.
///
/// ## Const friendly
///
/// A `Test` can be created at compile time.
/// This makes it possible to define tests as `const` values and collect them at build time,
/// instead of running collection code when the test binary starts.
#[derive(Debug, Default)]
#[non_exhaustive]
pub struct Test<Extra = ()> {
    function: TestFnHandle,
    pub meta: TestMeta<Extra>,
}

impl<Extra> Test<Extra> {
    pub const fn new(function: TestFnHandle, meta: TestMeta<Extra>) -> Self {
        Self { function, meta }
    }

    pub(crate) fn call(&self) -> TestResult {
        self.function.call()
    }
}

impl<Extra> Deref for Test<Extra> {
    type Target = TestMeta<Extra>;

    fn deref(&self) -> &Self::Target {
        &self.meta
    }
}

/// Metadata describing a test.
///
/// [`TestMeta`] contains information about a test without any execution context.
/// Some strategies intentionally only see metadata, so they can make decisions
/// (filtering, ignoring, grouping, formatting) without any risk of accidentally executing a test
/// at the wrong place.
///
/// All fields are designed to be constructible in `const` contexts.
/// This allows building tests at compile time.
/// Types like [`Cow<'static, str>`] are used for this reason.
///
/// The generic `Extra` parameter stores user provided metadata used to annotate tests for a
/// specific use case.
#[derive(Debug, Clone, Default)]
pub struct TestMeta<Extra = ()> {
    /// The display name of the test.
    ///
    /// This does not have to be globally unique, but it should be unique enough to
    /// make test output readable and to avoid confusion in formatters.
    ///
    /// The built in Rust harness typically uses the module path style for names,
    /// but Kitest does not require that.
    pub name: Cow<'static, str>,

    /// Whether the test should be ignored and optionally why.
    ///
    /// This is comparable to Rust's `#[ignore]` attribute.
    /// A [`TestIgnore`](super::strategy::ignore::TestIgnore) strategy usually takes this into
    /// account when deciding if a test should run.
    pub ignore: IgnoreStatus,

    /// Whether the test is expected to panic.
    ///
    /// This is comparable to Rust's `#[should_panic]` attribute.
    /// A [`TestPanicHandler`](super::strategy::panic::TestPanicHandler)
    /// can use this field to decide whether a panic is expected and whether an observed
    /// panic should fail the test.
    pub should_panic: PanicExpectation,

    /// Optional information about where this test comes from.
    ///
    /// This is most commonly a file location, for example a Rust source file or a text
    /// fixture, but it can represent any origin that makes sense for the test source.
    pub origin: Option<TestOrigin>,

    /// User provided metadata for this test.
    ///
    /// This is the main extension point for annotating tests with extra information,
    /// like flags, categories, tags, fixture names, feature requirements, or anything
    /// else your harness strategies need.
    pub extra: Extra,
}

/// Describes where a test comes from.
///
/// A [`TestOrigin`] is optional metadata that can be attached to [`TestMeta`].
/// It is meant to help users find the source of a test, for example a file on disk, a generated
/// fixture, or some external system.
///
/// Kitest's built in formatters treat the origin as display text and simply call
/// [`Display`] on it when they want to refer to a test source.
///
/// The [`origin!`](crate::origin) macro produces `Some(TestOrigin::TextFile { .. })` at the call
/// site, so it can be used to stamp tests with a source location easily.
#[non_exhaustive]
#[derive(Debug, Clone)]
pub enum TestOrigin {
    /// A typical location in a plain text file.
    ///
    /// This is formatted as `{file}:{line}:{column}` so that editors and terminals can
    /// recognize it and allow jumping directly to that location.
    TextFile {
        /// Path to the file.
        file: Cow<'static, str>,
        /// 1-based line number.
        line: u32,
        /// 1-based column number.
        column: u32,
    },

    /// A custom origin value.
    ///
    /// This can be whatever fits your use case.
    /// Keep in mind that built in formatting will display it in places where a `TextFile` origin
    /// might also appear.
    ///
    /// If you need richer output, custom formatters may choose to use their own origin type
    /// and format it differently.
    ///
    /// This variant stores a [`Whatever`].
    /// `Whatever` can be downcast, so if the producer of the origin and a formatter agree on the
    /// underlying type, the formatter may be able to recover the original value.
    Custom(Whatever),
}

impl Display for TestOrigin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TestOrigin::TextFile { file, line, column } => write!(f, "{file}:{line}:{column}"),
            TestOrigin::Custom(whatever) => Display::fmt(whatever, f),
        }
    }
}

/// Capture the source location where a test is defined.
///
/// This macro produces `Some(TestOrigin::TextFile { .. })` using the call site
/// information of the macro invocation.
///
/// It records:
/// - the file path
/// - the line number
/// - the column number
///
/// The resulting [`TestOrigin`] is formatted as `{file}:{line}:{column}`, which is
/// understood by most editors and terminals and allows jumping directly to the
/// source location.
///
/// This macro is most useful when building your own test definition macros.
/// In that setup, each test can automatically carry the location of the macro call,
/// even if the test is later executed from a generated list or a distributed slice.
#[macro_export]
macro_rules! origin {
    () => {
        ::std::option::Option::Some($crate::test::TestOrigin::TextFile {
            file: ::std::borrow::Cow::Borrowed(::std::file!()),
            line: ::std::line!(),
            column: ::std::column!(),
        })
    };
}

/// Describes how a test is executed.
///
/// [`TestFnHandle`] is the executable part of a [`Test`]. It stores a callable value that
/// produces a [`TestResult`].
///
/// Kitest supports multiple ways of storing a test function, so tests can be created in
/// `const` contexts, generated by macros, or built dynamically at runtime.
#[non_exhaustive]
pub enum TestFnHandle {
    /// A plain function pointer of type `fn() -> TestResult`.
    ///
    /// This is the most lightweight representation and is especially useful for macro based
    /// test generation (for example, implementing your own `#[test]` style macro).
    ///
    /// A macro can generate a small wrapper function that calls the actual test body and
    /// converts its return value into a [`TestResult`].
    /// The wrapper can then be stored as a function pointer in this variant.
    Ptr(fn() -> TestResult),

    /// An owned, boxed test function.
    ///
    /// This variant is useful when tests are constructed at runtime and need to be stored
    /// somewhere.
    /// Boxing a closure makes it easy to capture state and keep the handle independent of where it
    /// was created.
    Owned(Box<dyn TestFn + Send + Sync + RefUnwindSafe>),

    /// A static reference to a test function object.
    ///
    /// This is similar to [`Owned`](TestFnHandle::Owned), but instead of owning a boxed closure,
    /// it stores a reference to a function object with `'static` lifetime.
    /// This is useful when the closure is stored in a static value or otherwise lives for the
    /// entire program.
    Static(&'static (dyn TestFn + Send + Sync + RefUnwindSafe)),
}

impl Debug for TestFnHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Ptr(ptr) => f.debug_tuple("Ptr").field(ptr).finish(),
            Self::Owned(_) => write!(f, "Owned(...)"),
            Self::Static(_) => write!(f, "Static(...)"),
        }
    }
}

impl Default for TestFnHandle {
    fn default() -> Self {
        Self::Static(&|| {})
    }
}

impl TestFnHandle {
    /// Construct a [`TestFnHandle`] from a function pointer in a `const` context.
    ///
    /// This creates a [`Ptr`](Self::Ptr) variant from a `fn() -> TestResult`.
    /// Because this function is `const`, it can be used to construct tests at
    /// compile time.
    ///
    /// This is especially useful for macro based test definitions, where a macro
    /// generates a small wrapper function that adapts a test body to return a
    /// [`TestResult`].
    pub const fn from_const_fn(f: fn() -> TestResult) -> Self {
        Self::Ptr(f)
    }

    /// Construct a [`TestFnHandle`] from a boxed test function object.
    ///
    /// This creates an [`Owned`](TestFnHandle::Owned) variant from a boxed closure or function
    /// object.
    /// It is the usual choice when tests are built at runtime and need to capture data.
    ///
    /// This method takes ownership of the box.
    pub fn from_boxed<F, T>(f: F) -> Self
    where
        F: Fn() -> T + Send + Sync + RefUnwindSafe + 'static,
        T: Into<TestResult>,
    {
        Self::Owned(Box::new(f))
    }

    /// Construct a [`TestFnHandle`] from a static test function object.
    ///
    /// This creates a [`Static`](TestFnHandle::Static) variant from a reference with `'static`
    /// lifetime.
    /// The constructor is `const`, so it can be used in compile time test definitions.
    ///
    /// This is useful when a closure or function object is stored in a static
    /// value and should be reused without allocation.
    pub const fn from_static_obj(f: &'static (dyn TestFn + Send + Sync + RefUnwindSafe)) -> Self {
        Self::Static(f)
    }

    /// Execute the test and return its [`TestResult`].
    ///
    /// This method invokes the underlying test function regardless of how it is stored.
    /// It provides a uniform way for the harness and panic handler to execute tests.
    pub fn call(&self) -> TestResult {
        match self {
            Self::Ptr(f) => f(),
            Self::Owned(f) => f.call_test(),
            Self::Static(f) => f.call_test(),
        }
    }
}

/// A callable test function.
///
/// [`TestFn`] is a small trait used by [`TestFnHandle`] to execute tests behind a trait object.
/// It represents "something that can be called and produces a [`TestResult`]".
///
/// In theory, this could be modeled directly with Rust's built in [`Fn`] traits.
/// At the time of writing, implementing `Fn` for arbitrary user types is not available on stable
/// Rust, so Kitest uses this dedicated trait instead.
///
/// Kitest provides a blanket implementation for any function or closure that returns something
/// convertible into a [`TestResult`].
/// This includes:
///
/// - `()`
/// - `Result<T, E>` where `E: Debug`
///
/// This makes normal test functions work naturally, while still allowing custom closures or
/// adapters to be stored as `dyn TestFn`.
pub trait TestFn {
    /// Call the test function and produce a [`TestResult`].
    fn call_test(&self) -> TestResult;
}

impl<F, T> TestFn for F
where
    F: Fn() -> T,
    T: Into<TestResult>,
{
    fn call_test(&self) -> TestResult {
        (self)().into()
    }
}

/// The result type returned by a test function.
///
/// [`TestResult`] is what a [`TestFnHandle`] produces when a test is executed.
/// It is a small, opinionated result type: a test either succeeds or fails, and failures carry an
/// error message.
///
/// A successful test does not produce any value. A failing test carries a `String` that
/// describes what went wrong.
/// This keeps the common success path cheap.
///
/// `TestResult` is designed to be easy to return from typical test functions:
///
/// - It can be created from `()` which makes "no return value" test functions work well.
/// - It can be created from `Result<T, E>` where `E: Debug`. On failure, the error is formatted
///   using its `Debug` output and used as the failure message.
///
/// While `TestResult` is most often produced by regular Rust test functions, it does not have to
/// come from one.
/// A data driven test runner can also construct a `TestResult` directly, for example when
/// validating fixtures and producing its own error messages.
///
/// Note: failures store a `String` instead of a borrowed string.
/// The expectation is that `Ok` is the hot path, and allocating error strings only happens on
/// failure.
#[derive(Debug)]
pub struct TestResult(pub Result<(), String>);

impl From<()> for TestResult {
    fn from(_: ()) -> Self {
        Self(Ok(()))
    }
}

impl<E: Debug> From<Result<(), E>> for TestResult {
    fn from(v: Result<(), E>) -> Self {
        TestResult(v.map_err(|e| format!("{e:#?}")))
    }
}
