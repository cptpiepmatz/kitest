use std::{
    collections::{BTreeMap, HashMap},
    hash::{BuildHasher, Hash},
};

use crate::test::{Test, TestMeta};

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

pub trait TestGroups<'t, Extra: 't, GroupKey> {
    fn add(&mut self, key: GroupKey, test: &'t Test<Extra>);

    fn into_groups(
        self,
    ) -> impl ExactSizeIterator<Item = (GroupKey, impl ExactSizeIterator<Item = &'t Test<Extra>>)>;

    fn len(&self) -> usize;

    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

pub type TestGroupHashMap<'t, Extra, GroupKey, RandomState = std::hash::RandomState> =
    HashMap<GroupKey, Vec<&'t Test<Extra>>, RandomState>;

impl<'t, Extra: 't, GroupKey, RandomState> TestGroups<'t, Extra, GroupKey>
    for TestGroupHashMap<'t, Extra, GroupKey, RandomState>
where
    GroupKey: Eq + Hash,
    RandomState: BuildHasher + Default,
{
    fn add(&mut self, key: GroupKey, test: &'t Test<Extra>) {
        self.entry(key).or_default().push(test);
    }

    fn into_groups(
        self,
    ) -> impl ExactSizeIterator<Item = (GroupKey, impl ExactSizeIterator<Item = &'t Test<Extra>>)>
    {
        self.into_iter()
            .map(|(key, tests)| (key, tests.into_iter()))
    }

    fn len(&self) -> usize {
        self.values().map(|g| g.len()).sum()
    }
}

pub type TestGroupBTreeMap<'t, Extra, GroupKey> = BTreeMap<GroupKey, Vec<&'t Test<Extra>>>;

impl<'t, Extra: 't, GroupKey> TestGroups<'t, Extra, GroupKey>
    for TestGroupBTreeMap<'t, Extra, GroupKey>
where
    GroupKey: Ord,
{
    fn add(&mut self, key: GroupKey, test: &'t Test<Extra>) {
        self.entry(key).or_default().push(test);
    }

    fn into_groups(
        self,
    ) -> impl ExactSizeIterator<Item = (GroupKey, impl ExactSizeIterator<Item = &'t Test<Extra>>)>
    {
        self.into_iter()
            .map(|(key, tests)| (key, tests.into_iter()))
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
