use crate::{
    ignore::{IgnoreStatus, TestIgnore},
    test::TestMeta,
};

#[derive(Debug, Default)]
pub struct NoIgnore;

impl<Extra> TestIgnore<Extra> for NoIgnore {
    fn ignore(&self, _: &TestMeta<Extra>) -> IgnoreStatus {
        IgnoreStatus::Run
    }
}
