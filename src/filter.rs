use crate::meta::TestMeta;

pub trait TestFilter<Extra> {
    fn filter(&self, meta: &TestMeta<Extra>) -> bool;

    fn skip_filtering(&self) -> bool {
        false
    }
}

pub struct NoFilter;

impl<Extra> TestFilter<Extra> for NoFilter {
    fn filter(&self, _: &TestMeta<Extra>) -> bool {
        true
    }

    fn skip_filtering(&self) -> bool {
        true
    }
}

pub struct DefaultFilter {
    pub exact: bool,
    pub filter: Vec<String>,
    pub skip: Vec<String>,
}

impl DefaultFilter {
    pub fn new() -> Self {
        Self {
            exact: false,
            filter: Vec::new(),
            skip: Vec::new(),
        }
    }

    pub fn with_exact(self, exact: bool) -> Self {
        Self {
            exact,
            ..self
        }
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

impl Default for DefaultFilter {
    fn default() -> Self {
        Self::new()
    }
}

impl<Extra> TestFilter<Extra> for DefaultFilter {
    fn filter(&self, meta: &TestMeta<Extra>) -> bool {
        let name = meta.name.as_ref();

        if self.exact {
            if self.skip.iter().any(|s| name == s) {
                return false;
            }

            return self.filter.is_empty() || self.filter.iter().any(|f| name == f);
        }

        if self.skip.iter().any(|s| name.contains(s)) {
            return false;
        }

        return self.filter.is_empty() || self.filter.iter().any(|f| name.contains(f));
    }

    fn skip_filtering(&self) -> bool {
        self.filter.is_empty() && self.skip.is_empty()
    }
}

impl<Extra, F> TestFilter<Extra> for F
where
    F: Fn(&TestMeta<Extra>) -> bool,
{
    fn filter(&self, meta: &TestMeta<Extra>) -> bool {
        self(meta)
    }
}
