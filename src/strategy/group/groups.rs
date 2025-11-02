use std::{
    collections::{BTreeMap, HashMap},
    hash::{BuildHasher, Hash},
};

use crate::test::Test;

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
