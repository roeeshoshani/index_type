//! A fixed-size array with typed indexing.
//!
//! This module provides [`TypedArray`], a wrapper around `[T; N]` that uses a custom
//! [`IndexType`] for all indexing operations. Unlike `TypedSlice`, `TypedArray` has a
//! fixed size known at compile time.
//!
//! # Compile-Time Bounds Checking
//!
//! `TypedArray` performs compile-time checks to ensure that the array length `N`
//! fits within the bounds of the index type `I`. If `N > I::MAX_RAW_INDEX`, the
//! code will not compile.
//!
//! # Example
//!
//! ```
//! use index_type::IndexType;
//! use index_type::typed_array::TypedArray;
//!
//! #[derive(IndexType, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
//! struct PixelIdx(u8);
//!
//! let mut pixels: TypedArray<PixelIdx, [u8; 3], 4> = TypedArray::default();
//! pixels[PixelIdx::ZERO] = [255, 0, 0];  // Red pixel
//! pixels[PixelIdx(1)] = [0, 255, 0];     // Green pixel
//!
//! assert_eq!(pixels[PixelIdx::ZERO], [255, 0, 0]);
//! ```

use core::{
    array::TryFromSliceError,
    marker::PhantomData,
    ops::{Index, IndexMut},
};

use crate::{
    IndexType,
    typed_enumerate::UncheckedTypedEnumerate,
    typed_range_iter::{TypedRangeIter, TypedRangeIterExt},
    typed_slice::TypedSlice,
};

/// An array wrapper that uses a custom index type.
#[repr(transparent)]
pub struct TypedArray<I: IndexType, T, const N: usize> {
    raw: [T; N],
    phantom: PhantomData<fn(&I)>,
}

impl<I: IndexType, T, const N: usize> TypedArray<I, T, N> {
    // A compile time assertion to make sure that the array length `N` fits within the bounds of the index type `I`.
    // Used to emit compile time errors instead of runtime erros when we know at compile time that the array size is too big to fit in
    // the index type `I`.
    const _ASSERT_ARRAY_LENGTH_IN_INDEX_BOUNDS: () = if N > I::MAX_RAW_INDEX {
        panic!("array length is not in bounds of the index type");
    };

    /// Returns the length of the array as an index.
    #[inline]
    pub fn len(&self) -> I {
        // SAFETY: The length N is guaranteed to be valid for I by the type system.
        unsafe { I::from_raw_index_unchecked(N) }
    }

    /// Returns the length of the array as a `usize`.
    #[inline]
    pub const fn len_usize(&self) -> usize {
        N
    }

    /// Returns an iterator over the valid indices of this array.
    #[inline]
    pub fn indices(&self) -> TypedRangeIter<I> {
        (I::ZERO..self.len()).iter()
    }

    /// Returns an iterator over the elements with their indices.
    #[inline]
    pub fn iter_enumerated(&self) -> UncheckedTypedEnumerate<I, core::slice::Iter<'_, T>> {
        // SAFETY: `self.raw.iter()` yields exactly `N` items, and `N` is guaranteed to fit in `I`.
        unsafe { UncheckedTypedEnumerate::new(self.raw.iter()) }
    }

    /// Returns an iterator over the elements with their mutable references and indices.
    #[inline]
    pub fn iter_mut_enumerated(
        &mut self,
    ) -> UncheckedTypedEnumerate<I, core::slice::IterMut<'_, T>> {
        // SAFETY: `self.raw.iter_mut()` yields exactly `N` items, and `N` is guaranteed to fit in `I`.
        unsafe { UncheckedTypedEnumerate::new(self.raw.iter_mut()) }
    }

    /// Consumes the array and returns an iterator over the elements with their indices.
    #[inline]
    pub fn into_iter_enumerated(self) -> UncheckedTypedEnumerate<I, core::array::IntoIter<T, N>> {
        // SAFETY: `self.raw.into_iter()` yields exactly `N` items, and `N` is guaranteed to fit in `I`.
        unsafe { UncheckedTypedEnumerate::new(self.raw.into_iter()) }
    }

    #[inline]
    pub const fn as_slice(&self) -> &TypedSlice<I, T> {
        // SAFETY: The length of the array is guaranteed to be valid for I by the type system.
        unsafe { TypedSlice::from_slice_unchecked(&self.raw) }
    }

    #[inline]
    pub const fn as_mut_slice(&mut self) -> &mut TypedSlice<I, T> {
        // SAFETY: The length of the array is guaranteed to be valid for I by the type system.
        unsafe { TypedSlice::from_slice_unchecked_mut(&mut self.raw) }
    }

    /// Casts the index type of the `TypedArray`.
    #[inline]
    pub fn cast_index_type<I2: IndexType>(
        self,
    ) -> Result<TypedArray<I2, T, N>, I2::IndexTooBigError> {
        if I::MAX_RAW_INDEX <= I2::MAX_RAW_INDEX {
            Ok(unsafe { TypedArray::from_array_unchecked(self.raw) })
        } else {
            TypedArray::try_from_array(self.raw)
        }
    }

    #[inline]
    pub const fn as_array(&self) -> &[T; N] {
        &self.raw
    }

    #[inline]
    pub const fn as_mut_array(&mut self) -> &mut [T; N] {
        &mut self.raw
    }

    #[inline]
    pub fn into_array(self) -> [T; N] {
        self.raw
    }

    /// Tries to create a `TypedArray` from a raw array while checking that N is in bounds for I at runtime.
    #[inline]
    pub fn try_from_array(array: [T; N]) -> Result<Self, I::IndexTooBigError> {
        let _ = I::try_from_raw_index(N)?;
        Ok(unsafe { Self::from_array_unchecked(array) })
    }

    /// Tries to create a `TypedArray` reference from a raw array reference while checking that N is in bounds for I at runtime.
    #[inline]
    pub fn try_from_array_ref(array: &[T; N]) -> Result<&TypedArray<I, T, N>, I::IndexTooBigError> {
        let _ = I::try_from_raw_index(N)?;
        Ok(unsafe { Self::from_array_ref_unchecked(array) })
    }

    /// Tries to create a mutable `TypedArray` reference from a mutable raw array reference while checking that N is in bounds for I at
    /// runtime.
    #[inline]
    pub fn try_from_array_mut(
        array: &mut [T; N],
    ) -> Result<&mut TypedArray<I, T, N>, I::IndexTooBigError> {
        let _ = I::try_from_raw_index(N)?;
        Ok(unsafe { Self::from_array_mut_unchecked(array) })
    }

    /// Creates a `TypedArray` from a raw array without checking if N is in bounds for I.
    ///
    /// # Safety
    ///
    /// N must be less than or equal to `I::MAX_RAW_INDEX`.
    #[inline]
    pub const unsafe fn from_array_unchecked(array: [T; N]) -> Self {
        TypedArray {
            raw: array,
            phantom: PhantomData,
        }
    }

    /// Creates a `TypedArray` reference from a raw array reference without checking if N is in bounds for I.
    ///
    /// # Safety
    ///
    /// N must be less than or equal to `I::MAX_RAW_INDEX`.
    #[inline]
    pub const unsafe fn from_array_ref_unchecked(array: &[T; N]) -> &TypedArray<I, T, N> {
        // SAFETY: TypedArray is repr(transparent) over [T; N].
        unsafe { core::mem::transmute::<&[T; N], &TypedArray<I, T, N>>(array) }
    }

    /// Creates a mutable `TypedArray` reference from a mutable raw array reference without checking if N is in bounds for I.
    ///
    /// # Safety
    ///
    /// N must be less than or equal to `I::MAX_RAW_INDEX`.
    #[inline]
    pub const unsafe fn from_array_mut_unchecked(array: &mut [T; N]) -> &mut TypedArray<I, T, N> {
        // SAFETY: TypedArray is repr(transparent) over [T; N].
        unsafe { core::mem::transmute::<&mut [T; N], &mut TypedArray<I, T, N>>(array) }
    }

    /// Creates a `TypedArray` from a raw array while checking that N is in bounds for I at compile time.
    #[inline]
    pub const fn from_array(array: [T; N]) -> Self {
        const { Self::_ASSERT_ARRAY_LENGTH_IN_INDEX_BOUNDS };
        unsafe { Self::from_array_unchecked(array) }
    }

    /// Creates a `TypedArray` reference from a raw array reference while checking that N is in bounds for I at compile time.
    #[inline]
    pub const fn from_array_ref(array: &[T; N]) -> &TypedArray<I, T, N> {
        const { Self::_ASSERT_ARRAY_LENGTH_IN_INDEX_BOUNDS };
        unsafe { Self::from_array_ref_unchecked(array) }
    }

    /// Creates a mutable `TypedArray` reference from a mutable raw array reference while checking that N is in bounds for I at
    /// compile time.
    #[inline]
    pub const fn from_array_mut(array: &mut [T; N]) -> &mut TypedArray<I, T, N> {
        const { Self::_ASSERT_ARRAY_LENGTH_IN_INDEX_BOUNDS };
        unsafe { Self::from_array_mut_unchecked(array) }
    }

    #[inline]
    pub const fn each_ref(&self) -> TypedArray<I, &T, N> {
        let refs = self.raw.each_ref();
        // SAFETY: The length N is already known to be valid for I.
        unsafe { TypedArray::from_array_unchecked(refs) }
    }

    #[inline]
    pub fn each_mut(&mut self) -> TypedArray<I, &mut T, N> {
        let refs = self.raw.each_mut();
        // SAFETY: The length N is already known to be valid for I.
        unsafe { TypedArray::from_array_unchecked(refs) }
    }

    #[inline]
    pub fn map<U, F: FnMut(T) -> U>(self, f: F) -> TypedArray<I, U, N> {
        // SAFETY: The length N is already known to be valid for I.
        unsafe { TypedArray::from_array_unchecked(self.raw.map(f)) }
    }
}

impl<I: IndexType, T: PartialEq, const N: usize> PartialEq for TypedArray<I, T, N> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        PartialEq::eq(&self.raw, &other.raw)
    }
}

impl<I: IndexType, T: Eq, const N: usize> Eq for TypedArray<I, T, N> {}

impl<I: IndexType, T: PartialOrd, const N: usize> PartialOrd for TypedArray<I, T, N> {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        self.raw.partial_cmp(&other.raw)
    }
}

impl<I: IndexType, T: Ord, const N: usize> Ord for TypedArray<I, T, N> {
    #[inline]
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.raw.cmp(&other.raw)
    }
}

impl<I: IndexType, T: core::hash::Hash, const N: usize> core::hash::Hash for TypedArray<I, T, N> {
    #[inline]
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.raw.hash(state);
    }
}

impl<I: IndexType, T: core::fmt::Debug, const N: usize> core::fmt::Debug for TypedArray<I, T, N> {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::Debug::fmt(&self.raw, f)
    }
}

impl<I: IndexType, T: Clone, const N: usize> Clone for TypedArray<I, T, N> {
    #[inline]
    fn clone(&self) -> Self {
        unsafe { Self::from_array_unchecked(self.raw.clone()) }
    }

    #[inline]
    fn clone_from(&mut self, source: &Self) {
        self.raw.clone_from(source.as_array())
    }
}

impl<I: IndexType, T: Default, const N: usize> Default for TypedArray<I, T, N> {
    #[inline]
    fn default() -> Self {
        TypedArray {
            raw: core::array::from_fn(|_| T::default()),
            phantom: PhantomData,
        }
    }
}

impl<I: IndexType, T, const N: usize, X> Index<X> for TypedArray<I, T, N>
where
    TypedSlice<I, T>: Index<X>,
{
    type Output = <TypedSlice<I, T> as Index<X>>::Output;

    #[inline]
    fn index(&self, index: X) -> &Self::Output {
        self.as_slice().index(index)
    }
}

impl<I: IndexType, T, const N: usize, X> IndexMut<X> for TypedArray<I, T, N>
where
    TypedSlice<I, T>: IndexMut<X>,
{
    #[inline]
    fn index_mut(&mut self, index: X) -> &mut Self::Output {
        self.as_mut_slice().index_mut(index)
    }
}

impl<I: IndexType, T, const N: usize> AsRef<TypedSlice<I, T>> for TypedArray<I, T, N> {
    #[inline]
    fn as_ref(&self) -> &TypedSlice<I, T> {
        // SAFETY: The length N is already known to be valid for I.
        unsafe { TypedSlice::from_slice_unchecked(&self.raw) }
    }
}

impl<I: IndexType, T, const N: usize> AsMut<TypedSlice<I, T>> for TypedArray<I, T, N> {
    #[inline]
    fn as_mut(&mut self) -> &mut TypedSlice<I, T> {
        // SAFETY: The length N is already known to be valid for I.
        unsafe { TypedSlice::from_slice_unchecked_mut(&mut self.raw) }
    }
}

impl<I: IndexType, T, const N: usize> core::borrow::Borrow<TypedSlice<I, T>>
    for TypedArray<I, T, N>
{
    #[inline]
    fn borrow(&self) -> &TypedSlice<I, T> {
        // SAFETY: The length N is already known to be valid for I.
        unsafe { TypedSlice::from_slice_unchecked(&self.raw) }
    }
}

impl<I: IndexType, T, const N: usize> core::borrow::BorrowMut<TypedSlice<I, T>>
    for TypedArray<I, T, N>
{
    #[inline]
    fn borrow_mut(&mut self) -> &mut TypedSlice<I, T> {
        // SAFETY: The length N is already known to be valid for I.
        unsafe { TypedSlice::from_slice_unchecked_mut(&mut self.raw) }
    }
}

impl<'a, I: IndexType, T, const N: usize> IntoIterator for &'a TypedArray<I, T, N> {
    type Item = &'a T;
    type IntoIter = core::slice::Iter<'a, T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.raw.iter()
    }
}

impl<'a, I: IndexType, T, const N: usize> IntoIterator for &'a mut TypedArray<I, T, N> {
    type Item = &'a mut T;
    type IntoIter = core::slice::IterMut<'a, T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.raw.iter_mut()
    }
}

impl<I: IndexType, T, const N: usize> IntoIterator for TypedArray<I, T, N> {
    type Item = T;
    type IntoIter = core::array::IntoIter<T, N>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.raw.into_iter()
    }
}

impl<'a, I: IndexType, T, const N: usize> TryFrom<&'a TypedSlice<I, T>>
    for &'a TypedArray<I, T, N>
{
    type Error = TryFromSliceError;

    #[inline]
    fn try_from(slice: &'a TypedSlice<I, T>) -> Result<&'a TypedArray<I, T, N>, TryFromSliceError> {
        // SAFETY: The length is checked by try_into() and the original slice already has a valid I.
        Ok(unsafe { TypedArray::from_array_ref_unchecked(slice.as_slice().try_into()?) })
    }
}

impl<'a, I: IndexType, T, const N: usize> TryFrom<&'a mut TypedSlice<I, T>>
    for &'a mut TypedArray<I, T, N>
{
    type Error = TryFromSliceError;

    #[inline]
    fn try_from(
        slice: &'a mut TypedSlice<I, T>,
    ) -> Result<&'a mut TypedArray<I, T, N>, TryFromSliceError> {
        // SAFETY: The length is checked by try_into() and the original slice already has a valid I.
        Ok(unsafe { TypedArray::from_array_mut_unchecked(slice.as_mut_slice().try_into()?) })
    }
}

impl<'a, I: IndexType, T: Copy, const N: usize> TryFrom<&'a TypedSlice<I, T>>
    for TypedArray<I, T, N>
{
    type Error = TryFromSliceError;

    #[inline]
    fn try_from(slice: &'a TypedSlice<I, T>) -> Result<TypedArray<I, T, N>, TryFromSliceError> {
        // SAFETY: The length is checked by try_into() and the original slice already has a valid I.
        Ok(unsafe { TypedArray::from_array_unchecked(slice.as_slice().try_into()?) })
    }
}

impl<'a, I: IndexType, T: Copy, const N: usize> TryFrom<&'a mut TypedSlice<I, T>>
    for TypedArray<I, T, N>
{
    type Error = TryFromSliceError;

    #[inline]
    fn try_from(slice: &'a mut TypedSlice<I, T>) -> Result<TypedArray<I, T, N>, TryFromSliceError> {
        // SAFETY: The length is checked by try_into() and the original slice already has a valid I.
        Ok(unsafe { TypedArray::from_array_unchecked(slice.as_mut_slice().try_into()?) })
    }
}

impl<I: IndexType, T, const N: usize> core::ops::Deref for TypedArray<I, T, N> {
    type Target = TypedSlice<I, T>;

    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

impl<I: IndexType, T, const N: usize> core::ops::DerefMut for TypedArray<I, T, N> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut_slice()
    }
}
