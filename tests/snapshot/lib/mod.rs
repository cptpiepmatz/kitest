use std::path::Path;

pub mod test;

mod sanitize;

use kitest::test::TestOrigin;
pub use sanitize::*;

macro_rules! snapshot {
    ($mod_name:ident: [
        $($test_name:ident $(: {
            $($field_name:ident: $field_value: expr),* $(,)?
        })?),* $(,)?
    ]) => {
        mod $mod_name;

        #[test]
        fn $mod_name() {
            let file_name = crate::build_cargo_test(stringify!($mod_name)).unwrap();
            let expected = crate::run_rust_doc_test(file_name).unwrap();

            let tests = [$(
                $crate::lib::test::test! {
                    name: stringify!($test_name),
                    func: || $mod_name::$test_name(),
                    $($($field_name: $field_value,)*)?
                },
            )*];

            let actual = crate::Buffer::default();
            kitest::capture::reset_first_panic();
            let report = kitest::harness(&tests)
                .with_formatter(
                    kitest::formatter::pretty::PrettyFormatter::default()
                        .with_target(actual.clone())
                    )
                .with_runner(kitest::runner::SimpleRunner::default())
                .run();

            let actual = actual.try_to_string().unwrap();
            assert_eq!(expected.exit_code, report.exit_code());
            assert_eq!(
                $crate::lib::sanitize_panic_output(&expected.stdout),
                $crate::lib::sanitize_panic_output(&actual)
            );
        }
    }
}

pub(crate) use snapshot;

pub fn snapshot_file(file: impl AsRef<Path>, line: u32) -> TestOrigin {
    TestOrigin::TextFile {
        file: format!("tests/snapshot/snapshots/{}", file.as_ref().display()).into(),
        line,
        column: 8,
    }
}
