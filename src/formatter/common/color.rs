use std::io;

#[derive(Debug, Default, PartialEq, Eq, Clone, Copy)]
pub enum ColorSetting {
    #[default]
    Automatic,
    Always,
    Never,
}

pub(crate) mod colors {
    pub const RESET: &str = "\x1b[0m";
    pub const RED: &str = "\x1b[31m";
    pub const GREEN: &str = "\x1b[32m";
    pub const YELLOW: &str = "\x1b[33m";
    pub const CYAN: &str = "\x1b[36m";
}

pub trait SupportsColor {
    fn supports_color(&self) -> bool;
}

impl<T: io::IsTerminal> SupportsColor for T {
    fn supports_color(&self) -> bool {
        self.is_terminal()
    }
}
