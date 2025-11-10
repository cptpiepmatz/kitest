use std::{
    any::Any,
    fmt::{self, Debug, Display, Formatter},
};

pub struct Whatever(Box<dyn WhateverImpl>);

trait WhateverImpl: Debug + Display + Any + Send + Sync {
    fn clone_whatever(&self) -> Whatever;
    fn eq_whatever(&self, other: &Whatever) -> bool;
}

impl<T> WhateverImpl for T
where
    T: Debug + Display + Clone + Eq + Send + Sync + 'static,
{
    fn clone_whatever(&self) -> Whatever {
        Whatever(Box::new(self.clone()))
    }

    fn eq_whatever(&self, other: &Whatever) -> bool {
        let Some(other) = other.downcast_ref() else {
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
    pub fn from<T: Debug + Display + Clone + Eq + Send + Sync + 'static>(value: T) -> Whatever {
        Self(Box::new(value))
    }

    pub fn downcast<T: Send + Sync + 'static>(self) -> Result<Box<T>, Box<dyn Any + Send + Sync>> {
        (self.0 as Box<dyn Any + Send + Sync>).downcast::<T>()
    }

    pub fn downcast_ref<T: 'static>(&self) -> Option<&T> {
        (&self.0 as &dyn Any).downcast_ref()
    }

    pub fn downcast_mut<T: 'static>(&mut self) -> Option<&mut T> {
        (&mut self.0 as &mut dyn Any).downcast_mut()
    }
}
