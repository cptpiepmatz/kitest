use std::{cell::RefCell, mem};

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
