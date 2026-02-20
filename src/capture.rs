//! Output capture.
//!
//! This module provides a best effort way to capture stdout and stderr like output during test
//! execution.
//!
//! On stable Rust, rerouting the real process stdio streams is not generally available.
//! Because of that, Kitest cannot reliably intercept output written through `std::println!` and
//! friends.
//!
//! Instead, this module provides:
//! - an [`OutputCapture`] type that stores output events and their target (stdout or stderr)
//! - writers that behave like `stdout` and `stderr` but write into the capture
//! - macros ([`print!`], [`println!`], [`eprint!`], [`eprintln!`], [`dbg!`]) that mirror the
//!   standard ones but route into Kitest's capture
//!
//! This is only a best effort approach.
//! Output written to the real stdout or stderr will still go to the terminal.
//! In practice, tests should not rely on captured output unless they opt into these capture aware
//! macros or otherwise write through the capture API.

use std::{
    any::Any,
    backtrace::Backtrace,
    cell::RefCell,
    fmt::{self, Debug, Display},
    io::{self, Write},
    mem,
    panic::{self, PanicHookInfo},
    sync::{
        LazyLock,
        atomic::{AtomicBool, Ordering},
    },
    thread::{self, ThreadId},
};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum OutputTarget {
    Stdout,
    Stderr,
}

#[derive(Debug, Default, PartialEq, Eq, Clone, Copy)]
pub struct StdoutTarget;

impl From<StdoutTarget> for OutputTarget {
    fn from(_: StdoutTarget) -> Self {
        Self::Stdout
    }
}

#[derive(Debug, Default, PartialEq, Eq, Clone, Copy)]
pub struct StderrTarget;

impl From<StderrTarget> for OutputTarget {
    fn from(_: StderrTarget) -> Self {
        Self::Stderr
    }
}

#[derive(Debug)]
pub struct OutputEvent {
    pub target: OutputTarget,
    range: std::ops::Range<usize>,
}

#[derive(Debug, Default)]
pub struct OutputCapture {
    buf: Vec<u8>,
    events: Vec<OutputEvent>,
}

impl OutputCapture {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn clear(&mut self) {
        self.buf.clear();
        self.events.clear()
    }

    pub fn take(&mut self) -> Self {
        let buf = mem::take(&mut self.buf);
        let events = mem::take(&mut self.events);
        Self { buf, events }
    }

    fn push_event(&mut self, buf: &[u8], target: OutputTarget) {
        let start = self.buf.len();
        let end = start + buf.len();
        let range = start..end;
        self.buf.extend_from_slice(buf);
        self.events.push(OutputEvent { target, range });
    }

    pub fn stdout(&mut self) -> OutputWrite<'_, StdoutTarget> {
        OutputWrite {
            capture: self,
            marker: StdoutTarget,
        }
    }

    pub fn stderr(&mut self) -> OutputWrite<'_, StderrTarget> {
        OutputWrite {
            capture: self,
            marker: StderrTarget,
        }
    }

    pub fn raw(&self) -> &[u8] {
        &self.buf
    }

    fn read_target(&self, target: OutputTarget) -> impl Iterator<Item = &[u8]> {
        self.events
            .iter()
            .filter(move |event| event.target == target)
            .map(|event| &event.range)
            .cloned()
            .map(|range| &self.buf[range])
    }

    pub fn read_stdout(&self) -> impl Iterator<Item = &[u8]> {
        self.read_target(OutputTarget::Stdout)
    }

    pub fn read_stderr(&self) -> impl Iterator<Item = &[u8]> {
        self.read_target(OutputTarget::Stderr)
    }
}

// implement Clone manually to avoid clonable events, they don't make sense in absence of the capture
impl Clone for OutputCapture {
    fn clone(&self) -> Self {
        Self {
            buf: self.buf.clone(),
            events: self
                .events
                .iter()
                .map(|event| OutputEvent {
                    target: event.target,
                    range: event.range.clone(),
                })
                .collect(),
        }
    }
}

#[derive(Debug)]
pub struct OutputWrite<'c, Target> {
    capture: &'c mut OutputCapture,
    marker: Target,
}

impl<Target: Into<OutputTarget> + Copy> io::Write for OutputWrite<'_, Target> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.capture.push_event(buf, self.marker.into());
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

// TODO: move this into another module?

pub type PanicHook = Box<dyn Fn(&PanicHookInfo<'_>) + Sync + Send + 'static>;

pub trait PanicHookProvider: Debug {
    fn provide(&self) -> PanicHook;
}

fn payload_as_str(payload: &dyn Any) -> &str {
    payload
        .downcast_ref::<&str>()
        .copied()
        .or_else(|| payload.downcast_ref::<String>().map(|s| s.as_str()))
        .unwrap_or("Box<dyn Any>")
}

#[derive(Debug)]
struct ThreadIdFmt(ThreadId);

impl ThreadIdFmt {
    fn slice_id(&self) -> impl Display {
        let displayed = format!("{:?}", self.0);
        let slice = &displayed["ThreadId(".len()..];
        let slice = &slice[..slice.len() - 1];
        slice.to_string()
    }
}

impl Display for ThreadIdFmt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.slice_id().fmt(f)
    }
}

#[cfg(test)]
#[test]
fn assert_thread_id_format() {
    let thread = thread::current();
    let tid = thread.id();
    let tid_fmt = ThreadIdFmt(tid);
    assert_eq!(format!("{tid:?}"), format!("ThreadId({tid_fmt})"));
}

static FIRST_PANIC: AtomicBool = AtomicBool::new(true);
#[doc(hidden)]
pub fn reset_first_panic() {
    FIRST_PANIC.store(true, Ordering::Relaxed);
}

static DISABLED_BACKTRACE: LazyLock<String> =
    LazyLock::new(|| format!("{}", Backtrace::disabled()));

fn default_panic_hook(panic_hook_info: &PanicHookInfo<'_>) {
    // for reference: https://github.com/rust-lang/rust/blob/dfe1b8c97bcde283102f706d5dcdc3649e5e12e3/library/std/src/panicking.rs#L240

    TEST_OUTPUT_CAPTURE
        .with_borrow_mut(|capture| {
            let thread = thread::current();
            let name = thread.name().unwrap_or("<unnamed>");
            let tid = ThreadIdFmt(thread.id());

            let mut stderr = capture.stderr();

            stderr.write_fmt(format_args!("\nthread '{name}' ({tid}) panicked"))?;

            if let Some(location) = panic_hook_info.location() {
                stderr.write_fmt(format_args!(" at {location}"))?;
            }

            let payload = payload_as_str(panic_hook_info.payload());
            stderr.write_fmt(format_args!(":\n{payload}\n"))?;

            let backtrace = Backtrace::capture();
            let backtrace = format!("{backtrace}");
            match backtrace.as_str() == DISABLED_BACKTRACE.as_str() {
                false => stderr.write_all(backtrace.as_bytes()),
                true if FIRST_PANIC.swap(false, Ordering::Relaxed) => stderr.write_all(
                    b"note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace\n"
                ),
                true => Ok(())
            }
        })
        .expect("infallible for Vec<u8>");
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct DefaultPanicHookProvider;

impl PanicHookProvider for DefaultPanicHookProvider {
    fn provide(&self) -> PanicHook {
        Box::new(default_panic_hook)
    }
}

pub struct CapturePanicHookGuard(Option<PanicHook>);

impl CapturePanicHookGuard {
    pub fn install(panic_hook: PanicHook) -> Self {
        let old_hook = panic::take_hook();
        panic::set_hook(panic_hook);
        Self(Some(old_hook))
    }
}

impl Drop for CapturePanicHookGuard {
    fn drop(&mut self) {
        if let Some(old_hook) = self.0.take() {
            panic::set_hook(old_hook);
        }
    }
}

/// Controls whether Kitest's output macros capture output into [`TEST_OUTPUT_CAPTURE`].
///
/// Kitest provides `print!`, `println!`, `eprint!`, `eprintln!`, and `dbg!` macros that mirror
/// the standard library macros. This flag decides what those macros do:
///
/// - If `true` (default), the Kitest macros write into the per thread [`TEST_OUTPUT_CAPTURE`]
///   buffer, tagging each write as stdout or stderr.
/// - If `false`, the Kitest macros pass through to the standard library implementations
///   (`std::print!`, `std::println!`, `std::eprint!`, `std::eprintln!`, `std::dbg!`) and output
///   goes to the real process stdout or stderr.
///
/// This is a global static. Set it before any tests run, or only after all tests are done.
/// Changing it while tests are running can lead to confusing results, since different tests may
/// observe different behavior.
///
/// Note: this only affects Kitest's capture aware macros. Output written directly via
/// `std::println!` and friends is not captured on stable Rust.
pub static CAPTURE_OUTPUT_MACROS: AtomicBool = AtomicBool::new(true);

thread_local! {
    pub static TEST_OUTPUT_CAPTURE: RefCell<OutputCapture> = RefCell::new(OutputCapture::new());
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {{
        use ::std::{io::Write, sync::atomic::Ordering};
        match $crate::capture::CAPTURE_OUTPUT_MACROS.load(Ordering::Relaxed) {
            false => ::std::print!($($arg)*),
            true => $crate::capture::TEST_OUTPUT_CAPTURE.with_borrow_mut(|capture| {
                let mut stdout = capture.stdout();
                stdout.write_fmt(::std::format_args!($($arg)*)).expect("infallible for Vec<u8>");
            }),
        };
    }};
}

#[macro_export]
macro_rules! println {
    () => {{
        use ::std::{io::Write, sync::atomic::Ordering};
        match $crate::capture::CAPTURE_OUTPUT_MACROS.load(Ordering::Relaxed) {
            false => ::std::println!(),
            true => $crate::capture::TEST_OUTPUT_CAPTURE.with_borrow_mut(|capture| {
                let mut stdout = capture.stdout();
                stdout.write_all(b"\n").expect("infallible for Vec<u8>");
            }),
        };
    }};

    ($($arg:tt)*) => {{
        use ::std::{io::Write, sync::atomic::Ordering};
        match $crate::capture::CAPTURE_OUTPUT_MACROS.load(Ordering::Relaxed) {
            false => ::std::println!($($arg)*),
            true => $crate::capture::TEST_OUTPUT_CAPTURE.with_borrow_mut(|capture| {
                let mut stdout = capture.stdout();
                stdout.write_fmt(::std::format_args!($($arg)*)).expect("infallible for Vec<u8>");
                stdout.write_all(b"\n").expect("infallible for Vec<u8>");
            })
        };
    }};
}

#[macro_export]
macro_rules! eprint {
    ($($arg:tt)*) => {{
        use ::std::{io::Write, sync::atomic::Ordering};
        match $crate::capture::CAPTURE_OUTPUT_MACROS.load(Ordering::Relaxed) {
            false => ::std::eprint!($($arg)*),
            true => $crate::capture::TEST_OUTPUT_CAPTURE.with_borrow_mut(|capture| {
                let mut stderr = capture.stderr();
                stderr.write_fmt(::std::format_args!($($arg)*)).expect("infallible for Vec<u8>");
            }),
        };
    }};
}

#[macro_export]
macro_rules! eprintln {
    () => {{
        use ::std::{io::Write, sync::atomic::Ordering};

        match $crate::capture::CAPTURE_OUTPUT_MACROS.load(Ordering::Relaxed) {
            false => ::std::println!(),
            true => $crate::capture::TEST_OUTPUT_CAPTURE.with_borrow_mut(|capture| {
                let mut stdout = capture.stdout();
                stderr.write_all(b"\n").expect("infallible for Vec<u8>");
            }),
        };
    }};

    ($($arg:tt)*) => {{
        use ::std::{io::Write, sync::atomic::Ordering};
        match $crate::capture::CAPTURE_OUTPUT_MACROS.load(Ordering::Relaxed) {
            false => ::std::eprintln!($($arg)*),
            true => $crate::capture::TEST_OUTPUT_CAPTURE.with_borrow_mut(|capture| {
                let mut stderr = capture.stderr();
                stderr.write_fmt(::std::format_args!($($arg)*)).expect("infallible for Vec<u8>");
                stderr.write_all(b"\n").expect("infallible for Vec<u8>");
            }),
        };
    }};
}

#[macro_export]
macro_rules! dbg {
    () => {{
        use ::std::sync::atomic::Ordering;
        match $crate::capture::CAPTURE_OUTPUT_MACROS.load(Ordering::Relaxed) {
            false => ::std::dbg!(),
            true => $crate::eprintln!(
                "[{}:{}:{}]",
                ::std::file!(),
                ::std::line!(),
                ::std::column!()
            ),
        }
    }};
    ($val:expr $(,)?) => {{
        use ::std::sync::atomic::Ordering;
        match $crate::capture::CAPTURE_OUTPUT_MACROS.load(Ordering::Relaxed) {
            false => ::std::dbg!($val),
            true => {
                match $val {
                    tmp => {
                        $crate::eprintln!(
                            "[{}:{}:{}] {} = {:#?}",
                            ::std::file!(),
                            ::std::line!(),
                            ::std::column!(),
                            ::std::stringify!($val),
                            &&tmp as &dyn ::std::fmt::Debug,
                        );
                        tmp
                    }
                }
            }
        }
    }};
    ($($val:expr),+ $(,)?) => {{
        use ::std::sync::atomic::Ordering;
        match $crate::capture::CAPTURE_OUTPUT_MACROS.load(Ordering::Relaxed) {
            false => ::std::dbg!($($val),+),
            true => ($($crate::dbg!($val)),+,),
        }
    }};
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::Ordering;

    #[test]
    fn capture_output_macros_respects_flag() {
        CAPTURE_OUTPUT_MACROS.store(true, Ordering::Relaxed);
        TEST_OUTPUT_CAPTURE.with_borrow_mut(|capture| capture.clear());

        print!("hello");
        println!(" world");
        eprint!("err");
        eprintln!(" line");
        let _ = dbg!(42);

        TEST_OUTPUT_CAPTURE.with_borrow(|capture| {
            assert!(
                !capture.raw().is_empty(),
                "expected output to be captured when flag is true"
            );

            // sanity check: both stdout and stderr got something
            assert!(capture.read_stdout().count() > 0);
            assert!(capture.read_stderr().count() > 0);
        });

        CAPTURE_OUTPUT_MACROS.store(false, Ordering::Relaxed);
        TEST_OUTPUT_CAPTURE.with_borrow_mut(|capture| capture.clear());

        print!("hello");
        println!(" world");
        eprint!("err");
        eprintln!(" line");
        let _ = dbg!(1337);

        TEST_OUTPUT_CAPTURE.with_borrow(|capture| {
            assert!(
                capture.raw().is_empty(),
                "expected capture to stay empty when flag is false"
            );
        });
    }
}
