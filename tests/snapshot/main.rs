use std::{
    io, process::{Command, ExitStatus}, string::FromUtf8Error, sync::{Arc, Mutex}
};

use kitest::{
    formatter::{common::color::SupportsColor, pretty::PrettyFormatter},
    ignore::IgnoreStatus,
    panic::PanicExpectation,
    test::{Test, TestFnHandle, TestMeta},
};

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

#[derive(Debug)]
#[allow(dead_code)]
enum BufferToStringError {
    Poison,
    FromUtf8(FromUtf8Error),
}

impl Buffer {
    fn try_to_string(&self) -> Result<String, BufferToStringError> {
        let guard = self.0.lock().map_err(|_| BufferToStringError::Poison)?;
        let string =
            String::from_utf8(guard.to_vec()).map_err(|err| BufferToStringError::FromUtf8(err))?;
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

mod all_ok;
#[test]
fn all_ok() {
    let file_name = build_cargo_test("all_ok").unwrap();
    let expected = Command::new(format!("target/snapshot/{file_name}")).output().unwrap();
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
