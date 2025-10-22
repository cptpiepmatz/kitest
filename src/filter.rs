use crate::meta::TestMeta;

pub enum FilterDecision {
    Keep,
    Exclude,
    KeepAndDone,
    ExcludeAndDone,
}

pub trait TestFilter<Extra> {
    fn filter(&mut self, meta: &TestMeta<Extra>) -> FilterDecision;

    fn skip_filtering(&self) -> bool {
        false
    }
}

pub struct NoFilter;

impl<Extra> TestFilter<Extra> for NoFilter {
    fn filter(&mut self, _: &TestMeta<Extra>) -> FilterDecision {
        FilterDecision::Keep
    }

    fn skip_filtering(&self) -> bool {
        true
    }
}

#[derive(Default)]
pub struct DefaultFilter {
    exact: bool,
    filter: Vec<String>,
    skip: Vec<String>,
}

impl DefaultFilter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_exact(self, exact: bool) -> Self {
        Self { exact, ..self }
    }

    pub fn extend_filter(mut self, filter: impl IntoIterator<Item = String>) -> Self {
        self.filter.extend(filter);
        self
    }

    pub fn extend_skip(mut self, skip: impl IntoIterator<Item = String>) -> Self {
        self.skip.extend(skip);
        self
    }
}

impl<Extra> TestFilter<Extra> for DefaultFilter {
    fn filter(&mut self, meta: &TestMeta<Extra>) -> FilterDecision {
        let name = meta.name.as_ref();

        if self.exact {
            if self.skip.iter().any(|s| name == s) {
                return FilterDecision::Exclude;
            }

            match self.filter.is_empty() || self.filter.iter().any(|f| name == f) {
                true => return FilterDecision::Keep,
                false => return FilterDecision::Exclude,
            }
        }

        if self.skip.iter().any(|s| name.contains(s)) {
            return FilterDecision::Exclude;
        }

        match self.filter.is_empty() || self.filter.iter().any(|f| name.contains(f)) {
            true => FilterDecision::Keep,
            false => FilterDecision::Exclude,
        }
    }

    fn skip_filtering(&self) -> bool {
        self.filter.is_empty() && self.skip.is_empty()
    }
}

impl<Extra, F> TestFilter<Extra> for F
where
    F: Fn(&TestMeta<Extra>) -> FilterDecision,
{
    fn filter(&mut self, meta: &TestMeta<Extra>) -> FilterDecision {
        self(meta)
    }
}
