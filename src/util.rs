use std::{iter::FusedIterator, ops::ControlFlow};

pub trait IteratorExt: Iterator {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn continue_all_items() {
        let input = [1, 2, 3];
        let out: Vec<_> = input
            .into_iter()
            .map_until_inclusive(|x| ControlFlow::Continue(x * 2))
            .collect();
        assert_eq!(out, vec![2, 4, 6]);
    }

    #[test]
    fn break_is_inclusive() {
        let input = [1, 2, 3, 4, 5];
        let out: Vec<_> = input
            .into_iter()
            .map_until_inclusive(|x| {
                if x == 3 {
                    ControlFlow::Break(x * 10)
                } else {
                    ControlFlow::Continue(x * 10)
                }
            })
            .collect();
        assert_eq!(out, vec![10, 20, 30]); // includes the break value, then stops
    }
}
