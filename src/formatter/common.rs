use crate::formatter::FmtListTest;

#[derive(Debug, Default, PartialEq, Eq, Clone, Copy)]
pub enum ColorSetting {
    #[default]
    Automatic,
    Always,
    Never,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TestName<'t>(pub &'t str);

impl<'t, Extra> From<FmtListTest<'t, Extra>> for TestName<'t> {
    fn from(value: FmtListTest<'t, Extra>) -> Self {
        Self(value.meta.name.as_ref())
    }
}
