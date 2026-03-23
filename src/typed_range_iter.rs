use crate::{IndexScalarType, IndexType};

/// A trait which allows iterating over ranges which use custom index types.
///
/// The existing range types (e.g, [`Range`](core::ops::Range)) only support iteration when the underlying type implements the
/// [`Step`](core::iter::Step) trait, which is currently unstable so we can't implement it for our own types.
///
/// So, this type provides an extension trait which allows converting each range type to an iterable version of it which supports iteration
/// on every index type implementing the [`IndexType`] trait.
pub trait TypedRangeIterExt<I: IndexType> {
    type Iter: Iterator<Item = I>;

    fn iter(self) -> Self::Iter;
}
impl<I: IndexType> TypedRangeIterExt<I> for core::ops::Range<I> {
    type Iter = TypedRangeIter<I>;

    #[inline]
    fn iter(self) -> Self::Iter {
        TypedRangeIter(self)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TypedRangeIter<I: IndexType>(pub core::ops::Range<I>);

impl<I: IndexType> Iterator for TypedRangeIter<I> {
    type Item = I;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.0.start >= self.0.end {
            return None;
        }
        let res = self.0.start;
        self.0.start = unsafe { res.unchecked_add_scalar(I::Scalar::ONE) };
        Some(res)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        if self.0.start < self.0.end {
            let steps = unsafe { self.0.end.unchecked_sub_index(self.0.start) }.to_usize();
            (steps, Some(steps))
        } else {
            (0, Some(0))
        }
    }

    #[inline]
    fn count(self) -> usize {
        if self.0.start < self.0.end {
            unsafe { self.0.end.unchecked_sub_index(self.0.start) }.to_usize()
        } else {
            0
        }
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<I> {
        let res = self
            .0
            .start
            .checked_add_scalar(I::Scalar::try_from_usize(n)?)
            .ok()?;

        if res >= self.0.end {
            return None;
        }

        // SAFETY: We know that res is less than end, so adding one must not overflow
        self.0.start = unsafe { res.unchecked_add_scalar(I::Scalar::ONE) };

        Some(res)
    }

    #[inline]
    fn last(mut self) -> Option<I> {
        self.next_back()
    }

    #[inline]
    fn min(mut self) -> Option<I>
    where
        I: Ord,
    {
        self.next()
    }

    #[inline]
    fn max(mut self) -> Option<I>
    where
        I: Ord,
    {
        self.next_back()
    }

    #[inline]
    fn is_sorted(self) -> bool {
        true
    }
}

impl<I: IndexType> DoubleEndedIterator for TypedRangeIter<I> {
    #[inline]
    fn next_back(&mut self) -> Option<I> {
        if self.0.start >= self.0.end {
            return None;
        }
        let res = unsafe { self.0.end.unchecked_sub_scalar(I::Scalar::ONE) };
        self.0.end = res;
        Some(res)
    }

    #[inline]
    fn nth_back(&mut self, n: usize) -> Option<I> {
        let res = self
            .0
            .end
            .checked_sub_scalar(I::Scalar::try_from_usize(n)?)?
            .checked_sub_scalar(I::Scalar::ONE)?;

        if res < self.0.start {
            return None;
        }

        self.0.end = res;

        Some(res)
    }
}
