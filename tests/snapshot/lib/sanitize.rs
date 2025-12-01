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

pub fn sanitize_panic_output<'s>(input: &'s str) -> String {
    // 1. Normalize thread name + id
    let tmp = THREAD_RE.replace_all(input, "thread '<thread>'");

    // 2. Normalize paths to use forward slashes
    let tmp = PATH_RE.replace_all(tmp.as_ref(), |caps: &regex::Captures| {
        let mut path = caps["path"].to_string();
        path = path.replace('\\', "/");
        format!("{}:{}:{}", path, &caps["line"], &caps["col"])
    });

    tmp.to_string()
}
