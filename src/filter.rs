use std::{slice, vec};

use crate::meta::Test;

pub struct FilteredTests<'m, I, Extra>
where
    I: ExactSizeIterator<Item = &'m Test<Extra>> + Send,
    Extra: Sync + 'm,
{
    pub tests: I,
    pub filtered: usize,
}

pub trait TestFilter<Extra: Sync> {
    fn filter<'m>(
        &self,
        tests: &'m [Test<Extra>],
    ) -> FilteredTests<'m, impl ExactSizeIterator<Item = &'m Test<Extra>> + Send, Extra>;
}

pub struct NoFilter;

impl<Extra: Sync> TestFilter<Extra> for NoFilter {
    fn filter<'m>(
        &self,
        tests: &'m [Test<Extra>],
    ) -> FilteredTests<'m, impl ExactSizeIterator<Item = &'m Test<Extra>> + Send, Extra> {
        FilteredTests {
            tests: tests.iter(),
            filtered: 0,
        }
    }
}

#[derive(Default)]
pub struct DefaultFilter {
    exact: bool,
    filter: Vec<String>,
    skip: Vec<String>,
}

enum DefaultFilterIterator<'m, Extra> {
    Slice(slice::Iter<'m, Test<Extra>>),
    Vec(vec::IntoIter<&'m Test<Extra>>),
}

impl<'m, Extra> Iterator for DefaultFilterIterator<'m, Extra> {
    type Item = &'m Test<Extra>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            DefaultFilterIterator::Slice(iter) => iter.next(),
            DefaultFilterIterator::Vec(iter) => iter.next(),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match self {
            DefaultFilterIterator::Slice(iter) => iter.size_hint(),
            DefaultFilterIterator::Vec(iter) => iter.size_hint(),
        }
    }
}

impl<'m, Extra> ExactSizeIterator for DefaultFilterIterator<'m, Extra> {}

impl<Extra: Sync> TestFilter<Extra> for DefaultFilter {
    fn filter<'m>(
        &self,
        tests: &'m [Test<Extra>],
    ) -> FilteredTests<'m, impl ExactSizeIterator<Item = &'m Test<Extra>> + Send, Extra> {
        if self.filter.is_empty() && self.skip.is_empty() {
            return FilteredTests {
                tests: DefaultFilterIterator::Slice(tests.iter()),
                filtered: 0,
            };
        }

        if self.exact {
            let mut remaining = Vec::new();
            let mut filtered = 0;
            for meta in tests {
                let name = meta.name.as_ref();
                let in_filter =
                    self.filter.is_empty() || self.filter.iter().any(|filter| name == filter);

                if !in_filter {
                    filtered += 1;
                    continue;
                }

                let skipped = self.skip.iter().any(|skip| name == skip);
                match skipped {
                    true => filtered += 1,
                    false => remaining.push(meta),
                }
            }
            return FilteredTests {
                tests: DefaultFilterIterator::Vec(remaining.into_iter()),
                filtered,
            };
        }

        let mut remaining = Vec::new();
        let mut filtered = 0;
        for meta in tests {
            let name = meta.name.as_ref();
            let in_filter =
                self.filter.is_empty() || self.filter.iter().any(|filter| name.contains(filter));

            if !in_filter {
                filtered += 1;
                continue;
            }

            let skipped = self.skip.iter().any(|skip| name.contains(skip));
            match skipped {
                true => filtered += 1,
                false => remaining.push(meta),
            }
        }
        FilteredTests {
            tests: DefaultFilterIterator::Vec(remaining.into_iter()),
            filtered,
        }
    }
}
