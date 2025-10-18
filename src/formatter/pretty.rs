use std::{
    fmt::Display,
    hash::Hash,
    io::{self, IsTerminal},
    time::Duration,
};

use nu_ansi_term::{Color, Style};

use crate::{
    ConclusionGroups, TestMeta,
    formatter::{ColorConfig, StartData, Status, TestFormatter, TestOutcome},
};

pub struct PrettyFormatter {
    failures: Vec<FailureRecord>,
}

struct FailureRecord {
    full_name: String,
    status: &'static str,
    msg: Option<String>,
    stdout: Option<Vec<u8>>,
    stderr: Option<Vec<u8>>,
}

impl PrettyFormatter {
    pub fn new() -> Self {
        Self {
            failures: Vec::new(),
        }
    }

    fn use_color(color: &ColorConfig) -> bool {
        match color {
            ColorConfig::Always => true,
            ColorConfig::Never => false,
            ColorConfig::Auto => io::stdout().is_terminal(),
        }
    }

    fn fmt_duration(d: Duration) -> String {
        match d.as_secs() {
            0 => format!("{}ms", d.as_millis()),
            _ => format!("{:.2}s", d.as_secs_f64()),
        }
    }

    fn paint_dimmed(s: &str, use_color: bool) -> String {
        match use_color {
            true => Style::new().dimmed().paint(s).to_string(),
            false => s.to_string(),
        }
    }

    fn paint_ok(s: &str, use_color: bool) -> String {
        match use_color {
            true => Color::Green.paint(s).to_string(),
            false => s.to_string(),
        }
    }

    fn paint_warn(s: &str, use_color: bool) -> String {
        match use_color {
            true => Color::Yellow.paint(s).to_string(),
            false => s.to_string(),
        }
    }

    fn paint_fail(s: &str, use_color: bool) -> String {
        match use_color {
            true => Color::Red.paint(s).to_string(),
            false => s.to_string(),
        }
    }
}

impl<GroupKey: Eq + Hash + Display> TestFormatter<GroupKey> for PrettyFormatter {
    fn fmt_start(
        &mut self,
        w: &mut dyn io::Write,
        color: &ColorConfig,
        data: StartData,
    ) -> io::Result<()> {
        let planned = data.scheduled.saturating_sub(data.filtered);
        let use_color = Self::use_color(color);
        writeln!(
            w,
            "running {} {}",
            planned,
            Self::paint_dimmed("tests", use_color)
        )
    }

    fn fmt_test_finished<'m, 'o, E>(
        &mut self,
        w: &mut dyn io::Write,
        color: &ColorConfig,
        group: &GroupKey,
        meta: &TestMeta<'m, E>,
        outcome: &TestOutcome<'o>,
    ) -> io::Result<()> {
        let use_color = Self::use_color(color);
        let full_name = format!("{}::{}", group, meta.name);

        let dur_str = match outcome.duration {
            Some(d) => format!(
                " {}",
                Self::paint_dimmed(&format!("in {}", Self::fmt_duration(d)), use_color)
            ),
            None => String::new(),
        };

        match &outcome.status {
            Status::Passed => writeln!(
                w,
                "test {} ... {}{}",
                full_name,
                Self::paint_ok("ok", use_color),
                dur_str
            ),
            Status::Ignored => writeln!(
                w,
                "test {} ... {}",
                full_name,
                Self::paint_warn("ignored", use_color)
            ),
            Status::Failed { msg } => {
                writeln!(
                    w,
                    "test {} ... {}{}",
                    full_name,
                    Self::paint_fail("FAILED", use_color),
                    dur_str
                )?;
                self.failures.push(FailureRecord {
                    full_name,
                    status: "FAILED",
                    msg: msg.clone(),
                    stdout: outcome.stdout.map(|b| b.to_vec()),
                    stderr: outcome.stderr.map(|b| b.to_vec()),
                });
                Ok(())
            }
            Status::Error { msg } => {
                writeln!(
                    w,
                    "test {} ... {}{}",
                    full_name,
                    Self::paint_fail("ERROR", use_color),
                    dur_str
                )?;
                self.failures.push(FailureRecord {
                    full_name,
                    status: "ERROR",
                    msg: msg.clone(),
                    stdout: outcome.stdout.map(|b| b.to_vec()),
                    stderr: outcome.stderr.map(|b| b.to_vec()),
                });
                Ok(())
            }
        }
    }

    fn fmt_conclusion(
        &mut self,
        w: &mut dyn io::Write,
        color: &ColorConfig,
        conclusion: &ConclusionGroups<GroupKey>,
    ) -> io::Result<()> {
        let use_color = Self::use_color(color);

        match self.failures.is_empty() {
            false => {
                writeln!(w)?;
                writeln!(w, "failures:")?;
                writeln!(w)?;

                for f in &self.failures {
                    writeln!(w, "---- {} {} ----", f.full_name, "stdout")?;
                    if let Some(out) = &f.stdout {
                        match out.is_empty() {
                            true => {}
                            false => {
                                w.write_all(out)?;
                                match out.ends_with(b"\n") {
                                    true => {}
                                    false => {
                                        writeln!(w)?;
                                    }
                                }
                            }
                        }
                    }

                    writeln!(w, "---- {} {} ----", f.full_name, "stderr")?;
                    if let Some(err) = &f.stderr {
                        match err.is_empty() {
                            true => {}
                            false => {
                                w.write_all(err)?;
                                match err.ends_with(b"\n") {
                                    true => {}
                                    false => {
                                        writeln!(w)?;
                                    }
                                }
                            }
                        }
                    }

                    if let Some(msg) = &f.msg {
                        writeln!(w, "---- {} {} ----", f.full_name, "failure")?;
                        writeln!(w, "{msg}")?;
                    }

                    writeln!(w)?;
                }
            }
            true => {}
        }

        let failed = conclusion.failed();
        let status = match failed == 0 {
            true => Self::paint_ok("ok", use_color),
            false => Self::paint_fail("FAILED", use_color),
        };

        writeln!(
            w,
            "test result: {}. {} passed; {} failed; {} ignored; {} filtered out; finished in {}",
            status,
            conclusion.passed(),
            failed,
            conclusion.ignored(),
            conclusion.filtered_out(),
            Self::fmt_duration(conclusion.duration())
        )
    }
}
