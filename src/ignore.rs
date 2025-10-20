use std::borrow::Cow;

use crate::meta::TestMeta;

pub trait TestIgnore<Extra> {
    fn ignore<'m>(&self, meta: &'m TestMeta<Extra>) -> (bool, Option<Cow<'m, str>>);
}

pub struct NoIgnore;

impl<Extra> TestIgnore<Extra> for NoIgnore {
    fn ignore<'m>(&self, _: &'m TestMeta<Extra>) -> (bool, Option<Cow<'m, str>>) {
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
    fn ignore<'m>(&self, meta: &'m TestMeta<Extra>) -> (bool, Option<Cow<'m, str>>) {
        match (self, meta.ignore.0) {
            (Self::IncludeIgnored, _) => (false, None),
            (Self::IgnoredOnly, true) | (Self::Default, false) => (false, None),
            (Self::IgnoredOnly, false) | (Self::Default, true) => (
                true,
                meta.ignore.1.as_ref().map(|s| Cow::Borrowed(s.as_ref())),
            ),
        }
    }
}

impl<Extra, F> TestIgnore<Extra> for F
where
    for <'m> F: Fn(&'m TestMeta<Extra>) -> (bool, Option<Cow<'m, str>>),
{
    fn ignore<'m>(&self, meta: &'m TestMeta<Extra>) -> (bool, Option<Cow<'m, str>>) {
        self(meta)
    }
}
