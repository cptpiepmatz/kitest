use std::sync::LazyLock;

use regex::Regex;

static THREAD_RE: LazyLock<Regex> = LazyLock::new(|| {
    // Matches: thread 'something' (1234) or just: thread 'something'
    Regex::new(r"thread '([^']+)'(?: \(\d+\))?").unwrap()
});

static PATH_RE: LazyLock<Regex> = LazyLock::new(|| {
    // Example matches:
    //   tests\snapshot\snapshots\single_panic.rs:4:9
    //   tests/snapshot/snapshots/single_panic.rs:4:9
    Regex::new(r"(?P<path>tests[^\n:]+\.rs):(?P<line>\d+):(?P<col>\d+)").unwrap()
});

static TEST_RESULT_RE: LazyLock<Regex> = LazyLock::new(|| {
    // Matches: finished in 0.00s or finished in 12.34s etc.
    Regex::new(r"finished in [0-9]+\.[0-9]+s").unwrap()
});

pub fn sanitize_panic_output(input: &str) -> String {
    // 1. Normalize thread name + id
    let tmp = THREAD_RE.replace_all(input, "thread '<thread>'");

    // 2. Normalize paths to use forward slashes
    let tmp = PATH_RE.replace_all(tmp.as_ref(), |caps: &regex::Captures| {
        let mut path = caps["path"].to_string();
        path = path.replace('\\', "/");
        format!("{}:{}:{}", path, &caps["line"], &caps["col"])
    });

    // 3. Normalize execution time
    let tmp = TEST_RESULT_RE.replace_all(&tmp, "finished in <time>");

    tmp.to_string()
}

static NO_BENCHMARKS_RE: LazyLock<Regex> = LazyLock::new(|| {
    // Matches: 8 tests, 0 benchmarks etc.
    Regex::new(r"(?P<tests>[0-9]+ tests?), (?P<benchmarks>[0-9]+ benchmarks?)").unwrap()
});

pub fn sanitize_list_output(input: &str) -> String {
    NO_BENCHMARKS_RE
        .replace_all(input, |caps: &regex::Captures| caps["tests"].to_string())
        .to_string()
}
