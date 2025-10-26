use std::borrow::Cow;

use crate::test::TestMeta;

pub trait TestIgnore<Extra> {
    fn ignore(&self, meta: &TestMeta<Extra>) -> (bool, Option<Cow<'static, str>>);
}

pub struct NoIgnore;

impl<Extra> TestIgnore<Extra> for NoIgnore {
    fn ignore(&self, _: &TestMeta<Extra>) -> (bool, Option<Cow<'static, str>>) {
        (false, None)
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
    fn ignore(&self, meta: &TestMeta<Extra>) -> (bool, Option<Cow<'static, str>>) {
        match (self, meta.ignore.0) {
            (Self::IncludeIgnored, _) => (false, None),
            (Self::IgnoredOnly, true) | (Self::Default, false) => (false, None),
            (Self::IgnoredOnly, false) | (Self::Default, true) => (true, meta.ignore.1.clone()),
        }
    }
}

impl<Extra, F> TestIgnore<Extra> for F
where
    F: Fn(&TestMeta<Extra>) -> (bool, Option<Cow<'static, str>>),
{
    fn ignore(&self, meta: &TestMeta<Extra>) -> (bool, Option<Cow<'static, str>>) {
        self(meta)
    }
}
