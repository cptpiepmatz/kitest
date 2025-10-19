use crate::{filter::TestFilter, meta::TestMeta};

pub mod meta;
pub mod filter;

pub struct TestExecutor<'t, Iter, Filter, Extra = ()>
where
    Iter: Iterator<Item = &'t TestMeta<Extra>>,
    Filter: TestFilter<Extra>,
    Extra: 't,
{
    tests: Iter,
    filter: Filter,
}
