use crate::test::TestMeta;

pub trait TestGrouper<Extra, GroupKey, GroupCtx = ()> {
    fn group(&mut self, meta: &TestMeta<Extra>) -> GroupKey;

    fn group_ctx(&self, key: &GroupKey) -> Option<&GroupCtx> {
        let _ = key;
        None
    }
}

impl<F, Extra, GroupKey> TestGrouper<Extra, GroupKey> for F
where
    F: Fn(&TestMeta<Extra>) -> GroupKey,
{
    fn group(&mut self, meta: &TestMeta<Extra>) -> GroupKey {
        self(meta)
    }
}