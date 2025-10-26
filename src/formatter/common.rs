use crate::formatter::FmtListTest;

#[derive(Default)]
pub enum ColorSetting {
    #[default]
    Automatic,
    Always,
    Never,
}

pub struct TestName<'t>(pub &'t str);

impl<'t, Extra> From<FmtListTest<'t, Extra>> for TestName<'t> {
    fn from(value: FmtListTest<'t, Extra>) -> Self {
        Self(value.meta.name.as_ref())
    }
}
