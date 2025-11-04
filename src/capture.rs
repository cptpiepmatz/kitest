use std::{
    cell::RefCell,
    io::Write,
    mem,
    panic::{self, PanicHookInfo},
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

type PanicHook = Box<dyn Fn(&PanicHookInfo<'_>) + Sync + Send + 'static>;
pub struct CapturePanicHookGuard(Option<PanicHook>);

impl CapturePanicHookGuard {
    pub fn install() -> Self {
        let old_hook = panic::take_hook();

        panic::set_hook(Box::new(|panic_hook_info| {
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
        }));

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
