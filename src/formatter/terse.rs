pub use std::io;

use crate::formatter::{
    common::{TestName, color::ColorSetting},
    *,
};

#[derive(Debug)]
pub struct TerseFormatter<W: io::Write> {
    target: W,
    color_setting: ColorSetting,
}

impl Default for TerseFormatter<io::Stdout> {
    fn default() -> Self {
        Self {
            target: io::stdout(),
            color_setting: Default::default(),
        }
    }
}

impl<W: io::Write> TerseFormatter<W> {
    pub fn with_target<WithTarget: io::Write>(
        self,
        with_target: WithTarget,
    ) -> TerseFormatter<WithTarget> {
        TerseFormatter {
            target: with_target,
            color_setting: self.color_setting,
        }
    }

    pub fn with_color_setting(self, color_setting: impl Into<ColorSetting>) -> Self {
        TerseFormatter {
            color_setting: color_setting.into(),
            ..self
        }
    }
}

impl<'t, Extra: 't, W: io::Write> TestListFormatter<'t, Extra> for TerseFormatter<W> {
    type Error = io::Error;

    type ListTest = TestName<'t>;
    fn fmt_list_test(&mut self, data: Self::ListTest) -> Result<(), Self::Error> {
        writeln!(self.target, "{}: test", data.0)
    }

    type InitListing = ();
    type BeginListing = ();
    type EndListing = ();
}

// TODO: need to implement formatting for running tests
