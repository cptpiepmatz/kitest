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

impl DefaultFilter {
    pub fn with_exact(self, exact: bool) -> Self {
        Self { exact, ..self }
    }

    pub fn with_filter(self, filter: impl IntoIterator<Item = impl Into<String>>) -> Self {
        Self {
            filter: filter.into_iter().map(Into::into).collect(),
            ..self
        }
    }

    pub fn append_filter(&mut self, filter: impl IntoIterator<Item = impl Into<String>>) {
        self.filter.extend(filter.into_iter().map(Into::into));
    }

    pub fn with_skip(self, skip: impl IntoIterator<Item = impl Into<String>>) -> Self {
        Self {
            skip: skip.into_iter().map(Into::into).collect(),
            ..self
        }
    }

    pub fn append_skip(&mut self, skip: impl IntoIterator<Item = impl Into<String>>) {
        self.skip.extend(skip.into_iter().map(Into::into));
    }
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

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;
    use crate::test_support::*;

    #[test]
    fn empty_filter_allows_everything() {
        let tests: Vec<_> = (0..10)
            .map(|idx| test! {name: format!("test_{idx}")})
            .collect();

        let report = harness(&tests).with_filter(DefaultFilter::default()).run();
        assert_eq!(report.outcomes.len(), 10);
    }

    #[test]
    fn filtering_tests_works() {
        let tests = &[
            test! {name: "cool_test"},
            test! {name: "boring_test"},
            test! {name: "crazy_test"},
            test! {name: "super_cool_test"},
        ];

        let report = harness(tests)
            .with_filter(DefaultFilter::default().with_filter(["cool"]))
            .run();

        let filtered_tests: HashSet<_> = report.outcomes.into_iter().map(|(n, _)| n).collect();
        assert!(filtered_tests.contains("cool_test"));
        assert!(!filtered_tests.contains("boring_test"));
        assert!(!filtered_tests.contains("crazy_test"));
        assert!(filtered_tests.contains("super_cool_test"));
    }

    #[test]
    #[cfg_attr(ci, ignore = "ci too slow to reproduce this every time")]
    fn filter_exact_is_faster() {
        let tests: Vec<_> = (0..1000)
            .map(|idx| test! {name: format!("test_{idx}")})
            .collect();

        let not_exact_report = harness(&tests)
            .with_filter(
                DefaultFilter::default()
                    .with_filter(["test_500"])
                    .with_exact(false),
            )
            .run();
        assert_eq!(not_exact_report.outcomes.len(), 1);

        let exact_report = harness(&tests)
            .with_filter(
                DefaultFilter::default()
                    .with_filter(["test_500"])
                    .with_exact(true),
            )
            .run();
        assert_eq!(exact_report.outcomes.len(), 1);

        // we can't do multiply by 1.5
        assert!(exact_report.duration * 3 < not_exact_report.duration * 2);
    }

    #[test]
    fn skipping_tests_works() {
        let tests = &[
            test! {name: "cool_test"},
            test! {name: "boring_test"},
            test! {name: "super_boring_test"},
            test! {name: "crazy_test"},
        ];

        let report = harness(tests)
            .with_filter(DefaultFilter::default().with_skip(["boring"]))
            .run();

        let names: HashSet<_> = report.outcomes.into_iter().map(|(n, _)| n).collect();
        assert!(names.contains("cool_test"));
        assert!(!names.contains("boring_test"));
        assert!(!names.contains("super_boring_test"));
        assert!(names.contains("crazy_test"));
    }

    #[test]
    fn filtering_and_skipping_works() {
        let tests = &[
            test! {name: "cool_test"},
            test! {name: "boring_test"},
            test! {name: "crazy_test"},
            test! {name: "super_cool_test"},
            test! {name: "not_so_cool_test"},
        ];

        let report = harness(tests)
            .with_filter(
                DefaultFilter::default()
                    .with_filter(["cool"])
                    .with_skip(["super"]),
            )
            .run();

        let names: HashSet<_> = report.outcomes.into_iter().map(|(n, _)| n).collect();
        assert!(names.contains("cool_test"));
        assert!(!names.contains("super_cool_test"));
        assert!(!names.contains("boring_test"));
        assert!(!names.contains("crazy_test"));
        assert!(names.contains("not_so_cool_test"));
    }
}
