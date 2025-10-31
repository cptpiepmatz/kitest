use std::borrow::Cow;

use crate::test::TestMeta;

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

#[derive(Debug, Default)]
pub struct NoIgnore;

impl<Extra> TestIgnore<Extra> for NoIgnore {
    fn ignore(&self, _: &TestMeta<Extra>) -> IgnoreStatus {
        IgnoreStatus::Run
    }
}

#[derive(Debug, Default)]
pub enum DefaultIgnore {
    IncludeIgnored,
    IgnoredOnly,
    #[default]
    Default,
}

impl<Extra> TestIgnore<Extra> for DefaultIgnore {
    fn ignore(&self, meta: &TestMeta<Extra>) -> IgnoreStatus {
        match (self, &meta.ignore) {
            (DefaultIgnore::IgnoredOnly, IgnoreStatus::Run) => IgnoreStatus::Ignore,
            (DefaultIgnore::IncludeIgnored, _)
            | (DefaultIgnore::IgnoredOnly, IgnoreStatus::Ignore)
            | (DefaultIgnore::IgnoredOnly, IgnoreStatus::IgnoreWithReason(_))
            | (DefaultIgnore::Default, IgnoreStatus::Run) => IgnoreStatus::Run,
            (DefaultIgnore::Default, status) => status.clone(),
        }
    }
}

impl<Extra, F> TestIgnore<Extra> for F
where
    F: Fn(&TestMeta<Extra>) -> IgnoreStatus,
{
    fn ignore(&self, meta: &TestMeta<Extra>) -> IgnoreStatus {
        self(meta)
    }
}
