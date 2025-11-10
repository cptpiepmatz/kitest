pub use std::io;

use crate::formatter::{common::{TestName, color::{ColorSetting}}, *};

#[derive(Debug)]
pub struct TerseFormatter<W: io::Write> {
    pub target: W,
    pub color_setting: ColorSetting,
}

impl Default for TerseFormatter<io::Stdout> {
    fn default() -> Self {
        Self {
            target: io::stdout(),
            color_setting: Default::default(),
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
