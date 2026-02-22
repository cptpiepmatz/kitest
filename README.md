<p align="center">
  <picture>
    <img height="250" src="https://raw.githubusercontent.com/cptpiepmatz/kitest/main/logo/logo.svg">
  </picture>
</p>
<h1 align="center">kitest</h1>
<p align="center">
  <i>pronounced "kai-test"</i><br>
  <b>🪁 A composable test harness toolkit with room to fly.</b>
</p>

<br>

<div align="center">

[![License](https://img.shields.io/github/license/cptpiepmatz/kitest?style=for-the-badge)](./LICENSE)
[![Version](https://img.shields.io/crates/v/kitest?style=for-the-badge)](https://crates.io/crates/kitest)
![MSRV](https://img.shields.io/crates/msrv/kitest?style=for-the-badge)
[![Docs](https://img.shields.io/docsrs/kitest?style=for-the-badge)](https://docs.rs/kitest)

</div>

## About

Kitest provides building blocks for custom test harnesses on top of `cargo test`.

It ships with a defaults that behave similar to Rust's built in harness, but 
every part is replaceable. 
Filtering, ignoring, panic handling, execution strategy, and formatting can all 
be swapped independently.

Kitest is not a new testing style. 
It is a foundation for creating one.

## What kitest provides

- A default harness comparable to the built in one
- Data driven tests
- Test grouping with shared setup and teardown
- Suite level setup and teardown
- Pluggable output formatting
- Full control over filtering and ignore behavior

Example output with the default formatter:

<picture>
  <img src="https://raw.githubusercontent.com/cptpiepmatz/kitest/refs/heads/main/media/default-example.png">
</picture>

## Getting started

### Add the dependency

Kitest is typically added as a dev dependency:

```toml
[dev-dependencies]
kitest = "0.3.0"
```

### Disable the default harness

For integration tests:

```toml
[[test]]
name = "tests"
path = "tests/main.rs"
harness = false
```

To replace the unit test harness as well:

```toml
[lib]
harness = false
```

When disabling the lib harness provide a:

```rust
#[cfg(test)] 
fn main()
```

This function will be executed when running the test harness.

### Minimal example

```rust
use std::{borrow::Cow, process::Termination};
use kitest::prelude::*;

fn ok() {}
fn ignored() {}

const TESTS: &[Test] = &[
    Test::new(
        TestFnHandle::from_static_obj(&|| ok()),
        TestMeta {
            name: Cow::Borrowed("ok"),
            ignore: IgnoreStatus::Run,
            should_panic: PanicExpectation::ShouldNotPanic,
            origin: origin!(),
            extra: (),
        },
    ),
    Test::new(
        TestFnHandle::from_static_obj(&|| ignored()),
        TestMeta {
            name: Cow::Borrowed("ignored"),
            ignore: IgnoreStatus::IgnoreWithReason(Cow::Borrowed("not needed here")),
            should_panic: PanicExpectation::ShouldNotPanic,
            origin: origin!(),
            extra: (),
        },
    ),
];

fn main() -> impl Termination {
    kitest::harness(TESTS)
        .run()
        .report()
}
```

## Customizing the harness

`kitest::harness` returns a `TestHarness` with default strategies. 
Each component can be replaced.

```rust
use kitest::{
    filter::DefaultFilter,
    formatter::terse::TerseFormatter,
    ignore::DefaultIgnore,
    prelude::*,
};

fn main() -> impl std::process::Termination {
    let tests: &[Test] = &[];

    kitest::harness(tests)
        .with_filter(DefaultFilter::default().with_exact(true))
        .with_ignore(DefaultIgnore::IncludeIgnored)
        .with_formatter(TerseFormatter::default())
        .run()
        .report()
}
```

## Grouping tests

By default, tests are just a flat list.
Grouping allows structuring them into logical sets that share context.

This is useful when:

* Multiple tests need the same expensive setup
* A resource must be initialized once per group
* Cleanup should happen once after a batch of related tests
* Tests should be reported per logical unit instead of globally

Without grouping, setup and teardown typically happen per test.
With grouping, kitest executes tests per group, which makes shared setup and 
teardown straightforward.

Grouping is optional.
Calling `with_grouper` promotes the harness into a grouped harness.
Tests are then executed per group.

```rust
use kitest::{group::TestGroupBTreeMap, prelude::*};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd)]
enum Flag { A, B }

fn main() -> impl std::process::Termination {
    let tests: &[Test<Flag>] = &[];

    kitest::harness(tests)
        .with_grouper(|meta| meta.extra)
        .with_groups(TestGroupBTreeMap::new())
        .run()
        .report()
}
```

In this example, the `extra` metadata field determines the group.
All tests with the same `Flag` value run together.

Example grouped output:

<picture>
  <img src="https://raw.githubusercontent.com/cptpiepmatz/kitest/refs/heads/main/media/group-by-flag-example.png">
</picture>

## Output capture

Kitest can capture output written through its capture aware macros such as
`kitest::println!` and `kitest::eprintln!`.

This is a best effort approach.

On stable Rust there is no reliable way to globally intercept stdout and stderr.
Only output written through kitest's capture aware macros is guaranteed to be 
captured.

To make this easier for unit tests, kitest can override the standard print 
macros during test builds:

```rust
#[cfg(test)]
#[macro_use]
extern crate kitest;
```

When used in a crate root, this automatically overrides `println!`, `eprintln!`, 
and related macros during unit testing.
This does not apply to integration tests.

Even with this override, output capture is not perfect.
In general, printing during tests should be avoided as a best practice.
The capture system simply tries its best to make output visible and structured 
when printing happens anyway.

## Examples

This repository contains several examples:

* `default`
* `terse`
* `group_by_flag`
* `basic`
* `macros`

Run them with:

```bash
cargo run --example default
cargo run --example group_by_flag
```
