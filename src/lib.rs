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

trait FmtErrors<E> {
    fn push_on_error<T>(&mut self, data: (&'static str, Result<T, E>));
}

impl<E> FmtErrors<E> for Vec<(&'static str, E)> {
    fn push_on_error<T>(&mut self, (name, res): (&'static str, Result<T, E>)) {
        if let Err(err) = res {
            self.push((name, err));
        }
    }
}

macro_rules! named_fmt {
    ($fmt:ident.$method:ident($expr:expr)) => {
        (stringify!($method), $fmt.$method($expr))
    };
}

pub type TestOutcomes<'t> = HashMap<&'t str, TestOutcome, ahash::RandomState>;

#[non_exhaustive]
pub struct TestReport<'t, FmtError: 't> {
    pub outcomes: TestOutcomes<'t>,
    pub duration: Duration,
    pub fmt_errors: Vec<(&'static str, FmtError)>,
}

pub type GroupedTestOutcomes<'t, GroupKey> =
    HashMap<GroupKey, HashMap<&'t str, TestOutcome, ahash::RandomState>, ahash::RandomState>;

#[non_exhaustive]
pub struct GroupedTestReport<'t, GroupKey, FmtError: 't> {
    pub outcomes: GroupedTestOutcomes<'t, GroupKey>,
    pub duration: Duration,
    pub fmt_errors: Vec<(&'static str, FmtError)>,
}

pub fn list_grouped_tests<
    't,
    Filter: TestFilter<Extra>,
    Ignore: TestIgnore<Extra>,
    Grouper: TestGrouper<Extra, GroupKey, GroupCtx>,
    Groups: TestGroups<'t, Extra, GroupKey>,
    Formatter: GroupedTestListFormatter<'t, Extra, GroupKey, GroupCtx>,
    Extra: Sync + 't,
    GroupKey: 't,
    GroupCtx: 't,
>(
    tests: &'t [Test<Extra>],
    filter: Filter,
    ignore: Ignore,
    mut grouper: Grouper,
    mut groups: Groups,
    mut formatter: Formatter,
) -> Vec<(&'static str, Formatter::Error)> {
    let mut fmt_errors = Vec::new();
    fmt_errors.push_on_error(named_fmt!(
        formatter.fmt_init_listing(FmtInitListing { tests }.into())
    ));

    let FilteredTests { tests, filtered } = filter.filter(tests);
    fmt_errors.push_on_error(named_fmt!(
        formatter.fmt_begin_listing(
            FmtBeginListing {
                tests: tests.len(),
                filtered
            }
            .into()
        )
    ));

    tests.for_each(|test| groups.add(grouper.group(test), test));
    let groups = groups.into_groups();
    fmt_errors.push_on_error(named_fmt!(
        formatter.fmt_list_groups(
            FmtListGroups {
                groups: groups.len()
            }
            .into()
        )
    ));

    let mut active_count = 0;
    let mut ignore_count = 0;
    for (key, tests) in groups {
        let ctx = grouper.group_ctx(&key);
        let tests_len = tests.len();

        fmt_errors.push_on_error(named_fmt!(
            formatter.fmt_list_group_start(
                FmtListGroupStart {
                    tests: tests_len,
                    key: &key,
                    ctx
                }
                .into()
            )
        ));

        for test in tests {
            let ignored = ignore.ignore(test);
            match ignored.0 {
                true => ignore_count += 1,
                false => active_count += 1,
            }
            fmt_errors.push_on_error(named_fmt!(
                formatter.fmt_list_test(
                    FmtListTest {
                        meta: test,
                        ignored
                    }
                    .into()
                )
            ));
        }

        fmt_errors.push_on_error(named_fmt!(
            formatter.fmt_list_group_end(
                FmtListGroupEnd {
                    tests: tests_len,
                    key: &key,
                    ctx
                }
                .into()
            )
        ));
    }

    fmt_errors.push_on_error(named_fmt!(
        formatter.fmt_end_listing(
            FmtEndListing {
                active: active_count,
                ignored: ignore_count
            }
            .into()
        )
    ));

    fmt_errors
}

#[test]
fn foo() {}

#[test]
fn bar() {}

#[test]
#[ignore = "for a reason"]
fn ignored() {}
