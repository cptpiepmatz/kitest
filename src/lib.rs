use crate::{
    filter::TestFilter,
    group::{TestGroupRunner, TestGrouper, TestGroups},
    ignore::TestIgnore,
    meta::TestMeta,
    panic_handler::TestPanicHandler,
    runner::TestRunner,
};
use itertools::Itertools;

pub mod filter;
pub mod group;
pub mod ignore;
pub mod meta;
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

pub fn run_tests<
    'm,
    Iter: Iterator<Item = &'m TestMeta<Extra>>,
    Filter: TestFilter<Extra>,
    Runner: TestRunner<Extra>,
    Ignore: TestIgnore<Extra> + Sync,
    PanicHandler: TestPanicHandler<Extra> + Sync,
    Extra: 'm + Sync,
>(
    tests: Iter,
    filter: Filter,
    runner: Runner,
    ignore: Ignore,
    panic_handler: PanicHandler,
) {
    let mut filtered = 0;
    let tests = match filter.skip_filtering() {
        true => tests.collect_vec(),
        false => tests.flat_map(|meta| match filter.filter(meta) {
            true => Some(meta),
            false => {
                filtered += 1;
                None
            }
        }).collect_vec(),
    };

    let test_runs = tests.iter().map(|meta| {
        || {
            let (ignored, reason) = ignore.ignore(meta);
            if ignored {
                // TODO: report ignored
                return;
            };

            let test_result = panic_handler.handle(*meta);
        }
    });

    runner.run(test_runs);
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
    GroupKey,
    Extra: 'm + Sync,
>(
    tests: Iter,
    filter: Filter,
    grouper: Grouper,
    mut groups: Groups,
    group_runner: GroupRunner,
    runner: Runner,
    ignore: Ignore,
    panic_handler: PanicHandler,
) {
    let mut filtered = 0;
    match filter.skip_filtering() {
        true => tests.for_each(|meta| {
            let key = grouper.group(meta);
            groups.add(key, meta);
        }),
        false => tests.for_each(|meta| {
            if !filter.filter(meta) {
                filtered += 1;
                return;
            }

            let key = grouper.group(meta);
            groups.add(key, meta);
        }),
    }

    for (key, tests) in groups.iter() {
        group_runner.run_group(key, || {
            let test_runs = tests.iter().map(|meta| {
                || {
                    let (ignored, reason) = ignore.ignore(meta);
                    if ignored {
                        // TODO: report ignored
                        return;
                    };

                    let test_result = panic_handler.handle(*meta);
                }
            });

            runner.run(test_runs);
        });
    }
}
