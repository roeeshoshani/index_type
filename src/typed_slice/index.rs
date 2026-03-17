use core::{hint::unreachable_unchecked, ops::RangeBounds};

use crate::{IndexType, typed_slice::TypedSlice};

mod private_typed_slice_index {
    pub trait Sealed {}
}

/// A trait for types that can be used to index into a [`TypedSlice`].
///
/// # Safety
///
/// Implementations must ensure that `get_unchecked` and `get_unchecked_mut` are safe to call
/// if the index is within bounds.
pub unsafe trait TypedSliceIndex<T: ?Sized>: private_typed_slice_index::Sealed {
    /// The output type produced by this index.
    type Output: ?Sized;

    /// Returns a reference to the output at this index, or `None` if out of bounds.
    fn get(self, slice: &T) -> Option<&Self::Output>;

    /// Returns a mutable reference to the output at this index, or `None` if out of bounds.
    fn get_mut(self, slice: &mut T) -> Option<&mut Self::Output>;

    /// Returns a pointer to the output at this index, without checking bounds.
    ///
    /// # Safety
    ///
    /// The index must be within bounds.
    unsafe fn get_unchecked(self, slice: *const T) -> *const Self::Output;

    /// Returns a mutable pointer to the output at this index, without checking bounds.
    ///
    /// # Safety
    ///
    /// The index must be within bounds.
    unsafe fn get_unchecked_mut(self, slice: *mut T) -> *mut Self::Output;

    /// Returns a reference to the output at this index, panicking if out of bounds.
    #[track_caller]
    fn index(self, slice: &T) -> &Self::Output;

    /// Returns a mutable reference to the output at this index, panicking if out of bounds.
    #[track_caller]
    fn index_mut(self, slice: &mut T) -> &mut Self::Output;
}

impl<I: IndexType> private_typed_slice_index::Sealed for I {}

// SAFETY: I::to_index() provides a valid index into the underlying slice.
unsafe impl<I: IndexType, T> TypedSliceIndex<TypedSlice<I, T>> for I {
    type Output = T;

    #[inline]
    fn get(self, slice: &TypedSlice<I, T>) -> Option<&Self::Output> {
        slice.raw.get(self.to_index())
    }

    #[inline]
    fn get_mut(self, slice: &mut TypedSlice<I, T>) -> Option<&mut Self::Output> {
        slice.raw.get_mut(self.to_index())
    }

    #[inline]
    unsafe fn get_unchecked(self, slice: *const TypedSlice<I, T>) -> *const Self::Output {
        let ptr = slice as *const T;
        // SAFETY: The caller guarantees that the index is within bounds.
        unsafe { ptr.add(self.to_index()) }
    }

    #[inline]
    unsafe fn get_unchecked_mut(self, slice: *mut TypedSlice<I, T>) -> *mut Self::Output {
        let ptr = slice as *mut T;
        // SAFETY: The caller guarantees that the index is within bounds.
        unsafe { ptr.add(self.to_index()) }
    }

    #[inline]
    fn index(self, slice: &TypedSlice<I, T>) -> &Self::Output {
        &slice.raw[self.to_index()]
    }

    #[inline]
    fn index_mut(self, slice: &mut TypedSlice<I, T>) -> &mut Self::Output {
        &mut slice.raw[self.to_index()]
    }
}

impl<I: IndexType> private_typed_slice_index::Sealed for core::ops::Range<I> {}

// SAFETY: The range bounds are converted to usize and used to slice the underlying raw slice.
unsafe impl<I: IndexType, T> TypedSliceIndex<TypedSlice<I, T>> for core::ops::Range<I> {
    type Output = TypedSlice<I, T>;

    #[inline]
    fn get(self, slice: &TypedSlice<I, T>) -> Option<&Self::Output> {
        let raw_range = self.start.to_index()..self.end.to_index();
        slice
            .raw
            .get(raw_range)
            .map(|new_slice| {
                // SAFETY: TypedSlice is repr(transparent) over [T].
                unsafe { TypedSlice::from_slice_unchecked(new_slice) }
            })
    }

    #[inline]
    fn get_mut(self, slice: &mut TypedSlice<I, T>) -> Option<&mut Self::Output> {
        let raw_range = self.start.to_index()..self.end.to_index();
        slice
            .raw
            .get_mut(raw_range)
            .map(|new_slice| {
                // SAFETY: TypedSlice is repr(transparent) over [T].
                unsafe { TypedSlice::from_slice_unchecked_mut(new_slice) }
            })
    }

    #[inline]
    unsafe fn get_unchecked(self, slice: *const TypedSlice<I, T>) -> *const Self::Output {
        let raw_range = self.start.to_index()..self.end.to_index();
        let ptr = slice as *const T;
        // SAFETY: The caller guarantees that the range is within bounds.
        unsafe {
            let new_len = raw_range.end.unchecked_sub(raw_range.start);
            core::ptr::slice_from_raw_parts(ptr.add(raw_range.start), new_len) as _
        }
    }

    #[inline]
    unsafe fn get_unchecked_mut(self, slice: *mut TypedSlice<I, T>) -> *mut Self::Output {
        let raw_range = self.start.to_index()..self.end.to_index();
        let ptr = slice as *mut T;
        // SAFETY: The caller guarantees that the range is within bounds.
        unsafe {
            let new_len = raw_range.end.unchecked_sub(raw_range.start);
            core::ptr::slice_from_raw_parts_mut(ptr.add(raw_range.start), new_len) as _
        }
    }

    #[inline]
    fn index(self, slice: &TypedSlice<I, T>) -> &Self::Output {
        let raw_range = self.start.to_index()..self.end.to_index();
        // SAFETY: TypedSlice is repr(transparent) over [T].
        unsafe { TypedSlice::from_slice_unchecked(&slice.raw[raw_range]) }
    }

    #[inline]
    fn index_mut(self, slice: &mut TypedSlice<I, T>) -> &mut Self::Output {
        let raw_range = self.start.to_index()..self.end.to_index();
        // SAFETY: TypedSlice is repr(transparent) over [T].
        unsafe { TypedSlice::from_slice_unchecked_mut(&mut slice.raw[raw_range]) }
    }
}

impl<I: IndexType> private_typed_slice_index::Sealed for core::ops::RangeTo<I> {}
// SAFETY: Delegated to Range<I>.
unsafe impl<I: IndexType, T> TypedSliceIndex<TypedSlice<I, T>> for core::ops::RangeTo<I> {
    type Output = TypedSlice<I, T>;

    #[inline]
    fn get(self, slice: &TypedSlice<I, T>) -> Option<&TypedSlice<I, T>> {
        (I::ZERO..self.end).get(slice)
    }

    #[inline]
    fn get_mut(self, slice: &mut TypedSlice<I, T>) -> Option<&mut TypedSlice<I, T>> {
        (I::ZERO..self.end).get_mut(slice)
    }

    #[inline]
    unsafe fn get_unchecked(self, slice: *const TypedSlice<I, T>) -> *const TypedSlice<I, T> {
        // SAFETY: The caller guarantees that the range is within bounds.
        unsafe { (I::ZERO..self.end).get_unchecked(slice) }
    }

    #[inline]
    unsafe fn get_unchecked_mut(self, slice: *mut TypedSlice<I, T>) -> *mut TypedSlice<I, T> {
        // SAFETY: The caller guarantees that the range is within bounds.
        unsafe { (I::ZERO..self.end).get_unchecked_mut(slice) }
    }

    #[inline(always)]
    fn index(self, slice: &TypedSlice<I, T>) -> &TypedSlice<I, T> {
        (I::ZERO..self.end).index(slice)
    }

    #[inline]
    fn index_mut(self, slice: &mut TypedSlice<I, T>) -> &mut TypedSlice<I, T> {
        (I::ZERO..self.end).index_mut(slice)
    }
}

impl<I: IndexType> private_typed_slice_index::Sealed for core::ops::RangeFrom<I> {}
// SAFETY: Delegated to Range<I>.
unsafe impl<I: IndexType, T> TypedSliceIndex<TypedSlice<I, T>> for core::ops::RangeFrom<I> {
    type Output = TypedSlice<I, T>;

    #[inline]
    fn get(self, slice: &TypedSlice<I, T>) -> Option<&TypedSlice<I, T>> {
        (self.start..slice.len()).get(slice)
    }

    #[inline]
    fn get_mut(self, slice: &mut TypedSlice<I, T>) -> Option<&mut TypedSlice<I, T>> {
        (self.start..slice.len()).get_mut(slice)
    }

    #[inline]
    unsafe fn get_unchecked(self, slice: *const TypedSlice<I, T>) -> *const TypedSlice<I, T> {
        let raw_slice = slice as *const [T];
        // SAFETY: raw_slice comes from a valid TypedSlice pointer.
        let len = unsafe { I::from_index_unchecked(raw_slice.len()) };
        // SAFETY: The caller guarantees that the range is within bounds.
        unsafe { (self.start..len).get_unchecked(slice) }
    }

    #[inline]
    unsafe fn get_unchecked_mut(self, slice: *mut TypedSlice<I, T>) -> *mut TypedSlice<I, T> {
        let raw_slice = slice as *mut [T];
        // SAFETY: raw_slice comes from a valid TypedSlice pointer.
        let len = unsafe { I::from_index_unchecked(raw_slice.len()) };
        // SAFETY: The caller guarantees that the range is within bounds.
        unsafe { (self.start..len).get_unchecked_mut(slice) }
    }

    #[inline(always)]
    fn index(self, slice: &TypedSlice<I, T>) -> &TypedSlice<I, T> {
        (self.start..slice.len()).index(slice)
    }

    #[inline]
    fn index_mut(self, slice: &mut TypedSlice<I, T>) -> &mut TypedSlice<I, T> {
        (self.start..slice.len()).index_mut(slice)
    }
}

impl private_typed_slice_index::Sealed for core::ops::RangeFull {}
// SAFETY: Returns the whole slice, which is always valid.
unsafe impl<I: IndexType, T> TypedSliceIndex<TypedSlice<I, T>> for core::ops::RangeFull {
    type Output = TypedSlice<I, T>;

    #[inline]
    fn get(self, slice: &TypedSlice<I, T>) -> Option<&TypedSlice<I, T>> {
        Some(slice)
    }

    #[inline]
    fn get_mut(self, slice: &mut TypedSlice<I, T>) -> Option<&mut TypedSlice<I, T>> {
        Some(slice)
    }

    #[inline]
    unsafe fn get_unchecked(self, slice: *const TypedSlice<I, T>) -> *const TypedSlice<I, T> {
        slice
    }

    #[inline]
    unsafe fn get_unchecked_mut(self, slice: *mut TypedSlice<I, T>) -> *mut TypedSlice<I, T> {
        slice
    }

    #[inline(always)]
    fn index(self, slice: &TypedSlice<I, T>) -> &TypedSlice<I, T> {
        slice
    }

    #[inline]
    fn index_mut(self, slice: &mut TypedSlice<I, T>) -> &mut TypedSlice<I, T> {
        slice
    }
}

/// Converts a `RangeInclusive` to a `Range` without checking for overflow.
///
/// # Safety
///
/// `r.end() + 1` must be representable by `I`.
unsafe fn range_inclusive_to_exclusive_unchecked<I: IndexType>(
    r: core::ops::RangeInclusive<I>,
) -> core::ops::Range<I> {
    // this special code is used to handle the quirks of `RangeInclusive` related to the `exhausted` field.
    // see `RangeInclusive::into_slice_range`. the `end_bound` function can be used as a "side channel" to get the value of the
    // `exhausted` field which is not exposed directly in any public API.
    let exclusive_end_index = match r.end_bound() {
        core::ops::Bound::Included(&i) => {
            // SAFETY: The caller guarantees that i + 1 is representable.
            unsafe { i.unchecked_add_usize(1) }
        }
        core::ops::Bound::Excluded(&i) => i,
        core::ops::Bound::Unbounded => {
            // SAFETY: RangeInclusive always has a bound.
            unsafe { unreachable_unchecked() }
        }
    };
    *r.start()..exclusive_end_index
}

impl<I: IndexType> private_typed_slice_index::Sealed for core::ops::RangeInclusive<I> {}
// SAFETY: Delegated to the underlying raw slice indexing.
unsafe impl<I: IndexType, T> TypedSliceIndex<TypedSlice<I, T>> for core::ops::RangeInclusive<I> {
    type Output = TypedSlice<I, T>;

    #[inline]
    fn get(self, slice: &TypedSlice<I, T>) -> Option<&Self::Output> {
        let raw_range = self.start().to_index()..=self.end().to_index();
        slice
            .raw
            .get(raw_range)
            .map(|new_slice| {
                // SAFETY: TypedSlice is repr(transparent) over [T].
                unsafe { TypedSlice::from_slice_unchecked(new_slice) }
            })
    }

    #[inline]
    fn get_mut(self, slice: &mut TypedSlice<I, T>) -> Option<&mut Self::Output> {
        let raw_range = self.start().to_index()..=self.end().to_index();
        slice
            .raw
            .get_mut(raw_range)
            .map(|new_slice| {
                // SAFETY: TypedSlice is repr(transparent) over [T].
                unsafe { TypedSlice::from_slice_unchecked_mut(new_slice) }
            })
    }

    #[inline]
    unsafe fn get_unchecked(self, slice: *const TypedSlice<I, T>) -> *const Self::Output {
        // SAFETY: The caller guarantees that the range is within bounds.
        unsafe { range_inclusive_to_exclusive_unchecked(self).get_unchecked(slice) }
    }

    #[inline]
    unsafe fn get_unchecked_mut(self, slice: *mut TypedSlice<I, T>) -> *mut Self::Output {
        // SAFETY: The caller guarantees that the range is within bounds.
        unsafe { range_inclusive_to_exclusive_unchecked(self).get_unchecked_mut(slice) }
    }

    #[inline]
    fn index(self, slice: &TypedSlice<I, T>) -> &Self::Output {
        let raw_range = self.start().to_index()..=self.end().to_index();
        // SAFETY: TypedSlice is repr(transparent) over [T].
        unsafe { TypedSlice::from_slice_unchecked(&slice.raw[raw_range]) }
    }

    #[inline]
    fn index_mut(self, slice: &mut TypedSlice<I, T>) -> &mut Self::Output {
        let raw_range = self.start().to_index()..=self.end().to_index();
        // SAFETY: TypedSlice is repr(transparent) over [T].
        unsafe { TypedSlice::from_slice_unchecked_mut(&mut slice.raw[raw_range]) }
    }
}

impl<I: IndexType> private_typed_slice_index::Sealed for core::ops::RangeToInclusive<I> {}
// SAFETY: Delegated to RangeInclusive<I>.
unsafe impl<I: IndexType, T> TypedSliceIndex<TypedSlice<I, T>> for core::ops::RangeToInclusive<I> {
    type Output = TypedSlice<I, T>;

    #[inline]
    fn get(self, slice: &TypedSlice<I, T>) -> Option<&TypedSlice<I, T>> {
        (I::ZERO..=self.end).get(slice)
    }

    #[inline]
    fn get_mut(self, slice: &mut TypedSlice<I, T>) -> Option<&mut TypedSlice<I, T>> {
        (I::ZERO..=self.end).get_mut(slice)
    }

    #[inline]
    unsafe fn get_unchecked(self, slice: *const TypedSlice<I, T>) -> *const TypedSlice<I, T> {
        // SAFETY: The caller guarantees that the range is within bounds.
        unsafe { (I::ZERO..=self.end).get_unchecked(slice) }
    }

    #[inline]
    unsafe fn get_unchecked_mut(self, slice: *mut TypedSlice<I, T>) -> *mut TypedSlice<I, T> {
        // SAFETY: The caller guarantees that the range is within bounds.
        unsafe { (I::ZERO..=self.end).get_unchecked_mut(slice) }
    }

    #[inline(always)]
    fn index(self, slice: &TypedSlice<I, T>) -> &TypedSlice<I, T> {
        (I::ZERO..=self.end).index(slice)
    }

    #[inline]
    fn index_mut(self, slice: &mut TypedSlice<I, T>) -> &mut TypedSlice<I, T> {
        (I::ZERO..=self.end).index_mut(slice)
    }
}
