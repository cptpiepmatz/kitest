use std::{iter::FusedIterator, ops::ControlFlow};

trait IteratorExt: Iterator {
    #[must_use = "iterators are lazy and do nothing unless consumed"]
    fn map_until_inclusive<B, F>(self, f: F) -> impl FusedIterator<Item = B>
    where
        F: FnMut(Self::Item) -> ControlFlow<B, B>;
}

impl<I, T> IteratorExt for I
where
    I: Iterator<Item = T>,
{
    #[inline]
    fn map_until_inclusive<B, F>(self, mut f: F) -> impl FusedIterator<Item = B>
    where
        F: FnMut(Self::Item) -> ControlFlow<B, B>,
    {
        self.scan(true, move |should_continue, item| {
            if !*should_continue {
                return None;
            }

            match f(item) {
                ControlFlow::Continue(item) => Some(item),
                ControlFlow::Break(item) => {
                    *should_continue = false;
                    Some(item)
                }
            }
        })
        .fuse()
    }
}
