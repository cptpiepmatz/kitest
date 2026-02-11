use std::{
    collections::{BTreeMap, HashMap},
    hash::{BuildHasher, Hash},
};

use crate::test::Test;

/// Storage abstraction for grouped tests.
///
/// `TestGroups` defines how tests are collected into groups and later turned into an iterator of
/// groups.
/// Different data structures can be used depending on the desired ordering or performance
/// characteristics.
pub trait TestGroups<'t, Extra: 't, GroupKey> {
    /// Add a test to the group identified by `key`.
    fn add(&mut self, key: GroupKey, test: &'t Test<Extra>);

    /// Consume the storage and return an iterator over groups.
    ///
    /// Each item consists of the group key and an iterator over the tests in
    /// that group.
    fn into_groups(
        self,
    ) -> impl ExactSizeIterator<Item = (GroupKey, impl ExactSizeIterator<Item = &'t Test<Extra>>)>;

    /// Return the total number of tests across all groups.
    ///
    /// This counts all tests contained in the groups, not the number of groups.
    fn len(&self) -> usize;

    /// Return `true` if there are no tests in any group.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// A [`TestGroups`] implementation backed by [`HashMap`].
///
/// This requires `GroupKey: Eq + Hash` and does not guarantee any ordering
/// of groups.
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

/// A [`TestGroups`] implementation backed by [`BTreeMap`].
///
/// This requires `GroupKey: Ord` and yields groups ordered by their key.
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
