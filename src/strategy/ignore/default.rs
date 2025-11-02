use crate::{
    ignore::{IgnoreStatus, TestIgnore},
    test::TestMeta,
};

#[derive(Debug, Default)]
pub enum DefaultIgnore {
    IncludeIgnored,
    IgnoredOnly,
    #[default]
    Default,
}

impl<Extra> TestIgnore<Extra> for DefaultIgnore {
    fn ignore(&self, meta: &TestMeta<Extra>) -> IgnoreStatus {
        match (self, &meta.ignore) {
            (DefaultIgnore::IgnoredOnly, IgnoreStatus::Run) => IgnoreStatus::Ignore,
            (DefaultIgnore::IncludeIgnored, _)
            | (DefaultIgnore::IgnoredOnly, IgnoreStatus::Ignore)
            | (DefaultIgnore::IgnoredOnly, IgnoreStatus::IgnoreWithReason(_))
            | (DefaultIgnore::Default, IgnoreStatus::Run) => IgnoreStatus::Run,
            (DefaultIgnore::Default, status) => status.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{borrow::Cow, ops::Deref, sync::LazyLock};

    use super::*;
    use crate::{outcome::TestStatus, test::Test, test_support::*};

    // This can be done without a LazyLock at const time but requires more characters.
    static TESTS: LazyLock<[Test; 3]> = LazyLock::new(|| {
        [
            test! {name: "ok", ignore: false},
            test! {name: "ignored", ignore: true},
            test! {name: "ignored_with_reason", ignore: "with reason"},
        ]
    });

    #[test]
    fn ignoring_works() {
        let report = harness(TESTS.deref())
            .with_ignore(DefaultIgnore::default())
            .run();

        assert_eq!(report.outcomes.len(), 3);
        assert!(report.outcomes[0].1.status.passed());
        assert!(report.outcomes[1].1.status.ignored());
        assert!(report.outcomes[2].1.status.ignored());

        assert!(matches!(
            report.outcomes[1].1.status,
            TestStatus::Ignored { reason: None }
        ));
        assert!(matches!(
            report.outcomes[2].1.status,
            TestStatus::Ignored {
                reason: Some(Cow::Borrowed("with reason"))
            }
        ));
    }

    #[test]
    fn include_ignored_works() {
        let report = harness(TESTS.deref())
            .with_ignore(DefaultIgnore::IncludeIgnored)
            .run();

        assert_eq!(report.outcomes.len(), 3);
        assert!(report.outcomes[0].1.status.passed());
        assert!(report.outcomes[1].1.status.passed());
        assert!(report.outcomes[2].1.status.passed());
    }

    #[test]
    fn ignored_only_works() {
        let report = harness(TESTS.deref())
            .with_ignore(DefaultIgnore::IgnoredOnly)
            .run();

        assert_eq!(report.outcomes.len(), 3);
        assert!(report.outcomes[0].1.status.ignored());
        assert!(report.outcomes[1].1.status.passed());
        assert!(report.outcomes[2].1.status.passed());
    }
}
