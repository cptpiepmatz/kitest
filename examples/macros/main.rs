use std::{thread, time::Duration};

use kitest::{
    group::TestGroupBTreeMap, ignore::IgnoreDecision, test::{Test, TestMeta}
};

#[macro_use]
extern crate macros_support;

struct Extra {
    pub experimental: bool,
    pub flaky: bool,
}

#[linkme::distributed_slice]
pub static TESTS: [Test<Extra>];

/// Basic sanity test that should always run.
/// Also checks that bool math does what we think.
#[test]
fn some_test() {
    // absurd but still true
    assert_eq!(2 + 2, 4, "math broke, panic and call physics");
    assert_ne!(String::from("hi"), String::from("bye"), "identity failure");
    assert!(true && !false, "reality is compromised");

    // check that an empty vec is equal to itself
    let v: Vec<u8> = vec![];
    assert_eq!(v, v, "reflexivity should hold");

    // floating point "absurd but true": NaN is not equal to itself
    let nan = f32::NAN;
    assert!(nan != nan, "NaN must not equal itself per IEEE 754");
}

/// Marked flaky. We'll sleep a bit to simulate timing issues or racey code.
#[test]
#[flaky]
fn flaky_test() {
    // pretend nondeterministic timing
    thread::sleep(Duration::from_millis(37));

    // check something timing related but deterministic for us
    let start = std::time::Instant::now();
    thread::sleep(Duration::from_millis(5));
    let elapsed = start.elapsed();
    assert!(
        elapsed >= Duration::from_millis(5),
        "clock went backwards or sleep lied"
    );

    // another absurd truth:
    // any string always starts with "" (empty prefix)
    assert!( "kitest".starts_with(""), "empty prefix should match" );
}

/// Marked experimental. We'll group these later.
#[test]
#[experimental]
fn experimental_test() {
    // show off equality of different but equal slices
    let a: &[u8] = &[1, 2, 3];
    let b = vec![1u8, 2, 3];
    assert_eq!(a, b.as_slice(), "slices with same bytes should match");

    // check Option ergonomics
    let x: Option<i32> = Some(10);
    assert_eq!(x.unwrap(), 10, "unwrap(Some(_)) should work");
}

/// Long-ish sleeper to see how harness behaves with slow tests.
#[test]
fn slow_but_legit_test() {
    // sleep a "random looking" duration that is still fixed
    thread::sleep(Duration::from_millis(113));

    // absurd but true: 0usize is still 0, and we don't use raw usize
    let count: usize = 0;
    assert_eq!(count, 0, "zero should be zero");

    // UTF-8 roundtrip check
    let s = "🦀rust🦀";
    let bytes = s.as_bytes();
    let rebuilt = std::str::from_utf8(bytes).expect("valid utf8 should decode");
    assert_eq!(rebuilt, s, "utf8 should roundtrip");
}

/// This test is "flaky" on purpose (logically weird).
/// We'll assert something about ordering that is true but looks fragile.
#[test]
#[flaky]
fn ordering_flaky_test() {
    // The default sort of these numbers must be ascending
    let mut nums = vec![3, 1, 2, 1];
    nums.sort();
    assert_eq!(nums, vec![1, 1, 2, 3], "sort() should be stable-ish ascending");

    // sleep to mimic race windows
    thread::sleep(Duration::from_millis(9));

    // String::len counts bytes, not chars
    // "é" in UTF-8 is 2 bytes
    assert_eq!("é".len(), 2, "'é' should be 2 bytes in UTF-8");
}

/// Experimental stress-ish test. Sleeps a little, then does multiple asserts,
/// including asserting true == true in a very dramatic way.
#[test]
#[experimental]
fn chaos_experimental_test() {
    thread::sleep(Duration::from_millis(3));

    // true is still true
    assert!(true, "if this fails, the universe ended");

    // bool logic identities
    assert_eq!(true || false, true, "OR truth table broken");
    assert_eq!(true && false, false, "AND truth table broken");

    // Check that pushing then popping from a Vec works
    let mut v = Vec::new();
    v.push("kitest");
    assert_eq!(v.pop(), Some("kitest"), "push/pop should behave like stack");

    // Check that an empty iterator sums to 0
    let empty: [i32; 0] = [];
    let sum: i32 = empty.iter().copied().sum();
    assert_eq!(sum, 0, "sum over empty iterator should be 0");

    // usize to string and back should match
    let n: usize = 12345;
    let ns = n.to_string();
    let parsed: usize = ns.parse().expect("stringified usize should parse back");
    assert_eq!(parsed, n, "roundtrip parse should keep value");
}

/// A test that is so boring it almost insults the runner.
/// This one just asserts that "abc" contains "b".
#[test]
fn string_behavior_test() {
    assert!("abc".contains('b'), "'abc' should contain 'b'");
    assert!("abc".find('z').is_none(), "'abc' should not contain 'z'");
}

/// A test that will only fail if insanely basic math breaks or if
/// the optimizer does black magic that changes integers.
#[test]
fn math_is_still_math_test() {
    let four = 2 + 2;
    assert_eq!(four, 4, "2 + 2 should still be 4");

    // multiplication identity
    let x = 1234;
    assert_eq!(x * 1, x, "x * 1 should be identity");

    // zero annihilates multiplication
    assert_eq!(x * 0, 0, "x * 0 should be 0 always");

    // division by 1
    assert_eq!(x / 1, x, "x / 1 should be identity");
}

/// Check time monotonicity: Instant after sleep should be >= before.
#[test]
fn time_moves_forward_test() {
    let before = std::time::Instant::now();
    thread::sleep(Duration::from_millis(2));
    let after = std::time::Instant::now();
    assert!(
        after >= before,
        "time should move forward, not backwards"
    );
}

/// If this fails, Rust forgot how references work.
/// We never actually hit the panic message unless memory is cursed.
#[test]
fn ref_identity_test() {
    let val = String::from("kitest");
    let ptr_a: *const String = &val;
    let ptr_b: *const String = &val;
    assert_eq!(
        ptr_a, ptr_b,
        "two refs to same local should have same address"
    );
}

/// Make sure bool::then_some works like we think.
#[test]
fn then_some_test() {
    assert_eq!(true.then_some(10), Some(10), "true.then_some should give Some");
    assert_eq!(false.then_some(10), None, "false.then_some should give None");
}

/// If your macro system stores these tests in TESTS with metadata `Extra`,
/// the harness below will:
/// - ignore flaky tests in CI
/// - group experimental tests separately
fn main() {
    // could be set via some flag or env like CI=true
    let in_ci = true;

    kitest::harness(&TESTS)
        .with_ignore(|meta: &TestMeta<Extra>| match (meta.extra.flaky, in_ci) {
            (true, true) => IgnoreDecision::IgnoreWithReason("flaky in CI".into()),
            _ => IgnoreDecision::Run,
        })
        // group tests by the experimental flag so we can e.g. run them last
        .with_grouper(|meta: &TestMeta<Extra>| meta.extra.experimental)
        // use BTreeMap to get actual ordering
        .with_groups(TestGroupBTreeMap::default())
        .run();
}
