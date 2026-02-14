use std::{
    path::Path,
    sync::{LazyLock, Mutex},
};

pub mod test;

mod sanitize;

use kitest::test::TestOrigin;
pub use sanitize::*;

// pub use std::assert_eq as assert_str_eq;
macro_rules! snapshot {
    ($mod_name:ident: [
        $($test_name:ident $(: {
            $($field_name:ident: $field_value: expr),* $(,)?
        })?),* $(,)?
    ]) => {
        mod $mod_name {
            use $crate::lib::*;
            use std::{sync::LazyLock, ops::Deref};
            use kitest::{
                prelude::*,
                runner::SimpleRunner,
                formatter::{pretty::PrettyFormatter, terse::TerseFormatter}
            };

            mod test_functions {
                include!(concat!("snapshots/", stringify!($mod_name), ".rs"));
            }

            static BUILD_CARGO_TEST: LazyLock<String> = LazyLock::new(|| {
                crate::build_cargo_test(stringify!($mod_name)).unwrap()
            });

            const TEST_N: usize = [$(stringify!($test_name)),*].len();
            static TESTS: LazyLock<[Test; TEST_N]> = LazyLock::new(|| [$(
                test::test! {
                    name: stringify!($test_name),
                    func: || test_functions::$test_name(),
                    $($($field_name: $field_value,)*)?
                },
            )*]);

            mod pretty {
                use super::*;

                // on Windows does the built-in test harness call color instructions to the terminal
                // and not ansi color codes
                #[cfg(not(target_os = "windows"))]
                #[test]
                fn color() {
                    let expected = crate::run_rust_doc_test(
                        BUILD_CARGO_TEST.deref(),
                        ["--format=pretty", "--color=always"]
                    ).unwrap();

                    let _snapshot_lock_guard = SNAPSHOT_LOCK.lock();

                    let actual = crate::Buffer::default();
                    kitest::capture::reset_first_panic();
                    let formatter = PrettyFormatter::default()
                        .with_target(actual.clone())
                        .with_color_setting(true);
                    let report = kitest::harness(TESTS.deref())
                        .with_runner(SimpleRunner::default())
                        .with_formatter(formatter)
                        .run();

                    let actual = actual.try_to_string().unwrap();
                    assert_eq!(expected.exit_code, report.exit_code());
                    assert_str_eq!(
                        $crate::lib::sanitize_panic_output(&expected.stdout),
                        $crate::lib::sanitize_panic_output(&actual)
                    );
                }

                #[test]
                fn no_color() {
                    let expected = crate::run_rust_doc_test(
                        BUILD_CARGO_TEST.deref(),
                        ["--format=pretty", "--color=never"]
                    ).unwrap();

                    let _snapshot_lock_guard = SNAPSHOT_LOCK.lock();

                    let actual = crate::Buffer::default();
                    kitest::capture::reset_first_panic();
                    let formatter = PrettyFormatter::default()
                        .with_target(actual.clone())
                        .with_color_setting(false);
                    let report = kitest::harness(TESTS.deref())
                        .with_runner(SimpleRunner::default())
                        .with_formatter(formatter)
                        .run();

                    let actual = actual.try_to_string().unwrap();
                    assert_eq!(expected.exit_code, report.exit_code());
                    assert_str_eq!(
                        $crate::lib::sanitize_panic_output(&expected.stdout),
                        $crate::lib::sanitize_panic_output(&actual)
                    );
                }

                #[test]
                fn list() {
                    let expected = crate::run_rust_doc_test(
                        BUILD_CARGO_TEST.deref(),
                        ["--format=pretty", "--list"]
                    ).unwrap();

                    let _snapshot_lock_guard = SNAPSHOT_LOCK.lock();

                    let actual = crate::Buffer::default();
                    kitest::capture::reset_first_panic();
                    let formatter = PrettyFormatter::default().with_target(actual.clone());
                    kitest::harness(TESTS.deref())
                        .with_runner(SimpleRunner::default())
                        .with_formatter(formatter)
                        .list();

                    let actual = actual.try_to_string().unwrap();
                    assert_str_eq!(
                        $crate::lib::sanitize_list_output(&expected.stdout),
                        $crate::lib::sanitize_list_output(&actual)
                    );
                }
            }

            mod terse {
                use super::*;

                // on Windows does the built-in test harness call color instructions to the terminal
                // and not ansi color codes
                #[cfg(not(target_os = "windows"))]
                #[test]
                fn color() {
                    let expected = crate::run_rust_doc_test(
                        BUILD_CARGO_TEST.deref(),
                        ["--format=terse", "--color=always"]
                    ).unwrap();

                    let _snapshot_lock_guard = SNAPSHOT_LOCK.lock();

                    let actual = crate::Buffer::default();
                    kitest::capture::reset_first_panic();
                    let formatter = TerseFormatter::default()
                        .with_target(actual.clone())
                        .with_color_setting(true);
                    let report = kitest::harness(TESTS.deref())
                        .with_runner(SimpleRunner::default())
                        .with_formatter(formatter)
                        .run();

                    let actual = actual.try_to_string().unwrap();
                    assert_eq!(expected.exit_code, report.exit_code());
                    assert_str_eq!(
                        $crate::lib::sanitize_panic_output(&expected.stdout),
                        $crate::lib::sanitize_panic_output(&actual)
                    );
                }

                #[test]
                fn no_color() {
                    let expected = crate::run_rust_doc_test(
                        BUILD_CARGO_TEST.deref(),
                        ["--format=terse", "--color=never"]
                    ).unwrap();

                    let _snapshot_lock_guard = SNAPSHOT_LOCK.lock();

                    let actual = crate::Buffer::default();
                    kitest::capture::reset_first_panic();
                    let formatter = TerseFormatter::default()
                        .with_target(actual.clone())
                        .with_color_setting(false);
                    let report = kitest::harness(TESTS.deref())
                        .with_runner(SimpleRunner::default())
                        .with_formatter(formatter)
                        .run();

                    let actual = actual.try_to_string().unwrap();
                    assert_eq!(expected.exit_code, report.exit_code());
                    assert_str_eq!(
                        $crate::lib::sanitize_panic_output(&expected.stdout),
                        $crate::lib::sanitize_panic_output(&actual)
                    );
                }

                #[test]
                fn list() {
                    let expected = crate::run_rust_doc_test(
                        BUILD_CARGO_TEST.deref(),
                        ["--format=terse", "--list"]
                    ).unwrap();

                    let _snapshot_lock_guard = SNAPSHOT_LOCK.lock();

                    let actual = crate::Buffer::default();
                    kitest::capture::reset_first_panic();
                    let formatter = TerseFormatter::default().with_target(actual.clone());
                    kitest::harness(TESTS.deref())
                        .with_runner(SimpleRunner::default())
                        .with_formatter(formatter)
                        .list();

                    let actual = actual.try_to_string().unwrap();
                    assert_str_eq!(
                        $crate::lib::sanitize_list_output(&expected.stdout),
                        $crate::lib::sanitize_list_output(&actual)
                    );
                }
            }
        }
    }
}

pub(crate) use snapshot;

pub static SNAPSHOT_LOCK: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

pub fn snapshot_file(file: impl AsRef<Path>, line: u32) -> TestOrigin {
    TestOrigin::TextFile {
        file: format!("tests/snapshot/snapshots/{}", file.as_ref().display()).into(),
        line,
        column: 8,
    }
}
