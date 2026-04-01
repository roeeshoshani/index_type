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
//! use index_type::typed_range::TypedRangeIterExt;
//!
//! #[derive(IndexType, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
//! struct MyIdx(u32);
//!
//! // Iterate over a range using your custom index type
//! for idx in (MyIdx(5)..MyIdx(10)).iter() {
//!     println!("{:?}", idx);
//! }
//! ```

use core::{hint::unreachable_unchecked, iter::FusedIterator, ops::RangeBounds};

use crate::{IndexScalarType, IndexType};

/// An extension trait that provides `iter()` method on range types.
///
/// This allows iterating over ranges using custom index types.
///
/// # Example
///
/// ```
/// use index_type::IndexType;
/// use index_type::typed_range::TypedRangeIterExt;
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
pub struct TypedRange<I: IndexType> {
    /// The lower bound of the range (inclusive).
    pub start: I,
    /// The upper bound of the range (exclusive).
    pub end: I,
}

impl<I: IndexType + core::fmt::Debug> core::fmt::Debug for TypedRange<I> {
    fn fmt(&self, fmt: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.start.fmt(fmt)?;
        write!(fmt, "..")?;
        self.end.fmt(fmt)?;
        Ok(())
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
pub struct TypedRangeFrom<I: IndexType> {
    /// The lower bound of the range (inclusive).
    pub start: I,
}

impl<I: IndexType + core::fmt::Debug> core::fmt::Debug for TypedRangeFrom<I> {
    fn fmt(&self, fmt: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.start.fmt(fmt)?;
        write!(fmt, "..")?;
        Ok(())
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
    type Iter = TypedRangeInclusive<I>;

    #[inline]
    fn iter(self) -> Self::Iter {
        TypedRangeInclusive::from_raw_lossy(self)
    }
}

/// A range bounded inclusively below and above (`start..=end`) which supports iteration using custom index types.
///
/// The `RangeInclusive` `start..=end` contains all values with `x >= start`
/// and `x <= end`. It is empty unless `start <= end`.
///
/// This iterator is [fused], but the specific values of `start` and `end` after
/// iteration has finished are **unspecified** other than that [`.is_empty()`]
/// will return `true` once no more values will be produced.
///
/// [fused]: FusedIterator
/// [`.is_empty()`]: TypedRangeInclusive::is_empty
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct TypedRangeInclusive<I: IndexType> {
    start: I,
    end: I,
    exhausted: bool,
}

impl<I: IndexType + core::fmt::Debug> core::fmt::Debug for TypedRangeInclusive<I> {
    fn fmt(&self, fmt: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.start.fmt(fmt)?;
        write!(fmt, "..=")?;
        self.end.fmt(fmt)?;
        if self.exhausted {
            write!(fmt, " (exhausted)")?;
        }
        Ok(())
    }
}

impl<I: IndexType> TypedRangeInclusive<I> {
    /// Converts this range into a raw range.
    ///
    /// Note that in the case of [`TypedRangeInclusive`], unlike the other typed range types, this is a lossy conversion.
    /// The lost information is the value of the internal `exhausted` field of [`RangeInclusive`](core::ops::RangeInclusive) which can't
    /// be copied over since there is no way to set it if the `Idx` type does not implement the unstable `Step` trait, and we can't use
    /// the `Step` trait in trait bounds (e.g `I: Step`) as it is unstable.
    #[inline]
    pub const fn into_raw(self) -> core::ops::RangeInclusive<I> {
        self.start..=self.end
    }

    /// Converts a raw range into a typed range, with potential loss of information about the `exhausted` state of the raw range.
    ///
    /// Getting a [`RangeInclusive`](core::ops::RangeInclusive) object into `exhausted` state when the index type `I` does not
    /// implement the unstable `Step` trait is currently not possible using stable rust, since the range can't possibly be iterated.
    ///
    /// All types other than standard rust integers (e.g u32, usize), for example user defined types or non zero integer types, do not
    /// implement the `Step` trait. So, for all such types, using this function will not actually cause any loss of information.
    ///
    /// Additionally, if you are certain that your range is not exhausted, for example since it is a freshly constructed
    /// range (e.g `0..=5`), then you can safely use this function.
    ///
    /// If keeping the value of the `exhausted` state of the iterator is important to you, or if you are unsure whether you need it or
    /// not and you are ok with running a couple more opcodes, use [`from_raw`](TypedRangeInclusive::from_raw) instead.
    #[inline]
    pub const fn from_raw_lossy(range: core::ops::RangeInclusive<I>) -> Self {
        let start = *range.start();
        let end = *range.end();
        Self {
            start,
            end,
            exhausted: false,
        }
    }

    /// Converts a raw range into a typed range.
    pub fn from_raw(range: core::ops::RangeInclusive<I>) -> Self {
        // this special code is used to handle the quirks of `RangeInclusive` related to the `exhausted` field.
        // see `RangeInclusive::into_slice_range`. the `end_bound` function can be used as a "side channel" to get the value of the
        // `exhausted` field which is not exposed directly in any public API.
        match range.end_bound() {
            core::ops::Bound::Excluded(i) => Self {
                start: *range.start(),
                end: *i,
                exhausted: true,
            },
            core::ops::Bound::Included(i) => {
                let start = *range.start();
                let end = *i;
                Self {
                    start,
                    end,
                    exhausted: false,
                }
            }
            core::ops::Bound::Unbounded => unsafe { unreachable_unchecked() },
        }
    }

    /// Returns the starting index of the range.
    #[inline]
    pub fn start(&self) -> I {
        self.start
    }

    /// Returns the ending index of the range.
    #[inline]
    pub fn end(&self) -> I {
        self.end
    }

    /// Exhausts this iterator by pointing the start to the end and setting the exhausted flag.
    fn exhasut_from_start(&mut self) {
        self.start = self.end;
        self.exhausted = true;
    }

    /// Exhausts this iterator by pointing the end to the start and setting the exhausted flag.
    fn exhasut_from_end(&mut self) {
        self.end = self.start;
        self.exhausted = true;
    }

    /// Returns `true` if the range contains no elements, or if the iterator has been exhausted.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.exhausted || (self.start > self.end)
    }

    /// Returns the number of elements in the range.
    ///
    /// Returns 0 if the iterator is exhausted.
    #[inline]
    pub fn len(&self) -> usize {
        if self.exhausted {
            return 0;
        }
        let Some(diff) = self.end.checked_sub_index(self.start) else {
            return 0;
        };
        unsafe { diff.unchecked_add_scalar(I::Scalar::ONE) }.to_usize()
    }
}

impl<I: IndexType> Iterator for TypedRangeInclusive<I> {
    type Item = I;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if TypedRangeInclusive::is_empty(self) {
            return None;
        }

        if self.start == self.end {
            self.exhausted = true;
            Some(self.start)
        } else {
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
        if TypedRangeInclusive::is_empty(self) {
            return None;
        }

        let Some(offset) = I::Scalar::try_from_usize(n) else {
            self.exhasut_from_start();
            return None;
        };

        let Ok(res) = self.start.checked_add_scalar(offset) else {
            self.exhasut_from_start();
            return None;
        };

        if res > self.end {
            self.exhasut_from_start();
            None
        } else if res == self.end {
            self.exhasut_from_start();
            Some(res)
        } else {
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

impl<I: IndexType> DoubleEndedIterator for TypedRangeInclusive<I> {
    #[inline]
    fn next_back(&mut self) -> Option<I> {
        if TypedRangeInclusive::is_empty(self) {
            return None;
        }

        if self.end == self.start {
            self.exhausted = true;
            Some(self.end)
        } else {
            let res = self.end;
            self.end = unsafe { res.unchecked_sub_scalar(I::Scalar::ONE) };
            Some(res)
        }
    }

    #[inline]
    fn nth_back(&mut self, n: usize) -> Option<I> {
        if TypedRangeInclusive::is_empty(self) {
            return None;
        }

        let Some(offset) = I::Scalar::try_from_usize(n) else {
            self.exhasut_from_end();
            return None;
        };

        let Some(res) = self.end.checked_sub_scalar(offset) else {
            self.exhasut_from_end();
            return None;
        };

        if res < self.start {
            self.exhasut_from_end();
            None
        } else if res == self.start {
            self.exhasut_from_end();
            Some(res)
        } else {
            self.end = unsafe { res.unchecked_sub_scalar(I::Scalar::ONE) };
            Some(res)
        }
    }
}

impl<I: IndexType> ExactSizeIterator for TypedRangeInclusive<I> {
    fn len(&self) -> usize {
        self.len()
    }
}

impl<I: IndexType> FusedIterator for TypedRangeInclusive<I> {}
