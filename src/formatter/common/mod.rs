use crate::formatter::FmtListTest;

pub mod color;
pub mod label;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct TestName<'t>(pub &'t str);

impl<'t, Extra> From<FmtListTest<'t, Extra>> for TestName<'t> {
    fn from(value: FmtListTest<'t, Extra>) -> Self {
        Self(value.meta.name.as_ref())
    }
}
