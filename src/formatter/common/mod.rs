//! Common helpers for formatter implementations.
//!
//! This module contains small helper types that are convenient when implementing kitest formatters.
//! They are intentionally formatter focused and are not meant to be general purpose building blocks
//! for unrelated code.

use crate::formatter::FmtListTest;

pub mod color;
pub mod label;

/// A small newtype around a test name.
///
/// This is mainly used to make formatter implementations nicer to read, since it
/// can be constructed directly from [`FmtListTest`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct TestName<'t>(pub &'t str);

impl<'t, Extra> From<FmtListTest<'t, Extra>> for TestName<'t> {
    fn from(value: FmtListTest<'t, Extra>) -> Self {
        Self(value.meta.name.as_ref())
    }
}
