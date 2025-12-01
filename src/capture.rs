use std::{
    any::Any,
    cell::RefCell,
    fmt::Debug,
    io::Write,
    mem,
    panic::{self, PanicHookInfo, set_hook},
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

fn default_panic_hook(panic_hook_info: &PanicHookInfo<'_>) {
    if let Some(s) = panic_hook_info.payload().downcast_ref::<&str>() {
        TEST_OUTPUT_CAPTURE.with_borrow_mut(|capture| {
            capture
                .stderr
                .write(s.as_bytes())
                .expect("infallible for Vec<u8>")
        });
    } else if let Some(s) = panic_hook_info.payload().downcast_ref::<String>() {
        TEST_OUTPUT_CAPTURE.with_borrow_mut(|capture| {
            capture
                .stderr
                .write(s.as_bytes())
                .expect("infallible for Vec<u8>")
        });
    }
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
