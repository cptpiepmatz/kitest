use kitest::test::TestOrigin;

use crate::lib::snapshot;

snapshot!(all_ignored: [first: {ignore: true}, second: {ignore: true}, third: {ignore: "reasons"}]);
snapshot!(all_ok: [ok_1, ok_2, ok_3, ok_4]);
snapshot!(all_panic: [a, b, c, d]);
snapshot!(expected_panic: [no_panic: {should_panic: true, origin: TestOrigin::TextFile { file: "tests/snapshot/snapshots/expected_panic.rs".into(), line: 11, column: 8 }}, panic: {should_panic: true}]);
snapshot!(single_error: [fail]);
snapshot!(single_ok: [one_test]);
snapshot!(single_ignored: [one_test: {ignore: true}]);
snapshot!(single_panic: [panic]);
