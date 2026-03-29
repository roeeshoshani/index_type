use core::iter::FusedIterator;

use crate::{IndexScalarType, IndexType};

#[cold]
#[inline(never)]
fn panic_typed_enumerate_overflow() -> ! {
    panic!("typed enumerate index overflow")
}

/// An iterator adapter like [`Iterator::enumerate`] that yields typed indices.
///
/// This adapter checks on every call that the iterator has not yielded more than
/// `I::MAX_RAW_INDEX` items.
#[derive(Debug, Clone)]
pub struct TypedEnumerate<I: IndexType, Iter> {
    iter: Iter,
    next_index: I,
}

impl<I: IndexType, Iter> TypedEnumerate<I, Iter> {
    /// Creates a new typed enumerate adapter starting at index zero.
    #[inline]
    pub fn new(iter: Iter) -> Self {
        Self {
            iter,
            next_index: I::ZERO,
        }
    }
}

impl<I: IndexType, Iter: Iterator> Iterator for TypedEnumerate<I, Iter> {
    type Item = (I, Iter::Item);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let idx = self.next_index;
        let next_index = idx
            .checked_add_scalar(I::Scalar::ONE)
            .unwrap_or_else(|_| panic_typed_enumerate_overflow());
        let item = self.iter.next()?;
        self.next_index = next_index;
        Some((idx, item))
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<I: IndexType, Iter: DoubleEndedIterator + ExactSizeIterator> DoubleEndedIterator
    for TypedEnumerate<I, Iter>
{
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        let remaining = self.iter.len();
        if remaining == 0 {
            return None;
        }

        let offset = I::Scalar::try_from_usize(unsafe { remaining.unchecked_sub(1) })
            .unwrap_or_else(|| panic_typed_enumerate_overflow());

        let idx = self
            .next_index
            .checked_add_scalar(offset)
            .unwrap_or_else(|_| panic_typed_enumerate_overflow());

        self.iter.next_back().map(|item| (idx, item))
    }
}

impl<I: IndexType, Iter: ExactSizeIterator> ExactSizeIterator for TypedEnumerate<I, Iter> {
    #[inline]
    fn len(&self) -> usize {
        self.iter.len()
    }
}

impl<I: IndexType, Iter: FusedIterator> FusedIterator for TypedEnumerate<I, Iter> {}

/// An iterator adapter that yields typed indices alongside iterator items.
///
/// Unlike [`TypedEnumerate`], this variant does not perform runtime overflow checks while
/// iterating. The caller must guarantee its length invariant up front.
#[derive(Debug, Clone)]
pub struct UncheckedTypedEnumerate<I: IndexType, Iter> {
    iter: Iter,
    next_index: I,
}

impl<I: IndexType, Iter> UncheckedTypedEnumerate<I, Iter> {
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
        }
    }
}

impl<I: IndexType, Iter: Iterator> Iterator for UncheckedTypedEnumerate<I, Iter> {
    type Item = (I, Iter::Item);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|item| {
            let idx = self.next_index;
            // SAFETY: `UncheckedTypedEnumerate::new` guarantees the iterator can yield at most
            // `I::MAX_RAW_INDEX` items, so advancing the front index by one per yielded item
            // remains within the representable range required by this adapter.
            self.next_index = unsafe { self.next_index.unchecked_add_scalar(I::Scalar::ONE) };
            (idx, item)
        })
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<I: IndexType, Iter: DoubleEndedIterator + ExactSizeIterator> DoubleEndedIterator
    for UncheckedTypedEnumerate<I, Iter>
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
        let offset = unsafe { I::Scalar::from_usize_unchecked(remaining.unchecked_sub(1)) };
        // SAFETY: `next_index + (remaining - 1)` is the back index of the remaining iterator
        // state, which is bounded by the same total-length guarantee from `new`.
        let idx = unsafe { self.next_index.unchecked_add_scalar(offset) };

        self.iter.next_back().map(|item| (idx, item))
    }
}

impl<I: IndexType, Iter: ExactSizeIterator> ExactSizeIterator for UncheckedTypedEnumerate<I, Iter> {
    #[inline]
    fn len(&self) -> usize {
        self.iter.len()
    }
}

impl<I: IndexType, Iter: FusedIterator> FusedIterator for UncheckedTypedEnumerate<I, Iter> {}

/// Extension methods for typed enumerate adapters.
pub trait TypedIteratorExt: Iterator + Sized {
    /// Returns an iterator that yields typed indices alongside items.
    ///
    /// This is like [`Iterator::enumerate`], but it uses an [`IndexType`] for the index.
    #[inline]
    fn typed_enumerate<I: IndexType>(self) -> TypedEnumerate<I, Self> {
        TypedEnumerate::new(self)
    }

    /// Returns an unchecked typed enumerate adapter.
    ///
    /// # Safety
    ///
    /// Starting from the iterator's current state, the total number of items it can still yield
    /// must be at most `I::MAX_RAW_INDEX`.
    ///
    /// Additionally, if `Self` implements [`ExactSizeIterator`], its `len()` must accurately
    /// report the number of remaining items for the current state.
    #[inline]
    unsafe fn typed_enumerate_unchecked<I: IndexType>(self) -> UncheckedTypedEnumerate<I, Self> {
        unsafe { UncheckedTypedEnumerate::new(self) }
    }
}

impl<Iter: Iterator> TypedIteratorExt for Iter {}
