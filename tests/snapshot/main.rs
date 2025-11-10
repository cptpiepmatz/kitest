use std::{
    io, path::Path, process::{Command, ExitCode, ExitStatus}, string::FromUtf8Error, sync::{Arc, Mutex}
};

use kitest::{
    formatter::{common::color::SupportsColor, pretty::PrettyFormatter},
    ignore::IgnoreStatus,
    panic::PanicExpectation,
    test::{Test, TestFnHandle, TestMeta},
};

#[derive(Debug)]
#[allow(dead_code)]
pub enum Error {
    Io(io::Error),
    Poison,
    FromUtf8(FromUtf8Error),
}

#[derive(Debug, Default, Clone)]
struct Buffer(Arc<Mutex<Vec<u8>>>);

impl io::Write for Buffer {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let mut guard = self
            .0
            .lock()
            .map_err(|_| io::Error::other("poison error"))?;
        guard.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        let mut guard = self
            .0
            .lock()
            .map_err(|_| io::Error::other("poison error"))?;
        guard.flush()
    }
}

impl SupportsColor for Buffer {
    fn supports_color(&self) -> bool {
        false
    }
}

impl Buffer {
    fn try_to_string(&self) -> Result<String, Error> {
        let guard = self.0.lock().map_err(|_| Error::Poison)?;
        let string =
            String::from_utf8(guard.to_vec()).map_err(Error::FromUtf8)?;
        Ok(string)
    }
}

// rustc --test tests/snapshot.rs --cfg=snapshot --out-dir=target/snapshot
fn build_cargo_test(name: &str) -> io::Result<String> {
    let file = format!("tests/snapshot/{name}.rs");
    let args = [&file, "--test", "--cfg=snapshot", "--out-dir=target/snapshot"];
    let status = Command::new("rustc").args(&args).status()?;
    if !status.success() {
        return Err(io::Error::other("compilation failed"));
    }
    let output = Command::new("rustc").args(args.iter().chain(["--print=file-names"].iter())).output()?;
    if !output.status.success() {
        return Err(io::Error::other("printing name failed"));
    }
    let file_name = String::from_utf8_lossy(&output.stdout);
    Ok(file_name.trim().to_string())
}

struct RustDocTestReport {
    stdout: String,
    exit_code: ExitCode,
}

fn run_rust_doc_test(path: impl AsRef<Path>) -> Result<RustDocTestReport, Error> {
    let output = Command::new(path.as_ref()).arg("--test-threads=1").output().map_err(Error::Io)?;
    let stdout = String::from_utf8(output.stdout).map_err(Error::FromUtf8)?;
    let exit_code = match output.status.success() {
        true => ExitCode::SUCCESS,
        false => ExitCode::FAILURE,
    };
    Ok(RustDocTestReport { stdout, exit_code })
}

mod all_ok;
#[test]
fn all_ok() {
    let file_name = build_cargo_test("all_ok").unwrap();
    let expected = Command::new(format!("target/snapshot/{file_name}")).arg("--test-threads=1").output().unwrap();
    assert!(expected.status.success());
    let expected = String::from_utf8(expected.stdout).unwrap();

    let actual = Buffer::default();
    let _ = kitest::harness(&[Test::new(
        TestFnHandle::from_static_obj(&|| all_ok::ok_1()),
        TestMeta {
            name: "ok_1".into(),
            ignore: IgnoreStatus::default(),
            should_panic: PanicExpectation::default(),
            extra: (),
        },
    )])
    .with_formatter(PrettyFormatter::default().with_target(actual.clone()))
    .run();

    let actual = actual.try_to_string().unwrap();
    assert_eq!(actual, expected);
}
