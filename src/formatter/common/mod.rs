//! Common helpers for formatter implementations.
//!
//! This module contains small helper types that are convenient when implementing kitest formatters.
//! They are intentionally formatter focused and are not meant to be general purpose building blocks
//! for unrelated code.

use std::{collections::HashMap, fmt::Display, io, marker::PhantomData};

use crate::{
    formatter::{
        FmtGroupStart, GroupedTestFormatter, TestFormatter,
        common::{
            color::{ColorSetting, SupportsColor},
            label::{FromGroupKey, GroupLabel},
        },
    },
    outcome::TestFailure,
    test::Test,
};
use color::colors::*;

pub mod color;
pub mod fto; // format transfer object
pub mod label;

#[derive(Debug, Clone)]
pub(super) struct CommonFormatter<'t, W: io::Write, L, Extra> {
    pub target: W,
    pub color_setting: ColorSetting,
    pub tests: HashMap<&'t str, &'t Test<Extra>>,
    pub _label_marker: PhantomData<L>,
}

impl<'t, W: io::Write + SupportsColor, L, Extra> CommonFormatter<'t, W, L, Extra> {
    pub fn use_color(&self) -> bool {
        match self.color_setting {
            ColorSetting::Automatic => self.target.supports_color(),
            ColorSetting::Always => true,
            ColorSetting::Never => false,
        }
    }

    fn fmt_common_run_outcomes(&mut self, data: &fto::RunOutcomes) -> io::Result<()> {
        if !data.failures.is_empty() {
            writeln!(self.target)?;
            writeln!(self.target, "failures:")?;
            writeln!(self.target)?;
            for failure in data.failures.iter() {
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
            for failure in data.failures.iter() {
                writeln!(self.target, "    {}", failure.name)?;
            }
        }

        writeln!(self.target)?;
        write!(self.target, "test result: ")?;
        match (data.failed, self.use_color()) {
            (0, false) => write!(self.target, "ok. "),
            (0, true) => write!(self.target, "{GREEN}ok{RESET}. "),
            (_, false) => write!(self.target, "FAILED. "),
            (_, true) => write!(self.target, "{RED}FAILED{RESET}. "),
        }
    }
}

impl<'t, Extra> Default for CommonFormatter<'t, io::Stdout, GroupLabel<FromGroupKey>, Extra> {
    fn default() -> Self {
        Self {
            target: io::stdout(),
            color_setting: Default::default(),
            tests: Default::default(),
            _label_marker: PhantomData,
        }
    }
}

impl<'t, W: io::Write + SupportsColor + Send, L: Send, Extra: 't + Sync> TestFormatter<'t, Extra>
    for CommonFormatter<'t, W, L, Extra>
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
        ref data @ fto::RunOutcomes {
            ref passed,
            ref failed,
            ref ignored,
            ref filtered_out,
            ref duration,
            ..
        }: Self::RunOutcomes,
    ) -> Result<(), Self::Error> {
        self.fmt_common_run_outcomes(data)?;
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

impl<'t, Extra, GroupKey, GroupCtx, W, L> GroupedTestFormatter<'t, Extra, GroupKey, GroupCtx>
    for CommonFormatter<'t, W, L, Extra>
where
    Extra: 't + Sync,
    GroupKey: 't,
    GroupCtx: 't,
    W: io::Write + SupportsColor + Send,
    L: Send + Display,
    for<'b, 'g> L: From<&'b FmtGroupStart<'g, GroupKey, GroupCtx>>,
{
    type GroupedRunStart = fto::TestCount;
    fn fmt_grouped_run_start(&mut self, data: Self::GroupedRunStart) -> Result<(), Self::Error> {
        <CommonFormatter<'_, _, _, _> as TestFormatter<'_, Extra>>::fmt_run_start(self, data)
    }

    type GroupStart = fto::GroupStart<L>;
    fn fmt_group_start(&mut self, data: Self::GroupStart) -> Result<(), Self::Error> {
        writeln!(self.target)?;
        let group_name = match data.name.is_empty() {
            true => "default",
            false => data.name.as_str(),
        };
        writeln!(
            self.target,
            "group {group_name}, running {} tests",
            data.tests
        )
    }

    type GroupedRunOutcomes = fto::GroupedRunOutcomes<'t>;
    fn fmt_grouped_run_outcomes(
        &mut self,
        data: Self::GroupedRunOutcomes,
    ) -> Result<(), Self::Error> {
        let (
            groups,
            ref data @ fto::RunOutcomes {
                ref passed,
                ref failed,
                ref ignored,
                ref filtered_out,
                ref duration,
                ..
            },
        ) = data.split();

        self.fmt_common_run_outcomes(data)?;

        writeln!(
            self.target,
            "{passed} passed; {failed} failed; {ignored} ignored; {filtered_out} filtered out; across {groups} groups, finished in {:.2}s",
            duration.as_secs_f64()
        )?;
        writeln!(self.target)
    }

    type GroupOutcomes = ();
}
