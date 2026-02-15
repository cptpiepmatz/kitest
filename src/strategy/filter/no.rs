use crate::{
    filter::{FilteredTests, TestFilter},
    test::Test,
};

/// A [`TestFilter`] that does not filter out any tests.
///
/// All input tests are included in the run, and no tests are counted as filtered out.
#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub struct NoFilter;

impl NoFilter {
    pub fn new() -> Self {
        Self
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
