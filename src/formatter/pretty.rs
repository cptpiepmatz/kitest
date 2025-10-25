use std::io;

use crate::formatter::{FmtEndListing, FmtListTest, TestListFormatter};

pub struct PrettyFormatter<W: io::Write> {
    pub target: W,
    pub color_settings: ColorSetting,
}

#[derive(Default)]
pub enum ColorSetting {
    #[default]
    Automatic,
    Always,
    Never,
}

impl Default for PrettyFormatter<io::Stdout> {
    fn default() -> Self {
        Self {
            target: io::stdout(),
            color_settings: Default::default(),
        }
    }
}

pub struct TestName<'m>(&'m str);

impl<'m, Extra> From<FmtListTest<'m, Extra>> for TestName<'m> {
    fn from(value: FmtListTest<'m, Extra>) -> Self {
        Self(value.meta.name.as_ref())
    }
}

pub struct TestCount(usize);

impl From<FmtEndListing> for TestCount {
    fn from(value: FmtEndListing) -> Self {
        TestCount(value.active + value.ignored)
    }
}

impl<'m, Extra: 'm, W: io::Write> TestListFormatter<'m, Extra> for PrettyFormatter<W> {
    type Error = io::Error;

    type ListTest = TestName<'m>;
    fn fmt_list_test(&mut self, data: Self::ListTest) -> Result<(), Self::Error> {
        writeln!(self.target, "{}: test", data.0)
    }

    type EndListing = TestCount;
    fn fmt_end_listing(&mut self, data: Self::EndListing) -> Result<(), Self::Error> {
        writeln!(self.target, "\n{} tests", data.0)
    }

    type InitListing = ();
    type BeginListing = ();
}
