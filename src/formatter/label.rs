use std::{fmt::Display, marker::PhantomData};

use crate::formatter::FmtGroupStart;

#[derive(Debug, Default, Clone, Copy, Hash)]
pub struct FromGroupKey;

#[derive(Debug, Default, Clone, Copy, Hash)]
pub struct FromGroupCtx;

#[derive(Debug, Default, Clone, Hash)]
pub struct GroupLabel<M> {
    marker: PhantomData<M>,
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

impl<M> Display for GroupLabel<M> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.label)
    }
}
