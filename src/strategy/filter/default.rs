use std::{slice, vec};

use crate::{
    filter::{FilteredTests, TestFilter},
    test::Test,
};

#[derive(Debug, Default)]
pub struct DefaultFilter {
    exact: bool,
    filter: Vec<String>,
    skip: Vec<String>,
}

#[derive(Debug)]
enum DefaultFilterIterator<'t, Extra> {
    Slice(slice::Iter<'t, Test<Extra>>),
    Vec(vec::IntoIter<&'t Test<Extra>>),
}

impl<'t, Extra> Iterator for DefaultFilterIterator<'t, Extra> {
    type Item = &'t Test<Extra>;

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

impl<'t, Extra> ExactSizeIterator for DefaultFilterIterator<'t, Extra> {}

impl<Extra> TestFilter<Extra> for DefaultFilter {
    fn filter<'t>(
        &self,
        tests: &'t [Test<Extra>],
    ) -> FilteredTests<'t, impl ExactSizeIterator<Item = &'t Test<Extra>>, Extra> {
        if self.filter.is_empty() && self.skip.is_empty() {
            return FilteredTests {
                tests: DefaultFilterIterator::Slice(tests.iter()),
                filtered_out: 0,
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
                filtered_out: filtered,
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
            filtered_out: filtered,
        }
    }
}
