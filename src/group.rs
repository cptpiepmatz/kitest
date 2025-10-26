use std::{
    collections::{BTreeMap, HashMap},
    hash::{BuildHasher, Hash},
};

use crate::meta::{Test, TestMeta};

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

pub trait TestGroups<'m, Extra, GroupKey>:
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

pub type TestGroupHashMap<'m, Extra, GroupKey, RandomState = std::hash::RandomState> =
    HashMap<GroupKey, Vec<&'m Test<Extra>>, RandomState>;

impl<'m, Extra, GroupKey, RandomState> TestGroups<'m, Extra, GroupKey>
    for TestGroupHashMap<'m, Extra, GroupKey, RandomState>
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

pub type TestGroupBTreeMap<'m, Extra, GroupKey> = BTreeMap<GroupKey, Vec<&'m Test<Extra>>>;

impl<'m, Extra, GroupKey> TestGroups<'m, Extra, GroupKey> for TestGroupBTreeMap<'m, Extra, GroupKey>
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

pub trait TestGroupRunner<Extra, GroupKey, GroupCtx> {
    fn run_group<F, T>(&self, f: F, key: &GroupKey, ctx: Option<&GroupCtx>) -> T
    where
        F: FnOnce() -> T;
}

#[derive(Default)]
pub struct SimpleGroupRunner;

impl<Extra, GroupKey, GroupCtx> TestGroupRunner<Extra, GroupKey, GroupCtx> for SimpleGroupRunner {
    fn run_group<F, T>(&self, f: F, _: &GroupKey, _: Option<&GroupCtx>) -> T
    where
        F: FnOnce() -> T,
    {
        f()
    }
}
