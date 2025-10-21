use std::{collections::HashMap, hash::Hash, sync::Arc, thread};

use crate::{
    filter::{FilterDecision, TestFilter},
    group::{TestGroupRunner, TestGrouper, TestGroups},
    ignore::TestIgnore,
    meta::TestMeta,
    outcome::{TestOutcome, TestStatus},
    panic_handler::TestPanicHandler,
    runner::TestRunner,
};
use itertools::Itertools;

pub mod filter;
pub mod formatter;
pub mod group;
pub mod ignore;
pub mod meta;
pub mod outcome;
pub mod panic_handler;
pub mod runner;

pub struct TestExecutor<'t, Iter, Filter, Extra = ()>
where
    Iter: Iterator<Item = &'t TestMeta<Extra>>,
    Filter: TestFilter<Extra>,
    Extra: 't,
{
    tests: Iter,
    filter: Filter,
}

fn apply_filter<'m, Iter, Filter, Extra>(
    tests: Iter,
    mut filter: Filter,
) -> (Vec<&'m TestMeta<Extra>>, usize)
where
    Iter: Iterator<Item = &'m TestMeta<Extra>>,
    Filter: TestFilter<Extra>,
{
    let mut filtered = 0usize;
    let mut kept = Vec::new();

    let mut iter = tests.into_iter();

    if filter.skip_filtering() {
        kept = iter.collect_vec();
        return (kept, filtered);
    }

    while let Some(meta) = iter.next() {
        match filter.filter(&meta) {
            FilterDecision::Keep => {
                kept.push(meta);
            }
            FilterDecision::Exclude => {
                filtered += 1;
            }
            FilterDecision::KeepAndDone => {
                kept.push(meta);
                filtered += iter.count(); // everything remaining is filtered out
                break;
            }
            FilterDecision::ExcludeAndDone => {
                filtered += 1 + iter.count(); // this one plus the rest
                break;
            }
        }
    }

    (kept, filtered)
}

pub fn run_tests<
    'm,
    Iter: Iterator<Item = &'m TestMeta<Extra>>,
    Filter: TestFilter<Extra>,
    Runner: TestRunner<Extra>,
    Ignore: TestIgnore<Extra> + Send + Sync + 'm,
    PanicHandler: TestPanicHandler<Extra> + Send + Sync + 'm,
    Extra: 'm + Sync,
>(
    tests: Iter,
    filter: Filter,
    runner: Runner,
    ignore: Ignore,
    panic_handler: PanicHandler,
) -> TestReport<'m> {
    let (tests, filtered) = apply_filter(tests, filter);

    // fmt_start(tests: &[&TestMeta], filtered: usize)

    let ignore = Arc::new(ignore);
    let panic_handler = Arc::new(panic_handler);

    let report: TestReport<'_> = std::thread::scope(move |scope| {
        let test_runs = tests.into_iter().map(|meta| {
            let ignore = Arc::clone(&ignore);
            let panic_handler = Arc::clone(&panic_handler);

            (
                move || {
                    let (ignored, reason) = ignore.ignore(meta);
                    if ignored {
                        return TestStatus::Ignored { reason };
                    }

                    println!("before {}", meta.name);
                    let test_status = panic_handler.handle(meta);
                    println!("after {}", meta.name);
                    test_status
                },
                meta,
            )
        });

        TestReport(runner.run(test_runs, scope).collect())
    });

    println!("got report");
    // fmt_report()

    report
}

fn apply_grouped_filter<
    'm,
    Iter: Iterator<Item = &'m TestMeta<Extra>>,
    Filter: TestFilter<Extra>,
    Grouper: TestGrouper<GroupKey, Extra>,
    Groups: TestGroups<'m, GroupKey, Extra>,
    GroupKey,
    Extra: 'm,
>(
    mut tests: Iter,
    mut filter: Filter,
    grouper: Grouper,
    mut groups: Groups,
) -> (Groups, usize) {
    let mut filtered = 0;

    if filter.skip_filtering() {
        tests.for_each(|meta| groups.add(grouper.group(meta), meta));
        return (groups, filtered);
    }

    while let Some(meta) = tests.next() {
        match filter.filter(meta) {
            FilterDecision::Keep => groups.add(grouper.group(meta), meta),
            FilterDecision::Exclude => filtered += 1,
            FilterDecision::KeepAndDone => {
                groups.add(grouper.group(meta), meta);
                filtered += tests.count();
                return (groups, filtered);
            },
            FilterDecision::ExcludeAndDone => {
                filtered += 1 + tests.count();
                return (groups, filtered);
            },
        }
    }

    (groups, filtered)
}

pub fn run_grouped_tests<
    'm,
    Iter: Iterator<Item = &'m TestMeta<Extra>>,
    Filter: TestFilter<Extra>,
    Grouper: TestGrouper<GroupKey, Extra>,
    Groups: TestGroups<'m, GroupKey, Extra>,
    GroupRunner: TestGroupRunner<GroupKey, Extra>,
    Runner: TestRunner<Extra>,
    Ignore: TestIgnore<Extra> + Sync,
    PanicHandler: TestPanicHandler<Extra> + Sync,
    GroupKey: Eq + Hash,
    Extra: 'm + Sync,
>(
    tests: Iter,
    filter: Filter,
    grouper: Grouper,
    groups: Groups,
    group_runner: GroupRunner,
    runner: Runner,
    ignore: Ignore,
    panic_handler: PanicHandler,
) {
    let (groups, filtered) = apply_grouped_filter(tests, filter, grouper, groups);

    // ftm_grouped_start(&groups: impl Groups, filtered: usize)

    let group_runs = groups.into_iter().map(|(key, tests)| {
        let report = group_runner.run_group(&key, || {
            let test_runs = tests.iter().map(|meta| {
                (
                    || {
                        let (ignored, reason) = ignore.ignore(meta);
                        if ignored {
                            // fmt_ignored(meta: &TestMeta, reason: &str)
                            return TestStatus::Ignored { reason };
                        };

                        // fmt_start_test(meta: &TestMeta)
                        let test_status = panic_handler.handle(*meta);
                        // fmt_test_result(meta: &TestMeta, result: &TestResult)

                        test_status
                    },
                    *meta,
                )
            });

            thread::scope(|scope| runner.run(test_runs, scope).collect())
        });

        (key, report)
    });

    let report = GroupedTestReport(group_runs.collect());

    // fmt_grouped_report()

    // report
}

pub struct TestReport<'m>(HashMap<&'m str, TestOutcome, ahash::RandomState>);

pub struct GroupedTestReport<'m, GroupKey>(
    HashMap<GroupKey, HashMap<&'m str, TestOutcome, ahash::RandomState>, ahash::RandomState>,
);

#[test]
fn foo() {}

#[test]
fn bar() {}
