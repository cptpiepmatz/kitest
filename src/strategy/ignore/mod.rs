use std::borrow::Cow;

use crate::test::TestMeta;

mod no;
pub use no::*;

mod default;
pub use default::*;

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub enum IgnoreStatus {
    #[default]
    Run,
    Ignore,
    IgnoreWithReason(Cow<'static, str>),
}

impl From<bool> for IgnoreStatus {
    fn from(value: bool) -> Self {
        match value {
            true => Self::Ignore,
            false => Self::Run,
        }
    }
}

impl From<&'static str> for IgnoreStatus {
    fn from(value: &'static str) -> Self {
        Self::IgnoreWithReason(value.into())
    }
}

impl From<String> for IgnoreStatus {
    fn from(value: String) -> Self {
        Self::IgnoreWithReason(value.into())
    }
}

pub trait TestIgnore<Extra> {
    fn ignore(&self, meta: &TestMeta<Extra>) -> IgnoreStatus;
}

impl<Extra, F> TestIgnore<Extra> for F
where
    F: Fn(&TestMeta<Extra>) -> IgnoreStatus,
{
    fn ignore(&self, meta: &TestMeta<Extra>) -> IgnoreStatus {
        self(meta)
    }
}
