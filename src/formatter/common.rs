use crate::formatter::FmtListTest;

#[derive(Default)]
pub enum ColorSetting {
    #[default]
    Automatic,
    Always,
    Never,
}

pub struct TestName<'m>(pub &'m str);

impl<'m, Extra> From<FmtListTest<'m, Extra>> for TestName<'m> {
    fn from(value: FmtListTest<'m, Extra>) -> Self {
        Self(value.meta.name.as_ref())
    }
}