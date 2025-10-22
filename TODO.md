# Planned `rustdoctest` flags to implement

- [x] `--include-ignored`: Run ignored and not ignored tests
- [x] `--ignored`: Run only ignored tests
- [ ] `--force-run-in-process`: Forces tests to run in-process when panic=abort
- [ ] `--exclude-should-panic`: Excludes tests marked as should_panic
- [ ] `--list`: List all tests and benchmarks
- [ ] `--help`: Display this message
- [ ] `--logfile PATH`: Write logs to the specified file (deprecated)
- [ ] `--no-capture`: Don't capture stdout/stderr of each task, allow printing directly
- [x] `--test-threads n_threads`: Number of threads used for running tests in parallel
- [x] `--skip FILTER`: Skip tests whose names contain FILTER (can be used multiple times)
- [ ] `--quiet` / `-q`: Display one character per test instead of one line (alias to `--format=terse`)
- [x] `--exact`: Exactly match filters rather than by substring
- [ ] `--color auto|always|never`: Configure coloring of output
- [ ] `--format pretty|terse|json|junit`: Configure formatting of output
  - [ ] `--format pretty`
  - [ ] `--format terse`
  - [ ] `--format json`
  - [ ] `--format junit`
- [ ] `--show-output`: Show captured stdout of successful tests
- [ ] `--report-time`: Show execution time of each test (supports thresholds via env vars)
- [ ] `--ensure-time`: Treat excess test execution time as an error (uses same env vars as report-time)

# Unplanned `rustdoctest` flags
> [!NOTE]
> If not otherwise required, these will not be implemented in some way.
- `--test`: Run tests and not benchmarks
- `--bench`: Run benchmarks instead of tests
- `-Z unstable-options`: Enable nightly-only flags
