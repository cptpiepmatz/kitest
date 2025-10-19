use std::{
    collections::{BTreeMap, HashMap},
    hash::{BuildHasher, Hash},
};

use crate::meta::TestMeta;

pub trait TestGrouper<GroupKey, Extra> {
    fn group(&self, meta: &TestMeta<Extra>) -> GroupKey;
}

impl<F, GroupKey, Extra> TestGrouper<GroupKey, Extra> for F
where
    F: Fn(&TestMeta<Extra>) -> GroupKey,
{
    fn group(&self, meta: &TestMeta<Extra>) -> GroupKey {
        self(meta)
    }
}

pub trait TestGroups<'m, GroupKey, Extra> {
    fn add(&mut self, key: GroupKey, meta: &'m TestMeta<Extra>);

    fn len(&self) -> usize;

    fn iter<'s>(&'s self) -> impl Iterator<Item = (&'s GroupKey, &'s [&'m TestMeta<Extra>])>
    where
        'm: 's,
        GroupKey: 's,
        Extra: 'm;
}

pub type TestGroupHashMap<'m, GroupKey, Extra, RandomState = std::hash::RandomState> =
    HashMap<GroupKey, Vec<&'m TestMeta<Extra>>, RandomState>;

impl<'m, GroupKey, Extra, RandomState> TestGroups<'m, GroupKey, Extra>
    for TestGroupHashMap<'m, GroupKey, Extra, RandomState>
where
    GroupKey: Eq + Hash,
    RandomState: BuildHasher,
{
    fn add(&mut self, key: GroupKey, meta: &'m TestMeta<Extra>) {
        self.entry(key).or_default().push(meta);
    }

    fn len(&self) -> usize {
        self.values().map(|g| g.len()).sum()
    }

    fn iter<'s>(&'s self) -> impl Iterator<Item = (&'s GroupKey, &'s [&'m TestMeta<Extra>])>
    where
        'm: 's,
        GroupKey: 's,
        Extra: 'm,
    {
        self.iter().map(|(k, v)| (k, v.as_slice()))
    }
}

pub type TestGroupBTreeMap<'m, GroupKey, Extra> = BTreeMap<GroupKey, Vec<&'m TestMeta<Extra>>>;

impl<'m, GroupKey, Extra> TestGroups<'m, GroupKey, Extra> for TestGroupBTreeMap<'m, GroupKey, Extra>
where
    GroupKey: Ord,
{
    fn add(&mut self, key: GroupKey, meta: &'m TestMeta<Extra>) {
        self.entry(key).or_default().push(meta);
    }

    fn len(&self) -> usize {
        self.values().map(|g| g.len()).sum()
    }

    fn iter<'s>(&'s self) -> impl Iterator<Item = (&'s GroupKey, &'s [&'m TestMeta<Extra>])>
    where
        'm: 's,
        GroupKey: 's,
        Extra: 'm,
    {
        self.iter().map(|(k, v)| (k, v.as_slice()))
    }
}

pub trait TestGroupRunner<GroupKey, Extra> {
    fn run_group(&self, key: GroupKey, tests: &[&TestMeta<Extra>]);
}
