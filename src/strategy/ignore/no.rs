use crate::{
    ignore::{IgnoreStatus, TestIgnore},
    test::TestMeta,
};

/// A [`TestIgnore`] implementation that never ignores tests.
///
/// All tests are always executed, regardless of any ignore metadata they may
/// carry.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct NoIgnore;

impl<Extra> TestIgnore<Extra> for NoIgnore {
    fn ignore(&self, _: &TestMeta<Extra>) -> IgnoreStatus {
        IgnoreStatus::Run
    }
}
