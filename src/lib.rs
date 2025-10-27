use std::{collections::HashMap, time::Duration};

use crate::{
    filter::{FilteredTests, TestFilter},
    formatter::{
        FmtBeginListing, FmtEndListing, FmtInitListing, FmtListGroupEnd, FmtListGroupStart,
        FmtListGroups, FmtListTest, GroupedTestListFormatter,
    },
    group::{TestGrouper, TestGroups},
    ignore::TestIgnore,
    outcome::TestOutcome,
    test::Test,
};

pub mod formatter;
pub mod outcome;
pub mod test;

mod strategy;
pub use strategy::*;

mod harness;
pub use harness::*;

mod report;
pub use report::*;

#[test]
fn foo() {}

#[test]
fn bar() {}

#[test]
#[ignore = "for a reason"]
fn ignored() {}
