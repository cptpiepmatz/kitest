//! Common helpers for formatter implementations.
//!
//! This module contains small helper types that are convenient when implementing kitest formatters.
//! They are intentionally formatter focused and are not meant to be general purpose building blocks
//! for unrelated code.

use std::{collections::HashMap, io};

use crate::{
    formatter::{
        FmtListTest, TestFormatter,
        common::color::{ColorSetting, SupportsColor},
    },
    outcome::TestFailure,
    test::Test,
};
use color::colors::*;

pub mod color;
pub mod fto; // format transfer object
pub mod label;

/// A small newtype around a test name.
///
/// This is mainly used to make formatter implementations nicer to read, since it
/// can be constructed directly from [`FmtListTest`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct TestName<'t>(pub &'t str);

impl<'t, Extra> From<FmtListTest<'t, Extra>> for TestName<'t> {
    fn from(value: FmtListTest<'t, Extra>) -> Self {
        Self(value.meta.name.as_ref())
    }
}

#[derive(Debug, Clone)]
pub(super) struct CommonFormatter<'t, W: io::Write, Extra> {
    pub target: W,
    pub color_setting: ColorSetting,
    pub tests: HashMap<&'t str, &'t Test<Extra>>,
}

impl<'t, W: io::Write + SupportsColor, Extra> CommonFormatter<'t, W, Extra> {
    pub fn use_color(&self) -> bool {
        match self.color_setting {
            ColorSetting::Automatic => self.target.supports_color(),
            ColorSetting::Always => true,
            ColorSetting::Never => false,
        }
    }
}

impl<'t, Extra> Default for CommonFormatter<'t, io::Stdout, Extra> {
    fn default() -> Self {
        Self {
            target: io::stdout(),
            color_setting: Default::default(),
            tests: Default::default(),
        }
    }
}

impl<'t, Extra: 't + Sync, W: io::Write + SupportsColor + Send> TestFormatter<'t, Extra>
    for CommonFormatter<'t, W, Extra>
{
    type Error = io::Error;

    type RunInit = fto::Tests<'t, Extra>;
    fn fmt_run_init(&mut self, data: Self::RunInit) -> Result<(), Self::Error> {
        self.tests = HashMap::from_iter(data.0.iter().map(|test| (test.name.as_ref(), test)));
        Ok(())
    }

    type RunStart = fto::TestCount;
    fn fmt_run_start(&mut self, data: Self::RunStart) -> Result<(), Self::Error> {
        match data.0 {
            1 => writeln!(self.target, "\nrunning 1 test"),
            count => writeln!(self.target, "\nrunning {count} tests"),
        }
    }

    type RunOutcomes = fto::RunOutcomes<'t>;
    fn fmt_run_outcomes(
        &mut self,
        fto::RunOutcomes {
            passed,
            failed,
            ignored,
            filtered_out,
            duration,
            failures,
        }: Self::RunOutcomes,
    ) -> Result<(), Self::Error> {
        if !failures.is_empty() {
            writeln!(self.target)?;
            writeln!(self.target, "failures:")?;
            writeln!(self.target)?;
            for failure in failures.iter() {
                writeln!(self.target, "---- {} stdout ----", failure.name)?;
                match &failure.failure {
                    TestFailure::Error(err) => writeln!(self.target, "Error: {err}")?,
                    TestFailure::Panicked(_) => self.target.write_all(failure.output.raw())?,
                    TestFailure::DidNotPanic { .. } => {
                        if let Some(meta) = self.tests.get(failure.name)
                            && let Some(origin) = &meta.origin
                        {
                            write!(
                                self.target,
                                "note: test did not panic as expected at {origin}"
                            )?;
                        }
                    }
                    TestFailure::PanicMismatch {
                        got: _,
                        expected: None,
                    } => unreachable!("mismatch not possible without expectation"),
                    TestFailure::PanicMismatch {
                        got,
                        expected: Some(expected),
                    } => {
                        self.target.write_all(failure.output.raw())?;
                        writeln!(self.target, "note: panic did not contain expected string")?;
                        writeln!(self.target, "      panic message: {got:?}")?;
                        write!(self.target, " expected substring: {expected:?}")?;
                    }
                }
                writeln!(self.target)?;
            }
            writeln!(self.target)?;
            writeln!(self.target, "failures:")?;
            for failure in failures.iter() {
                writeln!(self.target, "    {}", failure.name)?;
            }
        }

        writeln!(self.target)?;
        write!(self.target, "test result: ")?;
        match (failed, self.use_color()) {
            (0, false) => write!(self.target, "ok. ")?,
            (0, true) => write!(self.target, "{GREEN}ok{RESET}. ")?,
            (_, false) => write!(self.target, "FAILED. ")?,
            (_, true) => write!(self.target, "{RED}FAILED{RESET}. ")?,
        }
        writeln!(
            self.target,
            "{passed} passed; {failed} failed; {ignored} ignored; 0 measured; {filtered_out} filtered out; finished in {:.2}s",
            duration.as_secs_f64()
        )?;
        writeln!(self.target)
    }

    type TestIgnored = ();
    type TestStart = ();
    type TestOutcome = ();
}
