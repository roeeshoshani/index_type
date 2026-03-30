use core::{hint::unreachable_unchecked, ops::RangeBounds};

use crate::{typed_slice::TypedSlice, IndexScalarType, IndexType};

mod private_typed_slice_index {
    pub trait Sealed {}
}

/// A trait for types that can be used to index a [`TypedSlice`].
///
/// This trait is analogous to the [`core::slice::SliceIndex`] trait from the standard library.
/// It enables using various index types to access elements or slices of a `TypedSlice`.
///
/// Implemented for:
/// - Single index types (`I: IndexType`) → returns `&T` or `&mut T`
/// - `Range<I>` → returns `&TypedSlice<I, T>` or `&mut TypedSlice<I, T>`
/// - `RangeFrom<I>`, `RangeTo<I>`, `RangeInclusive<I>`, `RangeToInclusive<I>`
/// - `RangeFull` (the full slice)
///
/// # Safety
///
/// This trait should not be implemented manually.
/// Implementations must uphold invariants about bounds checking, and incorrect implementations
/// may lead to undefined behavior. This trait is sealed to prevent incorrect implementations.
pub unsafe trait TypedSliceIndex<T: ?Sized>: private_typed_slice_index::Sealed {
    /// The output type of the indexing operation.
    type Output: ?Sized;

    /// Returns a reference to the output type, or `None` if out of bounds.
    fn get(self, slice: &T) -> Option<&Self::Output>;

    /// Returns a mutable reference to the output type, or `None` if out of bounds.
    fn get_mut(self, slice: &mut T) -> Option<&mut Self::Output>;

    /// Returns a reference to the output type without bounds checking.
    ///
    /// # Safety
    ///
    /// The index must be in bounds.
    unsafe fn get_unchecked(self, slice: *const T) -> *const Self::Output;

    /// Returns a mutable reference to the output type without bounds checking.
    ///
    /// # Safety
    ///
    /// The index must be in bounds.
    unsafe fn get_unchecked_mut(self, slice: *mut T) -> *mut Self::Output;

    /// Returns a reference to the output type, panicking if out of bounds.
    fn index(self, slice: &T) -> &Self::Output;

    /// Returns a mutable reference to the output type, panicking if out of bounds.
    fn index_mut(self, slice: &mut T) -> &mut Self::Output;
}

impl<I: IndexType> private_typed_slice_index::Sealed for I {}
unsafe impl<I: IndexType, T> TypedSliceIndex<TypedSlice<I, T>> for I {
    type Output = T;

    #[inline]
    fn get(self, slice: &TypedSlice<I, T>) -> Option<&Self::Output> {
        slice.raw.get(self.to_raw_index())
    }

    #[inline]
    fn get_mut(self, slice: &mut TypedSlice<I, T>) -> Option<&mut Self::Output> {
        slice.raw.get_mut(self.to_raw_index())
    }

    #[inline]
    unsafe fn get_unchecked(self, slice: *const TypedSlice<I, T>) -> *const Self::Output {
        let ptr = slice as *const T;
        // SAFETY: The caller ensures the index is in bounds.
        unsafe { ptr.add(self.to_raw_index()) }
    }

    #[inline]
    unsafe fn get_unchecked_mut(self, slice: *mut TypedSlice<I, T>) -> *mut Self::Output {
        let ptr = slice as *mut T;
        // SAFETY: The caller ensures the index is in bounds.
        unsafe { ptr.add(self.to_raw_index()) }
    }

    #[inline]
    fn index(self, slice: &TypedSlice<I, T>) -> &Self::Output {
        &slice.raw[self.to_raw_index()]
    }

    #[inline]
    fn index_mut(self, slice: &mut TypedSlice<I, T>) -> &mut Self::Output {
        &mut slice.raw[self.to_raw_index()]
    }
}

impl<I: IndexType> private_typed_slice_index::Sealed for core::ops::Range<I> {}
unsafe impl<I: IndexType, T> TypedSliceIndex<TypedSlice<I, T>> for core::ops::Range<I> {
    type Output = TypedSlice<I, T>;

    #[inline]
    fn get(self, slice: &TypedSlice<I, T>) -> Option<&Self::Output> {
        let raw_range = self.start.to_raw_index()..self.end.to_raw_index();
        slice
            .raw
            .get(raw_range)
            .map(|new_slice| unsafe { TypedSlice::from_slice_unchecked(new_slice) })
    }

    #[inline]
    fn get_mut(self, slice: &mut TypedSlice<I, T>) -> Option<&mut Self::Output> {
        let raw_range = self.start.to_raw_index()..self.end.to_raw_index();
        slice
            .raw
            .get_mut(raw_range)
            .map(|new_slice| unsafe { TypedSlice::from_slice_unchecked_mut(new_slice) })
    }

    #[inline]
    unsafe fn get_unchecked(self, slice: *const TypedSlice<I, T>) -> *const Self::Output {
        let raw_range = self.start.to_raw_index()..self.end.to_raw_index();
        let ptr = slice as *const T;
        // SAFETY: The caller ensures the range is in bounds.
        unsafe {
            let new_len = raw_range.end.unchecked_sub(raw_range.start);
            core::ptr::slice_from_raw_parts(ptr.add(raw_range.start), new_len) as _
        }
    }

    #[inline]
    unsafe fn get_unchecked_mut(self, slice: *mut TypedSlice<I, T>) -> *mut Self::Output {
        let raw_range = self.start.to_raw_index()..self.end.to_raw_index();
        let ptr = slice as *mut T;
        // SAFETY: The caller ensures the range is in bounds.
        unsafe {
            let new_len = raw_range.end.unchecked_sub(raw_range.start);
            core::ptr::slice_from_raw_parts_mut(ptr.add(raw_range.start), new_len) as _
        }
    }

    #[inline]
    fn index(self, slice: &TypedSlice<I, T>) -> &Self::Output {
        let raw_range = self.start.to_raw_index()..self.end.to_raw_index();
        // SAFETY: index() panics if out of bounds. TypedSlice is repr(transparent) over [T].
        unsafe { TypedSlice::from_slice_unchecked(&slice.raw[raw_range]) }
    }

    #[inline]
    fn index_mut(self, slice: &mut TypedSlice<I, T>) -> &mut Self::Output {
        let raw_range = self.start.to_raw_index()..self.end.to_raw_index();
        // SAFETY: index_mut() panics if out of bounds. TypedSlice is repr(transparent) over [T].
        unsafe { TypedSlice::from_slice_unchecked_mut(&mut slice.raw[raw_range]) }
    }
}

impl<I: IndexType> private_typed_slice_index::Sealed for core::ops::RangeTo<I> {}
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
        // SAFETY: The caller ensures the range is in bounds.
        unsafe { (I::ZERO..self.end).get_unchecked(slice) }
    }

    #[inline]
    unsafe fn get_unchecked_mut(self, slice: *mut TypedSlice<I, T>) -> *mut TypedSlice<I, T> {
        // SAFETY: The caller ensures the range is in bounds.
        unsafe { (I::ZERO..self.end).get_unchecked_mut(slice) }
    }

    #[inline]
    fn index(self, slice: &TypedSlice<I, T>) -> &TypedSlice<I, T> {
        (I::ZERO..self.end).index(slice)
    }

    #[inline]
    fn index_mut(self, slice: &mut TypedSlice<I, T>) -> &mut TypedSlice<I, T> {
        (I::ZERO..self.end).index_mut(slice)
    }
}

impl<I: IndexType> private_typed_slice_index::Sealed for core::ops::RangeFrom<I> {}
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
        // SAFETY: TypedSlice is repr(transparent) over [T].
        let len = unsafe { I::from_raw_index_unchecked(raw_slice.len()) };
        // SAFETY: The caller ensures the range is in bounds.
        unsafe { (self.start..len).get_unchecked(slice) }
    }

    #[inline]
    unsafe fn get_unchecked_mut(self, slice: *mut TypedSlice<I, T>) -> *mut TypedSlice<I, T> {
        let raw_slice = slice as *mut [T];
        // SAFETY: TypedSlice is repr(transparent) over [T].
        let len = unsafe { I::from_raw_index_unchecked(raw_slice.len()) };
        // SAFETY: The caller ensures the range is in bounds.
        unsafe { (self.start..len).get_unchecked_mut(slice) }
    }

    #[inline]
    fn index(self, slice: &TypedSlice<I, T>) -> &TypedSlice<I, T> {
        (self.start..slice.len()).index(slice)
    }

    #[inline]
    fn index_mut(self, slice: &mut TypedSlice<I, T>) -> &mut TypedSlice<I, T> {
        (self.start..slice.len()).index_mut(slice)
    }
}

impl private_typed_slice_index::Sealed for core::ops::RangeFull {}
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

    #[inline]
    fn index(self, slice: &TypedSlice<I, T>) -> &TypedSlice<I, T> {
        slice
    }

    #[inline]
    fn index_mut(self, slice: &mut TypedSlice<I, T>) -> &mut TypedSlice<I, T> {
        slice
    }
}

unsafe fn range_inclusive_to_exclusive_unchecked<I: IndexType>(
    r: core::ops::RangeInclusive<I>,
) -> core::ops::Range<I> {
    // this special code is used to handle the quirks of `RangeInclusive` related to the `exhausted` field.
    // see `RangeInclusive::into_slice_range`. the `end_bound` function can be used as a "side channel" to get the value of the
    // `exhausted` field which is not exposed directly in any public API.
    let exclusive_end_index = match r.end_bound() {
        core::ops::Bound::Included(&i) => unsafe { i.unchecked_add_scalar(I::Scalar::ONE) },
        core::ops::Bound::Excluded(&i) => i,
        core::ops::Bound::Unbounded => unsafe { unreachable_unchecked() },
    };
    *r.start()..exclusive_end_index
}

impl<I: IndexType> private_typed_slice_index::Sealed for core::ops::RangeInclusive<I> {}
unsafe impl<I: IndexType, T> TypedSliceIndex<TypedSlice<I, T>> for core::ops::RangeInclusive<I> {
    type Output = TypedSlice<I, T>;

    #[inline]
    fn get(self, slice: &TypedSlice<I, T>) -> Option<&Self::Output> {
        let raw_range = self.start().to_raw_index()..=self.end().to_raw_index();
        slice
            .raw
            .get(raw_range)
            .map(|new_slice| unsafe { TypedSlice::from_slice_unchecked(new_slice) })
    }

    #[inline]
    fn get_mut(self, slice: &mut TypedSlice<I, T>) -> Option<&mut Self::Output> {
        let raw_range = self.start().to_raw_index()..=self.end().to_raw_index();
        slice
            .raw
            .get_mut(raw_range)
            .map(|new_slice| unsafe { TypedSlice::from_slice_unchecked_mut(new_slice) })
    }

    #[inline]
    unsafe fn get_unchecked(self, slice: *const TypedSlice<I, T>) -> *const Self::Output {
        // SAFETY: The caller ensures the range is in bounds.
        unsafe { range_inclusive_to_exclusive_unchecked(self).get_unchecked(slice) }
    }

    #[inline]
    unsafe fn get_unchecked_mut(self, slice: *mut TypedSlice<I, T>) -> *mut Self::Output {
        // SAFETY: The caller ensures the range is in bounds.
        unsafe { range_inclusive_to_exclusive_unchecked(self).get_unchecked_mut(slice) }
    }

    #[inline]
    fn index(self, slice: &TypedSlice<I, T>) -> &Self::Output {
        let raw_range = self.start().to_raw_index()..=self.end().to_raw_index();
        // SAFETY: index() panics if out of bounds. TypedSlice is repr(transparent) over [T].
        unsafe { TypedSlice::from_slice_unchecked(&slice.raw[raw_range]) }
    }

    #[inline]
    fn index_mut(self, slice: &mut TypedSlice<I, T>) -> &mut Self::Output {
        let raw_range = self.start().to_raw_index()..=self.end().to_raw_index();
        // SAFETY: index_mut() panics if out of bounds. TypedSlice is repr(transparent) over [T].
        unsafe { TypedSlice::from_slice_unchecked_mut(&mut slice.raw[raw_range]) }
    }
}

impl<I: IndexType> private_typed_slice_index::Sealed for core::ops::RangeToInclusive<I> {}
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
        // SAFETY: The caller ensures the range is in bounds.
        unsafe { (I::ZERO..=self.end).get_unchecked(slice) }
    }

    #[inline]
    unsafe fn get_unchecked_mut(self, slice: *mut TypedSlice<I, T>) -> *mut TypedSlice<I, T> {
        // SAFETY: The caller ensures the range is in bounds.
        unsafe { (I::ZERO..=self.end).get_unchecked_mut(slice) }
    }

    #[inline]
    fn index(self, slice: &TypedSlice<I, T>) -> &TypedSlice<I, T> {
        (I::ZERO..=self.end).index(slice)
    }

    #[inline]
    fn index_mut(self, slice: &mut TypedSlice<I, T>) -> &mut TypedSlice<I, T> {
        (I::ZERO..=self.end).index_mut(slice)
    }
}
