pub mod test;

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
            let report = kitest::harness(&tests)
                .with_formatter(PrettyFormatter::default().with_target(actual.clone()))
                .run();

            let actual = actual.try_to_string().unwrap();
            assert_eq!(expected.exit_code, report.exit_code());
            assert_eq!(expected.stdout, actual);
        }
    }
}

pub(crate) use snapshot;
