use crate::test::TestMeta;

/// A strategy for assigning tests to groups.
///
/// The grouper maps each test to a `GroupKey`. The key is used by the grouped
/// harness to collect tests into groups. A `GroupKey` is typically something
/// small and cheap to clone, like a string, integer, or small enum. It should
/// at least support equality so the harness can decide which tests belong to
/// the same group.
///
/// In addition to the key, a grouper can optionally provide group context via
/// [`Self::group_ctx`]. This is meant for heavier or richer group data (for
/// example a display label or configuration) that we do not want to compute or
/// store on every test. The context type does not need to implement many traits.
///
/// For simple setups where no context is needed, `TestGrouper` is implemented
/// for `Fn(&TestMeta<Extra>) -> GroupKey`, so a closure can act as a grouper.
pub trait TestGrouper<Extra, GroupKey, GroupCtx = ()> {
    /// Return the group key for a test.
    fn group(&mut self, meta: &TestMeta<Extra>) -> GroupKey;

    /// Optionally return group context for a group key.
    ///
    /// The default implementation returns `None`.
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
