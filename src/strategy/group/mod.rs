//! Grouping support for kitest.
//!
//! This module contains everything that is relevant for grouped test harnesses.
//! Grouping lets us split tests into logical groups (for example by module or
//! tag) and then run and report them as groups instead of a flat list.
//!
//! Grouping is built around three traits:
//! - [`TestGrouper`] decides which group a test belongs to
//! - [`TestGroups`] stores the grouped tests
//! - [`TestGroupRunner`] controls how groups are executed
//!
//! Together these traits define how tests are assigned to groups, how those
//! groups are held, and how they are run.

mod grouper;
pub use grouper::*;

mod groups;
pub use groups::*;

mod group_runner;
pub use group_runner::*;
