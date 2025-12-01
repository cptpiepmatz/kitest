use crate::lib::snapshot;

// snapshot!(all_ignored: [first: {ignore: true}, second: {ignore: true}, third: {ignore: "reasons"}]);
// snapshot!(all_ok: [ok_1, ok_2, ok_3, ok_4]);
// snapshot!(single_error: [fail]);
// snapshot!(single_ok: [one_test]);
// snapshot!(single_ignored: [one_test: {ignore: true}]);
snapshot!(single_panic: [panic]);
