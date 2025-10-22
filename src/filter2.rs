use crate::meta::TestMeta;

pub struct FilteredTests<'m, I, Extra>
where
    I: Iterator<Item = &'m TestMeta<Extra>>,
    Extra: 'm,
{
    tests: I,
    filtered: usize,
}

pub trait TestFilter<Extra> {
    fn filter<'m>(
        &self,
        tests: &'m [TestMeta<Extra>],
    ) -> FilteredTests<'m, impl Iterator<Item = &'m TestMeta<Extra>>, Extra>;
}

pub struct NoFilter;

impl<Extra> TestFilter<Extra> for NoFilter {
    fn filter<'m>(
        &self,
        tests: &'m [TestMeta<Extra>],
    ) -> FilteredTests<'m, impl Iterator<Item = &'m TestMeta<Extra>>, Extra> {
        FilteredTests {
            tests: tests.iter(),
            filtered: 0,
        }
    }
}
