use std::path::Path;

pub mod test;

mod sanitize;

use kitest::{formatter::common::color::ColorSetting, test::TestOrigin};
pub use sanitize::*;

#[cfg(not(target_os = "windows"))]
pub const COLOR_SETTING: ColorSetting = ColorSetting::Always;

// on Windows does the built-in test harness could color instructions to the terminal and not ansi 
// colors
#[cfg(target_os = "windows")]
pub const COLOR_SETTING: ColorSetting = ColorSetting::Never;

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
                        .with_color_setting($crate::lib::COLOR_SETTING)
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
