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

            let tests = [$(
                $crate::lib::test::test! {
                    name: stringify!($test_name),
                    func: || $mod_name::$test_name(),
                    $($($field_name: $field_value,)*)?
                },
            )*];

            let harness = kitest::harness(&tests)
                .with_runner(kitest::runner::SimpleRunner::default())
                .with_formatter(kitest::formatter::no::NoFormatter);
            let pretty_formatter = || kitest::formatter::pretty::PrettyFormatter::default();

            // on Windows does the built-in test harness call color instructions to the terminal 
            // and not ansi color codes
            #[cfg(not(target_os = "windows"))]
            let _pretty_test_color = {
                let expected = crate::run_rust_doc_test(
                    &file_name,
                    ["--format=pretty", "--color=always"]
                ).unwrap();

                let actual = crate::Buffer::default();
                kitest::capture::reset_first_panic();
                let formatter = pretty_formatter()
                    .with_target(actual.clone())
                    .with_color_setting(true);
                let report = harness.clone().with_formatter(formatter).run();

                let actual = actual.try_to_string().unwrap();
                assert_eq!(expected.exit_code, report.exit_code());
                assert_eq!(
                    $crate::lib::sanitize_panic_output(&expected.stdout),
                    $crate::lib::sanitize_panic_output(&actual)
                );
            };

            let _pretty_test_no_color = {
                let expected = crate::run_rust_doc_test(
                    &file_name,
                    ["--format=pretty", "--color=never"]
                ).unwrap();

                let actual = crate::Buffer::default();
                kitest::capture::reset_first_panic();
                let formatter = pretty_formatter()
                    .with_target(actual.clone())
                    .with_color_setting(false);
                let report = harness.clone().with_formatter(formatter).run();

                let actual = actual.try_to_string().unwrap();
                assert_eq!(expected.exit_code, report.exit_code());
                assert_eq!(
                    $crate::lib::sanitize_panic_output(&expected.stdout),
                    $crate::lib::sanitize_panic_output(&actual)
                );
            };

            let _pretty_list = {
                let expected = crate::run_rust_doc_test(
                    &file_name,
                    ["--format=pretty", "--list"]
                ).unwrap();

                let actual = crate::Buffer::default();
                kitest::capture::reset_first_panic();
                let formatter = pretty_formatter().with_target(actual.clone());
                harness.clone().with_formatter(formatter).list();

                let actual = actual.try_to_string().unwrap();
                assert_eq!(
                    $crate::lib::sanitize_list_output(&expected.stdout),
                    $crate::lib::sanitize_list_output(&actual)
                );
            };

            // TODO: also test listing and terse formatter
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
