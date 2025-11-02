use crate::{
    ignore::{IgnoreStatus, TestIgnore},
    test::TestMeta,
};

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

    }
}
