use std::{
    any::Any,
    backtrace::Backtrace,
    cell::RefCell,
    fmt::Debug,
    io::{self, Write},
    mem,
    panic::{self, PanicHookInfo},
    sync::{
        LazyLock,
        atomic::{AtomicBool, Ordering},
    },
    thread,
};

#[derive(Debug, Default)]
pub struct TestOutputCapture {
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
}

impl TestOutputCapture {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn clear(&mut self) {
        self.stdout.clear();
        self.stderr.clear();
    }

    pub fn take(&mut self) -> Self {
        let stdout = mem::take(&mut self.stdout);
        let stderr = mem::take(&mut self.stderr);
        Self { stdout, stderr }
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
        .map(|s| *s)
        .or_else(|| payload.downcast_ref::<String>().map(|s| s.as_str()))
        .unwrap_or("Box<dyn Any>")
}

static FIRST_PANIC: AtomicBool = AtomicBool::new(true);
static DISABLED_BACKTRACE: LazyLock<String> =
    LazyLock::new(|| format!("{}", Backtrace::disabled()));

fn default_panic_hook(panic_hook_info: &PanicHookInfo<'_>) {
    // for reference: https://github.com/rust-lang/rust/blob/dfe1b8c97bcde283102f706d5dcdc3649e5e12e3/library/std/src/panicking.rs#L240

    TEST_OUTPUT_CAPTURE
        .with_borrow_mut(|capture| {
            let thread = thread::current();
            let name = thread.name().unwrap_or("<unnamed>");
            let tid = thread.id();

            capture
                .stderr
                .write_fmt(format_args!("\nthread '{name}' ({tid:?}) panicked"))?;

            if let Some(location) = panic_hook_info.location() {
                capture.stderr.write_fmt(format_args!(" at {location}"))?;
            }

            let payload = payload_as_str(panic_hook_info.payload());
            capture.stderr.write_fmt(format_args!(":\n{payload}\n"))?;

            let backtrace = Backtrace::capture();
            let backtrace = format!("{backtrace}");
            match backtrace.as_str() == DISABLED_BACKTRACE.as_str() {
            true => capture.stderr.write_all(backtrace.as_bytes()),
            false if FIRST_PANIC.swap(false, Ordering::Relaxed) => capture.stderr.write_all(
                b"note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace"
            ),
            false => Ok(())
        }
        })
        .expect("infallible for Vec<u8>");
}

#[derive(Debug, Default)]
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

thread_local! {
    pub static TEST_OUTPUT_CAPTURE: RefCell<TestOutputCapture> = RefCell::new(TestOutputCapture::new());
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {{
        use ::std::io::Write;
        $crate::capture::TEST_OUTPUT_CAPTURE.with_borrow_mut(|capture| {
            capture.stdout.write_fmt(::std::format_args!($($arg)*)).expect("infallible for Vec<u8>");
        });
    }};
}

#[macro_export]
macro_rules! println {
    ($($arg:tt)*) => {{
        use ::std::io::Write;
        $crate::capture::TEST_OUTPUT_CAPTURE.with_borrow_mut(|capture| {
            capture.stdout.write_fmt(::std::format_args!($($arg)*)).expect("infallible for Vec<u8>");
            capture.stdout.write_all(b"\n").expect("infallible for Vec<u8>");
        });
    }};
}

#[macro_export]
macro_rules! eprint {
    ($($arg:tt)*) => {{
        use ::std::io::Write;
        $crate::capture::TEST_OUTPUT_CAPTURE.with_borrow_mut(|capture| {
            capture.stderr.write_fmt(::std::format_args!($($arg)*)).expect("infallible for Vec<u8>");
        });
    }};
}

#[macro_export]
macro_rules! eprintln {
    ($($arg:tt)*) => {{
        use ::std::io::Write;
        $crate::capture::TEST_OUTPUT_CAPTURE.with_borrow_mut(|capture| {
            capture.stderr.write_fmt(::std::format_args!($($arg)*)).expect("infallible for Vec<u8>");
            capture.stderr.write_all(b"\n").expect("infallible for Vec<u8>");
        });
    }};
}

#[macro_export]
macro_rules! dbg {
    () => {
        $crate::eprintln!("[{}:{}:{}]", ::std::file!(), ::std::line!(), ::std::column!())
    };
    ($val:expr $(,)?) => {
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
    };
    ($($val:expr),+ $(,)?) => {
        ($($crate::dbg!($val)),+,)
    };
}
