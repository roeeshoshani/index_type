//! Extension traits and iterators for iterating over ranges with custom index types.
//!
//! The standard library's range types ([`core::ops::Range`], [`core::ops::RangeFrom`], [`core::ops::RangeInclusive`]) cannot be directly
//! iterated over with custom index types because they require the [`core::iter::Step`] trait, which is
//! currently unstable. This module provides extension traits that convert range types into
//! iterator types that work with any [`IndexType`].
//!
//! # Example
//!
//! ```
//! use index_type::IndexType;
//! use index_type::range::TypedRangeIterExt;
//!
//! #[derive(IndexType, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
//! struct MyIdx(u32);
//!
//! // Iterate over a range using your custom index type
//! for idx in (MyIdx(5)..MyIdx(10)).iter() {
//!     println!("{:?}", idx);
//! }
//! ```

use core::iter::FusedIterator;

use crate::{IndexScalarType, IndexType};

/// An extension trait that provides `iter()` method on range types.
///
/// This allows iterating over ranges using custom index types.
///
/// # Example
///
/// ```
/// use index_type::IndexType;
/// use index_type::range::TypedRangeIterExt;
///
/// #[derive(IndexType, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
/// struct Idx(u32);
///
/// let start = Idx(0);
/// let end = Idx(5);
///
/// for i in (start..end).iter() {
///     println!("{:?}", i);
/// }
/// ```
pub trait TypedRangeIterExt<I: IndexType> {
    /// The iterator type produced by calling `iter()`.
    type Iter: Iterator<Item = I>;

    /// Converts the range into an iterator.
    fn iter(self) -> Self::Iter;
}

impl<I: IndexType> TypedRangeIterExt<I> for core::ops::Range<I> {
    type Iter = TypedRange<I>;

    #[inline]
    fn iter(self) -> Self::Iter {
        TypedRange::from_raw(self)
    }
}

/// A (half-open) range bounded inclusively below and exclusively above (`start..end`) which supports iteration using custom index types.
///
/// The range `start..end` contains all values with `start <= x < end`.
/// It is empty if `start >= end`.
#[derive(Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TypedRange<I: IndexType> {
    /// The lower bound of the range (inclusive).
    pub start: I,
    /// The upper bound of the range (exclusive).
    pub end: I,
}

impl<I: IndexType + core::fmt::Debug> core::fmt::Debug for TypedRange<I> {
    fn fmt(&self, fmt: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(fmt, "{:?}..{:?}", self.start, self.end)
    }
}

impl<I: IndexType> TypedRange<I> {
    /// Converts this range into a raw range.
    #[inline]
    pub const fn into_raw(self) -> core::ops::Range<I> {
        self.start..self.end
    }

    /// Converts a raw range into a typed range.
    #[inline]
    pub const fn from_raw(value: core::ops::Range<I>) -> Self {
        Self {
            start: value.start,
            end: value.end,
        }
    }

    /// Returns the number of elements in the range.
    ///
    /// Returns 0 if `start >= end`.
    #[inline]
    pub fn len(&self) -> usize {
        self.end
            .checked_sub_index(self.start)
            .unwrap_or(I::Scalar::ZERO)
            .to_usize()
    }

    /// Returns `true` if the range contains no elements.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.start >= self.end
    }
}

impl<I: IndexType> From<core::ops::Range<I>> for TypedRange<I> {
    fn from(value: core::ops::Range<I>) -> Self {
        Self::from_raw(value)
    }
}

impl<I: IndexType> Iterator for TypedRange<I> {
    type Item = I;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.start >= self.end {
            return None;
        }
        let res = self.start;
        self.start = unsafe { res.unchecked_add_scalar(I::Scalar::ONE) };
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
        let Some(offset) = I::Scalar::try_from_usize(n) else {
            self.start = self.end;
            return None;
        };

        let Ok(res) = self.start.checked_add_scalar(offset) else {
            self.start = self.end;
            return None;
        };

        if res >= self.end {
            self.start = self.end;
            return None;
        }

        self.start = unsafe { res.unchecked_add_scalar(I::Scalar::ONE) };

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

impl<I: IndexType> DoubleEndedIterator for TypedRange<I> {
    #[inline]
    fn next_back(&mut self) -> Option<I> {
        if self.start >= self.end {
            return None;
        }
        let res = unsafe { self.end.unchecked_sub_scalar(I::Scalar::ONE) };
        self.end = res;
        Some(res)
    }

    #[inline]
    fn nth_back(&mut self, n: usize) -> Option<I> {
        let Some(offset) = I::Scalar::try_from_usize(n) else {
            self.end = self.start;
            return None;
        };

        let Some(res) = self
            .end
            .checked_sub_scalar(offset)
            .and_then(|x| x.checked_sub_scalar(I::Scalar::ONE))
        else {
            self.end = self.start;
            return None;
        };

        if res < self.start {
            self.end = self.start;
            return None;
        }

        self.end = res;

        Some(res)
    }
}

impl<I: IndexType> ExactSizeIterator for TypedRange<I> {
    #[inline]
    fn len(&self) -> usize {
        self.len()
    }
}

impl<I: IndexType> FusedIterator for TypedRange<I> {}

impl<I: IndexType> TypedRangeIterExt<I> for core::ops::RangeFrom<I> {
    type Iter = TypedRangeFrom<I>;

    #[inline]
    fn iter(self) -> Self::Iter {
        TypedRangeFrom::from_raw(self)
    }
}

/// A range only bounded inclusively below (`start..`) which supports iteration using custom index types.
///
/// This range contains all values with `x >= start`.
#[derive(Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TypedRangeFrom<I: IndexType> {
    /// The lower bound of the range (inclusive).
    pub start: I,
}

impl<I: IndexType + core::fmt::Debug> core::fmt::Debug for TypedRangeFrom<I> {
    fn fmt(&self, fmt: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(fmt, "{:?}..", self.start)
    }
}

impl<I: IndexType> TypedRangeFrom<I> {
    /// Converts this range into a raw range.
    #[inline]
    pub const fn into_raw(self) -> core::ops::RangeFrom<I> {
        self.start..
    }

    /// Converts a raw range into a typed range.
    pub const fn from_raw(value: core::ops::RangeFrom<I>) -> Self {
        Self { start: value.start }
    }
}

impl<I: IndexType> From<core::ops::RangeFrom<I>> for TypedRangeFrom<I> {
    fn from(value: core::ops::RangeFrom<I>) -> Self {
        Self::from_raw(value)
    }
}

impl<I: IndexType> Iterator for TypedRangeFrom<I> {
    type Item = I;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let res = self.start;
        self.start = res.checked_add_scalar(I::Scalar::ONE).unwrap();
        Some(res)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (usize::MAX, None)
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<I> {
        let res = self
            .start
            .checked_add_scalar(I::Scalar::try_from_usize(n).unwrap())
            .unwrap();

        self.start = res.checked_add_scalar(I::Scalar::ONE).unwrap();

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

impl<I: IndexType> FusedIterator for TypedRangeFrom<I> {}

impl<I: IndexType> TypedRangeIterExt<I> for core::ops::RangeInclusive<I> {
    type Iter = TypedRangeInclusiveIter<I>;

    #[inline]
    fn iter(self) -> Self::Iter {
        TypedRangeInclusive::from_raw(self).iter()
    }
}

/// A range bounded inclusively below and above (`start..=end`).
///
/// The range `start..=end` contains all values with `start <= x <= end`.
/// It is empty if `start > end`.
///
/// Unlike [`TypedRangeInclusiveIter`], this type is a range rather than an
/// iterator. Use [`TypedRangeInclusive::iter`] to iterate over it.
#[derive(Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TypedRangeInclusive<I: IndexType> {
    /// The lower bound of the range (inclusive).
    pub start: I,
    /// The upper bound of the range (inclusive).
    pub end: I,
}

impl<I: IndexType + core::fmt::Debug> core::fmt::Debug for TypedRangeInclusive<I> {
    fn fmt(&self, fmt: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(fmt, "{:?}..={:?}", self.start, self.end)
    }
}

impl<I: IndexType> TypedRangeInclusive<I> {
    /// Converts this range into a raw range.
    #[inline]
    pub const fn into_raw(self) -> core::ops::RangeInclusive<I> {
        self.start..=self.end
    }

    /// Converts a raw range into a typed range.
    ///
    /// The standard library does not specify the values of the bounds of a
    /// `RangeInclusive` after it has been exhausted. Consequently, if
    /// `range` is exhausted, the value returned by this function is
    /// unspecified and depends on the internal implementation of
    /// [`RangeInclusive`](core::ops::RangeInclusive) in the version of Rust
    /// being used. This function does not preserve or provide any guarantees
    /// about the exhausted state of the raw range.
    #[inline]
    pub const fn from_raw(range: core::ops::RangeInclusive<I>) -> Self {
        Self {
            start: *range.start(),
            end: *range.end(),
        }
    }

    /// Creates an iterator over this range.
    ///
    /// Note that unlike the other range types (e.g [`TypedRange`]), the inclusive range type itself is not an iterator, and iteration
    /// over it is provided using a separate helper type: [`TypedRangeInclusiveIter`]. This separation is done since iterating over an
    /// inclusive range requires storing extra state other than the start and end indices due to overflow related problems. This extra
    /// state does not belong in the core [`TypedRangeInclusive`] type, and is instead used only in the iterator type
    /// [`TypedRangeInclusiveIter`].
    #[inline]
    pub const fn iter(self) -> TypedRangeInclusiveIter<I> {
        TypedRangeInclusiveIter {
            start: self.start,
            end: self.end,
            exhausted: false,
        }
    }

    /// Returns `true` if the range contains no elements.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.start > self.end
    }

    /// Tries to compute the number of elements remaining in the iterator, returning `None` in case the resulting length overflows `usize`.
    #[inline]
    pub fn try_len(&self) -> Option<usize> {
        let Some(diff) = self.end.checked_sub_index(self.start) else {
            return Some(0);
        };
        diff.to_usize().checked_add(1)
    }

    /// Returns the number of elements in the range.
    #[inline]
    pub fn len(&self) -> usize {
        self.try_len()
            .expect("inclusive range length overflowed usize")
    }
}

impl<I: IndexType> From<core::ops::RangeInclusive<I>> for TypedRangeInclusive<I> {
    fn from(value: core::ops::RangeInclusive<I>) -> Self {
        Self::from_raw(value)
    }
}

/// An iterator over a [`TypedRangeInclusive`].
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct TypedRangeInclusiveIter<I: IndexType> {
    start: I,
    end: I,
    exhausted: bool,
}

impl<I: IndexType + core::fmt::Debug> core::fmt::Debug for TypedRangeInclusiveIter<I> {
    fn fmt(&self, fmt: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(fmt, "{:?}..={:?}", self.start, self.end)?;
        if self.exhausted {
            write!(fmt, " (exhausted)")?;
        }
        Ok(())
    }
}

impl<I: IndexType> TypedRangeInclusiveIter<I> {
    /// Returns the starting index of the iterator.
    ///
    /// The returned value is unspecified if the iterator is exhausted.
    #[inline]
    pub const fn start(&self) -> I {
        self.start
    }

    /// Returns the ending index of the iterator.
    ///
    /// The returned value is unspecified if the iterator is exhausted.
    #[inline]
    pub const fn end(&self) -> I {
        self.end
    }

    /// Returns `true` if the iterator contains no elements or has overflowed.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.exhausted || self.start > self.end
    }

    /// Tries to compute the number of elements remaining in the iterator, returning `None` in case the resulting length overflows `usize`.
    #[inline]
    pub fn try_len(&self) -> Option<usize> {
        if self.exhausted {
            return Some(0);
        }
        let Some(diff) = self.end.checked_sub_index(self.start) else {
            return Some(0);
        };
        diff.to_usize().checked_add(1)
    }

    /// Returns the number of elements remaining in the iterator.
    #[inline]
    pub fn len(&self) -> usize {
        self.try_len()
            .expect("inclusive range length overflowed usize")
    }
}

impl<I: IndexType> Iterator for TypedRangeInclusiveIter<I> {
    type Item = I;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if TypedRangeInclusiveIter::is_empty(self) {
            return None;
        }
        let res = self.start;
        match self.start.checked_add_scalar(I::Scalar::ONE) {
            Ok(start_plus_1) => self.start = start_plus_1,
            Err(_) => self.exhausted = true,
        }
        Some(res)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        match self.try_len() {
            Some(len) => (len, Some(len)),
            None => {
                // The length is too big to fit in a `usize`.
                // Return a best effort size hint.
                // The upper bound can't really be expressed using `usize`, so we return `None` instead.
                (usize::MAX, None)
            }
        }
    }

    #[inline]
    fn count(self) -> usize {
        self.len()
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<I> {
        if TypedRangeInclusiveIter::is_empty(self) {
            return None;
        }

        let Some(offset) = I::Scalar::try_from_usize(n) else {
            self.exhausted = true;
            return None;
        };

        let Ok(res) = self.start.checked_add_scalar(offset) else {
            self.exhausted = true;
            return None;
        };

        if res > self.end {
            self.exhausted = true;
            return None;
        }

        match res.checked_add_scalar(I::Scalar::ONE) {
            Ok(res_plus_1) => {
                self.start = res_plus_1;
            }
            Err(_) => {
                self.exhausted = true;
            }
        }

        Some(res)
    }

    #[inline]
    fn last(mut self) -> Option<Self::Item> {
        self.next_back()
    }

    #[inline]
    fn min(mut self) -> Option<Self::Item>
    where
        I: Ord,
    {
        self.next()
    }

    #[inline]
    fn max(mut self) -> Option<Self::Item>
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
        if TypedRangeInclusiveIter::is_empty(self) {
            return None;
        }
        let res = self.end;
        match self.end.checked_sub_scalar(I::Scalar::ONE) {
            Some(end_minus_1) => {
                self.end = end_minus_1;
            }
            None => {
                self.exhausted = true;
            }
        };
        Some(res)
    }

    #[inline]
    fn nth_back(&mut self, n: usize) -> Option<I> {
        if TypedRangeInclusiveIter::is_empty(self) {
            return None;
        }

        let Some(offset) = I::Scalar::try_from_usize(n) else {
            self.exhausted = true;
            return None;
        };

        let Some(res) = self.end.checked_sub_scalar(offset) else {
            self.exhausted = true;
            return None;
        };

        if res < self.start {
            self.exhausted = true;
            return None;
        }

        match res.checked_sub_scalar(I::Scalar::ONE) {
            Some(res_minus_1) => {
                self.end = res_minus_1;
            }
            None => {
                self.exhausted = true;
            }
        }

        Some(res)
    }
}

impl<I: IndexType> ExactSizeIterator for TypedRangeInclusiveIter<I> {
    #[inline]
    fn len(&self) -> usize {
        TypedRangeInclusiveIter::len(self)
    }
}

impl<I: IndexType> FusedIterator for TypedRangeInclusiveIter<I> {}
