use crate::{filter::TestFilter, meta::TestMeta};

pub mod meta;
pub mod filter;
pub mod ignore;
pub mod grouper;

pub struct TestExecutor<'t, Iter, Filter, Extra = ()>
where
    Iter: Iterator<Item = &'t TestMeta<Extra>>,
    Filter: TestFilter<Extra>,
    Extra: 't,
{
    tests: Iter,
    filter: Filter,
}
