use crate::{
    filter::{FilteredTests, TestFilter},
    test::Test,
};

#[derive(Debug, Default)]
pub struct NoFilter;

impl NoFilter {
    pub fn new() -> Self {
        Self::default()
    }
}

impl<Extra: Sync> TestFilter<Extra> for NoFilter {
    fn filter<'t>(
        &self,
        tests: &'t [Test<Extra>],
    ) -> FilteredTests<'t, impl ExactSizeIterator<Item = &'t Test<Extra>>, Extra> {
        FilteredTests {
            tests: tests.iter(),
            filtered_out: 0,
        }
    }
}
