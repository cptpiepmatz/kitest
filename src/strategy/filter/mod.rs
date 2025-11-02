use crate::test::Test;

mod no;
pub use no::*;

mod default;
pub use default::*;

#[derive(Debug)]
pub struct FilteredTests<'t, I, Extra>
where
    I: ExactSizeIterator<Item = &'t Test<Extra>>,
    Extra: 't,
{
    pub tests: I,
    pub filtered_out: usize,
}

pub trait TestFilter<Extra> {
    fn filter<'t>(
        &self,
        tests: &'t [Test<Extra>],
    ) -> FilteredTests<'t, impl ExactSizeIterator<Item = &'t Test<Extra>>, Extra>;
}
