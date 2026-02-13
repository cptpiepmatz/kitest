use crate::formatter::*;

/// A formatter that produces no output.
///
/// `NoFormatter` implements all formatter and list formatter traits, but discards
/// every event. This is useful when we want to run or list tests without any
/// formatting (for example in benchmarks or when integrating kitest into another
/// system that handles its own reporting).
#[derive(Debug, Default, Clone)]
pub struct NoFormatter;

macro_rules! impl_unit_from {
    [$($name:ident$(<$($generic:tt),*>)?),* $(,)?] => {$(
        impl$(<$($generic),*>)? From<$name$(<$($generic),*>)?> for () {
            fn from(_: $name$(<$($generic),*>)?) -> () {}
        })*
    };
}

impl_unit_from![
    FmtRunInit<'t, Extra>,
    FmtRunStart,
    FmtTestIgnored<'t, 'r, Extra>,
    FmtTestStart<'t, Extra>,
    FmtTestOutcome<'t, 'o, Extra>,
    FmtRunOutcomes<'t, 'o>,
    FmtGroupedRunStart,
    FmtGroupStart<'g, GroupKey, GroupCtx>,
    FmtGroupOutcomes<'t, 'g, 'o, GroupKey, GroupCtx>,
    FmtGroupedRunOutcomes<'t, 'o, GroupKey>,
    FmtInitListing<'t, Extra>,
    FmtBeginListing,
    FmtListTest<'t, Extra>,
    FmtEndListing,
    FmtListGroups,
    FmtListGroupStart<'g, GroupKey, GroupCtx>,
    FmtListGroupEnd<'g, GroupKey, GroupCtx>,
];

impl<'t, Extra: 't> TestFormatter<'t, Extra> for NoFormatter {
    type Error = ();
    type RunInit = ();
    type RunStart = ();
    type TestIgnored = ();
    type TestStart = ();
    type TestOutcome = ();
    type RunOutcomes = ();
}

impl<'t, Extra: 't, GroupKey: 't, GroupCtx: 't> GroupedTestFormatter<'t, Extra, GroupKey, GroupCtx>
    for NoFormatter
{
    type GroupedRunStart = ();
    type GroupStart = ();
    type GroupOutcomes = ();
    type GroupedRunOutcomes = ();
}

impl<'t, Extra: 't> TestListFormatter<'t, Extra> for NoFormatter {
    type Error = ();
    type InitListing = ();
    type BeginListing = ();
    type ListTest = ();
    type EndListing = ();
}

impl<'t, Extra: 't, GroupKey: 't, GroupCtx: 't>
    GroupedTestListFormatter<'t, Extra, GroupKey, GroupCtx> for NoFormatter
{
    type ListGroups = ();
    type ListGroupStart = ();
    type ListGroupEnd = ();
}
