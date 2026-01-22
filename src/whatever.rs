use std::{
    any::Any,
    fmt::{self, Debug, Display, Formatter},
    panic::RefUnwindSafe,
};

/// Type erased user data with a fixed set of trait bounds.
///
/// [`Whatever`] is a small type erasure container. It stores a value of any `'static` type, as
/// long as it implements:
///
/// - [`Debug`]
/// - [`Display`]
/// - [`Clone`]
/// - [`Eq`]
/// - [`RefUnwindSafe`]
/// - [`Send`] and [`Sync`]
///
/// The main purpose of `Whatever` is to allow attaching "nice to have" user data in places where
/// it is useful for reporting or formatting, but not required for executing tests. In those cases,
/// the runtime indirection is acceptable.
///
/// Why this exists:
///
/// Kitest already carries `Extra` through most types so strategies can use metadata without
/// downcasting. Adding more generic parameters everywhere for occasional optional data would make
/// signatures even heavier and would spread type parameters through the whole crate.
///
/// `Whatever` avoids that by erasing the type when the value is not essential for the core test
/// execution path.
///
/// Note: this is intentionally not something the built-in Rust test harness needs. It is a
/// Kitest specific tradeoff to keep the generic surface area under control.
pub struct Whatever(Box<dyn WhateverImpl>);

trait WhateverImpl: Debug + Display + Any + RefUnwindSafe + Send + Sync {
    fn clone_whatever(&self) -> Whatever;
    fn eq_whatever(&self, other: &Whatever) -> bool;
}

impl<T> WhateverImpl for T
where
    T: Debug + Display + Clone + Eq + RefUnwindSafe + Send + Sync + 'static,
{
    fn clone_whatever(&self) -> Whatever {
        Whatever(Box::new(self.clone()))
    }

    fn eq_whatever(&self, other: &Whatever) -> bool {
        let Some(other) = other.as_any_ref().downcast_ref() else {
            return false;
        };
        self.eq(other)
    }
}

impl Clone for Whatever {
    fn clone(&self) -> Self {
        self.0.clone_whatever()
    }
}

impl Debug for Whatever {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Debug::fmt(&self.0, f)
    }
}

impl Display for Whatever {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Display::fmt(&self.0, f)
    }
}

impl PartialEq for Whatever {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq_whatever(other)
    }
}

impl Eq for Whatever {}

impl Whatever {
    /// Create a new [`Whatever`] from a concrete value.
    ///
    /// The value must implement the full set of traits required by `Whatever`:
    /// [`Debug`], [`Display`], [`Clone`], [`Eq`], [`RefUnwindSafe`], [`Send`], and [`Sync`].
    ///
    /// This constructor performs type erasure.
    /// The original type information can later be recovered using the `Any` based accessors if
    /// both producer and consumer agree on the concrete type.
    pub fn from<T: Debug + Display + Clone + Eq + RefUnwindSafe + Send + Sync + 'static>(
        value: T,
    ) -> Whatever {
        Self(Box::new(value))
    }

    /// Convert this [`Whatever`] into a boxed [`Any`].
    ///
    /// This consumes the `Whatever` and returns the underlying value as a trait object.
    /// It can later be downcast to the original concrete type.
    ///
    /// This is useful when ownership of the erased value is needed.
    pub fn into_any(self) -> Box<dyn Any + RefUnwindSafe + Send + Sync> {
        self.0
    }

    /// Get a shared reference to the underlying value as [`Any`].
    ///
    /// This allows inspecting or downcasting the stored value without taking ownership.
    pub fn as_any_ref(&self) -> &dyn Any {
        &self.0
    }

    /// Get a mutable reference to the underlying value as [`Any`].
    ///
    /// This allows mutating or downcasting the stored value without taking ownership.
    pub fn as_any_mut(&mut self) -> &mut dyn Any {
        &mut self.0
    }
}
