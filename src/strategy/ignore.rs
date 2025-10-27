use std::borrow::Cow;

use crate::test::TestMeta;

pub enum IgnoreDecision {
    Run,
    Ignore,
    IgnoreWithReason(Cow<'static, str>),
}

pub trait TestIgnore<Extra> {
    fn ignore(&self, meta: &TestMeta<Extra>) -> IgnoreDecision;
}

pub struct NoIgnore;

impl<Extra> TestIgnore<Extra> for NoIgnore {
    fn ignore(&self, _: &TestMeta<Extra>) -> IgnoreDecision {
        IgnoreDecision::Run
    }
}

#[derive(Default)]
pub enum DefaultIgnore {
    IncludeIgnored,
    IgnoredOnly,
    #[default]
    Default,
}

impl<Extra> TestIgnore<Extra> for DefaultIgnore {
    fn ignore(&self, meta: &TestMeta<Extra>) -> IgnoreDecision {
        match (self, meta.ignore.0) {
            (Self::IncludeIgnored, _) => IgnoreDecision::Run,
            (Self::IgnoredOnly, true) | (Self::Default, false) => IgnoreDecision::Run,
            (Self::IgnoredOnly, false) | (Self::Default, true) => match meta.ignore.1.clone() {
                Some(reason) => IgnoreDecision::IgnoreWithReason(reason),
                None => IgnoreDecision::Ignore,
            },
        }
    }
}

impl<Extra, F> TestIgnore<Extra> for F
where
    F: Fn(&TestMeta<Extra>) -> IgnoreDecision,
{
    fn ignore(&self, meta: &TestMeta<Extra>) -> IgnoreDecision {
        self(meta)
    }
}
