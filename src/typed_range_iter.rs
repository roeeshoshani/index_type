use core::iter::FusedIterator;

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

impl<I: IndexType> TypedRangeIter<I> {
    #[inline]
    pub fn len(&self) -> usize {
        if self.0.start < self.0.end {
            unsafe { self.0.end.unchecked_sub_index(self.0.start) }.to_usize()
        } else {
            0
        }
    }
}

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
        let len = self.len();
        (len, Some(len))
    }

    #[inline]
    fn count(self) -> usize {
        self.len()
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

impl<I: IndexType> ExactSizeIterator for TypedRangeIter<I> {
    #[inline]
    fn len(&self) -> usize {
        self.len()
    }
}

impl<I: IndexType> FusedIterator for TypedRangeIter<I> {}

impl<I: IndexType> TypedRangeIterExt<I> for core::ops::RangeFrom<I> {
    type Iter = TypedRangeFromIter<I>;

    #[inline]
    fn iter(self) -> Self::Iter {
        TypedRangeFromIter(self)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TypedRangeFromIter<I: IndexType>(pub core::ops::RangeFrom<I>);

impl<I: IndexType> Iterator for TypedRangeFromIter<I> {
    type Item = I;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let res = self.0.start;
        self.0.start = res.checked_add_scalar(I::Scalar::ONE).unwrap();
        Some(res)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (usize::MAX, None)
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<I> {
        let res = self
            .0
            .start
            .checked_add_scalar(I::Scalar::try_from_usize(n).unwrap())
            .unwrap();

        self.0.start = res.checked_add_scalar(I::Scalar::ONE).unwrap();

        Some(res)
    }

    #[inline]
    fn min(mut self) -> Option<I>
    where
        I: Ord,
    {
        self.next()
    }

    #[inline]
    fn is_sorted(self) -> bool {
        true
    }
}

impl<I: IndexType> FusedIterator for TypedRangeFromIter<I> {}

impl<I: IndexType> TypedRangeIterExt<I> for core::ops::RangeInclusive<I> {
    type Iter = TypedRangeInclusiveIter<I>;

    #[inline]
    fn iter(self) -> Self::Iter {
        TypedRangeInclusiveIter::new(self)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TypedRangeInclusiveIter<I: IndexType> {
    start: I,
    end: I,
    exhausted: bool,
}
impl<I: IndexType> TypedRangeInclusiveIter<I> {
    #[inline]
    pub fn new(range: core::ops::RangeInclusive<I>) -> Self {
        Self {
            start: *range.start(),
            end: *range.end(),
            exhausted: false,
        }
    }
}

impl<I: IndexType> TypedRangeInclusiveIter<I> {
    #[inline]
    pub fn start(&self) -> I {
        self.start
    }

    #[inline]
    pub fn end(&self) -> I {
        self.end
    }

    #[inline]
    pub fn len(&self) -> usize {
        if self.exhausted {
            return 0;
        }

        unsafe {
            self.end
                .unchecked_sub_index(self.start)
                .unchecked_add_scalar(I::Scalar::ONE)
        }
        .to_usize()
    }
}

impl<I: IndexType> Iterator for TypedRangeInclusiveIter<I> {
    type Item = I;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.exhausted {
            return None;
        }

        if self.start == self.end {
            self.exhausted = true;
            Some(self.start)
        } else {
            // start < end
            let res = self.start;
            self.start = unsafe { res.unchecked_add_scalar(I::Scalar::ONE) };
            Some(res)
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len();
        (len, Some(len))
    }

    #[inline]
    fn count(self) -> usize {
        self.len()
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<I> {
        if self.exhausted {
            return None;
        }

        let res = self
            .start
            .checked_add_scalar(I::Scalar::try_from_usize(n)?)
            .ok()?;

        if res > self.end {
            None
        } else if res == self.end {
            self.exhausted = true;
            Some(res)
        } else {
            // res < end

            // SAFETY: We know that res is less than end, so adding one must not overflow
            self.start = unsafe { res.unchecked_add_scalar(I::Scalar::ONE) };

            Some(res)
        }
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

impl<I: IndexType> DoubleEndedIterator for TypedRangeInclusiveIter<I> {
    #[inline]
    fn next_back(&mut self) -> Option<I> {
        if self.exhausted {
            return None;
        }

        if self.end == self.start {
            self.exhausted = true;
            Some(self.end)
        } else {
            // end > start
            let res = self.end;
            self.end = unsafe { res.unchecked_sub_scalar(I::Scalar::ONE) };
            Some(res)
        }
    }

    #[inline]
    fn nth_back(&mut self, n: usize) -> Option<I> {
        if self.exhausted {
            return None;
        }

        let res = self.end.checked_sub_scalar(I::Scalar::try_from_usize(n)?)?;

        if res < self.start {
            None
        } else if res == self.start {
            self.exhausted = true;
            Some(res)
        } else {
            // res > start

            // SAFETY: We know that res is greater than start, so subtracting one must not overflow
            self.end = unsafe { res.unchecked_sub_scalar(I::Scalar::ONE) };

            Some(res)
        }
    }
}

impl<I: IndexType> ExactSizeIterator for TypedRangeInclusiveIter<I> {
    fn len(&self) -> usize {
        self.len()
    }
}

impl<I: IndexType> FusedIterator for TypedRangeInclusiveIter<I> {}
