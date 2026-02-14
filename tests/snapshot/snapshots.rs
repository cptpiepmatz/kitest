use crate::lib::snapshot;

snapshot!(all_ignored: [first: {ignore: true}, second: {ignore: true}, third: {ignore: "reasons"}]);
snapshot!(all_ok: [ok_1, ok_2, ok_3, ok_4]);
snapshot!(all_panic: [a, b, c, d]);
snapshot!(expected_panic: [
    no_panic_when_expected: {
        should_panic: true,
        origin: snapshot_file("expected_panic.rs", 3)
    },
    no_panic_with_expected_message: {
        should_panic: "did panic",
        origin: snapshot_file("expected_panic.rs", 12)
    },
    panic_any: {
        should_panic: true,
        origin: snapshot_file("expected_panic.rs", 21)
    },
    panic_from_assert: {
        should_panic: "did panic",
        origin: snapshot_file("expected_panic.rs", 30)
    },
    panic_with_matching_message: {
        should_panic: "did panic",
        origin: snapshot_file("expected_panic.rs", 38)
    },
    panic_with_partial_message_match: {
        should_panic: "panic",
        origin: snapshot_file("expected_panic.rs", 46)
    },
    panic_with_wrong_message: {
        should_panic: "other",
        origin: snapshot_file("expected_panic.rs", 54)
    },
]);
snapshot!(panic_in_the_middle: [a_ok, b_panic, c_ok, d_panic, e_panic, f_ok]);
snapshot!(single_error: [fail]);
snapshot!(single_ok: [one_test]);
snapshot!(single_ignored: [one_test: {ignore: true}]);
snapshot!(single_panic: [panic]);
