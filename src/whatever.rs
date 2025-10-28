use std::{
    any::Any,
    fmt::{Debug, Display},
    ops::Deref,
};

pub type BoxedWhatever = Box<dyn Whatever>;

pub trait Whatever: Any + Debug + Display + Send + Sync + 'static {
    fn clone_whatever(&self) -> BoxedWhatever;
    fn eq_whatever(&self, other: &dyn Whatever) -> bool;
}

impl<T> Whatever for T
where
    T: Any + Debug + Display + Clone + Eq + Send + Sync,
{
    fn clone_whatever(&self) -> BoxedWhatever {
        Box::new(self.clone())
    }

    fn eq_whatever(&self, other: &dyn Whatever) -> bool {
        (other as &dyn Any)
            .downcast_ref::<T>()
            .map(|other| other == self)
            .unwrap_or(false)
    }
}

impl Clone for BoxedWhatever {
    fn clone(&self) -> Self {
        self.clone_whatever()
    }
}

impl PartialEq for BoxedWhatever {
    fn eq(&self, other: &Self) -> bool {
        self.eq_whatever(other.deref())
    }
}

impl Eq for BoxedWhatever {}
