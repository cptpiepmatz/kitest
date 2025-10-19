use std::collections::BTreeMap;

use crate::meta::TestMeta;

pub trait TestGrouper<GroupKey, GroupCtx, Extra>
where
    GroupKey: Ord,
{
    fn group<RandomState>(
        &self,
        meta: &TestMeta<Extra>,
        groups: &mut BTreeMap<GroupKey, (GroupCtx, Vec<&TestMeta<Extra>>)>,
    );
}

impl<GroupKey, GroupCtx, Extra, F> TestGrouper<GroupKey, GroupCtx, Extra> for F
where
    GroupKey: Ord,
    F: Fn(&TestMeta<Extra>, &mut BTreeMap<GroupKey, (GroupCtx, Vec<&TestMeta<Extra>>)>),
{
    fn group<RandomState>(
        &self,
        meta: &TestMeta<Extra>,
        groups: &mut BTreeMap<GroupKey, (GroupCtx, Vec<&TestMeta<Extra>>)>,
    ) {
        self(meta, groups)
    }
}
