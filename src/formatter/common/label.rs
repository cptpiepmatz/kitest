//! Utilities for deriving display labels for groups.

use std::{fmt::Display, marker::PhantomData};

use crate::formatter::FmtGroupStart;

/// Marker type indicating that a group label should be derived from the group key.
///
/// Requires the `GroupKey` to implement [`Display`].
#[derive(Debug, Default, Clone, Copy, Hash, PartialEq, Eq)]
pub struct FromGroupKey;

/// Marker type indicating that a group label should be derived from the group context.
///
/// Requires the `GroupCtx` to implement [`Display`].
#[derive(Debug, Default, Clone, Copy, Hash, PartialEq, Eq)]
pub struct FromGroupCtx;

/// A displayable label for a test group.
///
/// - `GroupLabel<FromGroupKey>` derives its label from the group key
/// - `GroupLabel<FromGroupCtx>` derives its label from the group context
///
/// In both cases, the respective type must implement [`Display`].
#[derive(Debug, Default, Clone, Hash, PartialEq, Eq)]
pub struct GroupLabel<M> {
    marker: PhantomData<M>,

    /// The computed label string.
    pub label: String,
}

impl<'g, GroupKey: Display, GroupCtx> From<&FmtGroupStart<'g, GroupKey, GroupCtx>>
    for GroupLabel<FromGroupKey>
{
    fn from(value: &FmtGroupStart<'g, GroupKey, GroupCtx>) -> Self {
        GroupLabel {
            marker: PhantomData,
            label: value.key.to_string(),
        }
    }
}

impl<GroupKey: Display, GroupCtx> From<(&GroupKey, Option<&GroupCtx>)>
    for GroupLabel<FromGroupKey>
{
    fn from(value: (&GroupKey, Option<&GroupCtx>)) -> Self {
        GroupLabel {
            marker: PhantomData,
            label: value.0.to_string(),
        }
    }
}

// TODO: implement this conversion for other types

impl<'g, GroupKey, GroupCtx: Display> From<&FmtGroupStart<'g, GroupKey, GroupCtx>>
    for GroupLabel<FromGroupCtx>
{
    fn from(value: &FmtGroupStart<'g, GroupKey, GroupCtx>) -> Self {
        GroupLabel {
            marker: PhantomData,
            label: value.ctx.map(|ctx| ctx.to_string()).unwrap_or_default(),
        }
    }
}

impl<GroupKey, GroupCtx: Display> From<(&GroupKey, Option<&GroupCtx>)>
    for GroupLabel<FromGroupCtx>
{
    fn from(value: (&GroupKey, Option<&GroupCtx>)) -> Self {
        GroupLabel {
            marker: PhantomData,
            label: value.1.map(|ctx| ctx.to_string()).unwrap_or_default(),
        }
    }
}

impl<M> Display for GroupLabel<M> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.label)
    }
}
