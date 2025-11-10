#![allow(special_module_name)]

use std::{
    io,
    path::Path,
    process::{Command, ExitCode},
    string::FromUtf8Error,
    sync::{Arc, Mutex},
};

use kitest::formatter::common::color::SupportsColor;

mod lib;
mod tests;

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
        let string = String::from_utf8(guard.to_vec()).map_err(Error::FromUtf8)?;
        Ok(string)
    }
}

// rustc --test tests/snapshot.rs --cfg=snapshot --out-dir=target/snapshot
fn build_cargo_test(name: &str) -> io::Result<String> {
    let file = format!("tests/snapshot/tests/{name}.rs");
    let args = [
        &file,
        "--test",
        "--cfg=snapshot",
        "--out-dir=target/snapshot",
    ];
    let status = Command::new("rustc").args(args).status()?;
    if !status.success() {
        return Err(io::Error::other("compilation failed"));
    }
    let output = Command::new("rustc")
        .args(args.iter().chain(["--print=file-names"].iter()))
        .output()?;
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
    let path = format!("target/snapshot/{}", path.as_ref().display());
    let output = Command::new(path)
        .arg("--test-threads=1")
        .output()
        .map_err(Error::Io)?;
    let stdout = String::from_utf8(output.stdout).map_err(Error::FromUtf8)?;
    let exit_code = match output.status.success() {
        true => ExitCode::SUCCESS,
        false => ExitCode::FAILURE,
    };
    Ok(RustDocTestReport { stdout, exit_code })
}
