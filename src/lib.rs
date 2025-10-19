use crate::{filter::TestFilter, meta::TestMeta};

pub mod meta;
pub mod filter;
pub mod ignore;
pub mod group;
pub mod panic_handler;

pub struct TestExecutor<'t, Iter, Filter, Extra = ()>
where
    Iter: Iterator<Item = &'t TestMeta<Extra>>,
    Filter: TestFilter<Extra>,
    Extra: 't,
{
    tests: Iter,
    filter: Filter,
}

pub trait TestRunner<Extra> {
    fn run(&self, tests: &[&TestMeta<Extra>]);
}
