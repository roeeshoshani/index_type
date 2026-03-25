use core::iter::FusedIterator;

use crate::{IndexScalarType, IndexType};

/// An iterator adapter that yields typed indices alongside iterator items.
#[derive(Debug, Clone)]
pub struct TypedIterEnumerate<I: IndexType, T, Iter> {
    iter: Iter,
    next_index: I,
    marker: core::marker::PhantomData<fn(&T)>,
}

impl<I: IndexType, T, Iter> TypedIterEnumerate<I, T, Iter> {
    /// Creates a new typed enumerate adapter starting at index zero.
    ///
    /// # Safety
    ///
    /// Starting from the iterator's current state, the total number of items it can still yield
    /// must be at most `I::MAX_RAW_INDEX`.
    ///
    /// Additionally, if `Iter` implements [`ExactSizeIterator`], its `len()` must accurately
    /// report the number of remaining items for the current state.
    #[inline]
    pub unsafe fn new(iter: Iter) -> Self {
        Self {
            iter,
            next_index: I::ZERO,
            marker: core::marker::PhantomData,
        }
    }
}

impl<I: IndexType, T, Iter: Iterator> Iterator for TypedIterEnumerate<I, T, Iter> {
    type Item = (I, Iter::Item);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|item| {
            let idx = self.next_index;
            // SAFETY: `TypedIterEnumerate::new` guarantees the iterator can yield at most
            // `I::MAX_RAW_INDEX` items, so advancing the front index by one per yielded item
            // remains within the representable range of `I`.
            self.next_index = unsafe { self.next_index.unchecked_add_scalar(I::Scalar::ONE) };
            (idx, item)
        })
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<I: IndexType, T, Iter: DoubleEndedIterator + ExactSizeIterator> DoubleEndedIterator
    for TypedIterEnumerate<I, T, Iter>
{
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        let remaining = self.iter.len();
        if remaining == 0 {
            return None;
        }

        // SAFETY: `remaining - 1` fits in the scalar type because `new` guarantees the iterator's
        // remaining length fits in `I`, and `ExactSizeIterator::len` is required by the safety
        // contract to be accurate.
        let offset = unsafe { I::Scalar::from_usize_unchecked(remaining - 1) };
        // SAFETY: `next_index + (remaining - 1)` is the back index of the remaining iterator
        // state, which is bounded by the same total-length guarantee from `new`.
        let idx = unsafe { self.next_index.unchecked_add_scalar(offset) };

        self.iter.next_back().map(|item| (idx, item))
    }
}

impl<I: IndexType, T, Iter: ExactSizeIterator> ExactSizeIterator
    for TypedIterEnumerate<I, T, Iter>
{
    fn len(&self) -> usize {
        self.iter.len()
    }
}

impl<I: IndexType, T, Iter: FusedIterator> FusedIterator for TypedIterEnumerate<I, T, Iter> {}
