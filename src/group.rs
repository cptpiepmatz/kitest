use std::{
    collections::{BTreeMap, HashMap},
    hash::{BuildHasher, Hash},
};

use crate::meta::{Test, TestMeta};

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

pub trait TestGroups<'m, GroupKey, Extra>:
    IntoIterator<Item = (GroupKey, Vec<&'m Test<Extra>>)>
where
    Extra: 'm,
{
    fn add(&mut self, key: GroupKey, test: &'m Test<Extra>);

    fn len(&self) -> usize;

    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

pub type TestGroupHashMap<'m, GroupKey, Extra, RandomState = std::hash::RandomState> =
    HashMap<GroupKey, Vec<&'m Test<Extra>>, RandomState>;

impl<'m, GroupKey, Extra, RandomState> TestGroups<'m, GroupKey, Extra>
    for TestGroupHashMap<'m, GroupKey, Extra, RandomState>
where
    GroupKey: Eq + Hash,
    RandomState: BuildHasher + Default,
{
    fn add(&mut self, key: GroupKey, test: &'m Test<Extra>) {
        self.entry(key).or_default().push(test);
    }

    fn len(&self) -> usize {
        self.values().map(|g| g.len()).sum()
    }
}

pub type TestGroupBTreeMap<'m, GroupKey, Extra> = BTreeMap<GroupKey, Vec<&'m Test<Extra>>>;

impl<'m, GroupKey, Extra> TestGroups<'m, GroupKey, Extra> for TestGroupBTreeMap<'m, GroupKey, Extra>
where
    GroupKey: Ord,
{
    fn add(&mut self, key: GroupKey, test: &'m Test<Extra>) {
        self.entry(key).or_default().push(test);
    }

    fn len(&self) -> usize {
        self.values().map(|g| g.len()).sum()
    }
}

pub trait TestGroupRunner<GroupKey, Extra> {
    fn run_group<F, T>(&self, key: &GroupKey, f: F) -> T
    where
        F: FnOnce() -> T;
}

#[derive(Default)]
pub struct SimpleGroupRunner;

impl<GroupKey, Extra> TestGroupRunner<GroupKey, Extra> for SimpleGroupRunner {
    fn run_group<F, T>(&self, _: &GroupKey, f: F) -> T
    where
        F: FnOnce() -> T,
    {
        f()
    }
}
