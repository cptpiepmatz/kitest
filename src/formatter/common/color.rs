//! Color related utilities for formatters.

use std::io;

/// Controls whether colored output should be used by a formatter.
///
/// The actual decision may depend on the selected variant and the output target.
#[derive(Debug, Default, PartialEq, Eq, Clone, Copy)]
pub enum ColorSetting {
    /// Enable color only if the output target reports that it supports color.
    ///
    /// This is the default.
    #[default]
    Automatic,

    /// Always emit ANSI color codes, regardless of the target.
    Always,

    /// Never emit ANSI color codes.
    Never,
}

pub(crate) mod colors {
    pub const RESET: &str = "\x1b(B\x1b[m";
    pub const RED: &str = "\x1b[31m";
    pub const GREEN: &str = "\x1b[32m";
    pub const YELLOW: &str = "\x1b[33m";
    pub const CYAN: &str = "\x1b[36m";
}

/// A small trait used to determine whether a writer supports colored output.
///
/// This is used together with [`ColorSetting::Automatic`] to heuristically
/// decide if ANSI color codes should be emitted.
pub trait SupportsColor {
    /// Return `true` if this target supports colored output.
    fn supports_color(&self) -> bool;
}

impl<T: io::IsTerminal> SupportsColor for T {
    fn supports_color(&self) -> bool {
        self.is_terminal()
    }
}
