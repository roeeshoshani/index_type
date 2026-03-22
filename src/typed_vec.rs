//! A growable, typed vector.
//!
//! This module provides [`TypedVec`], which is a wrapper around [`Vec`] that uses a custom
//! [`IndexType`](crate::IndexType) for indexing.

use core::{
    borrow::{Borrow, BorrowMut},
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

use alloc::{boxed::Box, collections::TryReserveError, vec::Vec};

use crate::{
    IndexScalarType, IndexTooBigError, IndexType, typed_slice::TypedSlice,
    utils::range_bounds_to_raw,
};

/// A `Vec` wrapper that uses a custom index type.
#[repr(transparent)]
pub struct TypedVec<I: IndexType, T> {
    raw: Vec<T>,
    phantom: PhantomData<fn(&I)>,
}
impl<I: IndexType, T> TypedVec<I, T> {
    /// Creates a new, empty `TypedVec`.
    #[inline]
    pub const fn new() -> Self {
        Self {
            raw: Vec::new(),
            phantom: PhantomData,
        }
    }

    /// Creates a new `TypedVec` with the specified capacity.
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            raw: Vec::with_capacity(capacity),
            phantom: PhantomData,
        }
    }

    /// Tries to create a `TypedVec` from a `Vec`.
    ///
    /// Returns an error if the `Vec`'s length exceeds the maximum value representable by `I`.
    #[inline]
    pub fn try_from_vec(vec: Vec<T>) -> Result<Self, I::IndexTooBigError> {
        let _ = I::try_from_raw_index(vec.len())?;
        let res = Self {
            raw: vec,
            phantom: PhantomData,
        };
        Ok(res)
    }

    /// Creates a `TypedVec` from a `Vec` without checking if its length is in bounds.
    ///
    /// # Safety
    ///
    /// The `Vec`'s length must be less than or equal to `I::MAX_RAW_INDEX`.
    #[inline]
    pub unsafe fn from_vec_unchecked(vec: Vec<T>) -> Self {
        Self {
            raw: vec,
            phantom: PhantomData,
        }
    }

    /// Creates a `TypedVec` from raw parts.
    ///
    /// # Safety
    ///
    /// See `Vec::from_raw_parts`.
    #[inline]
    pub unsafe fn from_raw_parts(
        ptr: *mut T,
        length: usize,
        capacity: usize,
    ) -> Result<Self, I::IndexTooBigError> {
        let _ = I::try_from_raw_index(length)?;
        Ok(Self {
            // SAFETY: The caller ensures the pointer is valid and the parts are correct.
            raw: unsafe { Vec::from_raw_parts(ptr, length, capacity) },
            phantom: PhantomData,
        })
    }

    /// Decomposes the `TypedVec` into its raw parts.
    #[inline]
    pub fn into_raw_parts(self) -> (*mut T, usize, usize) {
        self.raw.into_raw_parts()
    }

    /// Converts the `TypedVec` into a `Vec`.
    #[inline]
    pub fn into_vec(self) -> Vec<T> {
        self.raw
    }

    /// Returns the length of the `TypedVec` as a scalar.
    #[inline]
    pub fn len(&self) -> I::Scalar {
        // SAFETY: The length of the vector is guaranteed to be representable by I::Scalar
        // because we check it on creation and on every operation that increases it.
        unsafe { <I::Scalar as IndexScalarType>::from_usize_unchecked(self.raw.len()) }
    }

    /// Returns the length of the `TypedVec` as an index.
    #[inline]
    pub fn len_as_index(&self) -> I {
        // SAFETY: The length of the vector is guaranteed to be representable by I
        // because we check it on creation and on every operation that increases it.
        unsafe { I::from_raw_index_unchecked(self.raw.len()) }
    }

    /// Returns the capacity of the `TypedVec`.
    #[inline]
    pub fn capacity(&self) -> usize {
        self.raw.capacity()
    }

    /// Appends an element to the back of the `TypedVec`.
    ///
    /// This is a wrapper around `Vec::push` that checks if the new length is within bounds for `I`.
    #[inline]
    pub fn push(&mut self, value: T) -> Result<I, I::IndexTooBigError> {
        let res = self.len_as_index();
        let _new_len = res.checked_add_scalar(<I::Scalar as IndexScalarType>::ONE)?;
        self.raw.push(value);
        Ok(res)
    }

    /// Appends all elements from another `TypedVec` to the back of this one.
    ///
    /// This is a wrapper around `Vec::append` that checks if the combined length is within bounds for `I`.
    #[inline]
    pub fn append(&mut self, other: &mut TypedVec<I, T>) -> Result<(), I::IndexTooBigError> {
        let _new_len = self.len_as_index().checked_add_scalar(other.len())?;
        self.raw.append(&mut other.raw);
        Ok(())
    }

    #[inline]
    pub const fn as_mut_ptr(&mut self) -> *mut T {
        self.raw.as_mut_ptr()
    }

    #[inline]
    pub fn reserve(&mut self, additional: usize) {
        self.raw.reserve(additional);
    }

    #[inline]
    pub fn reserve_exact(&mut self, additional: usize) {
        self.raw.reserve_exact(additional)
    }

    #[inline]
    pub fn try_reserve(&mut self, additional: usize) -> Result<(), TryReserveError> {
        self.raw.try_reserve(additional)
    }

    #[inline]
    pub fn try_reserve_exact(&mut self, additional: usize) -> Result<(), TryReserveError> {
        self.raw.try_reserve_exact(additional)
    }

    #[inline]
    pub fn shrink_to_fit(&mut self) {
        self.raw.shrink_to_fit();
    }

    #[inline]
    pub fn shrink_to(&mut self, min_capacity: usize) {
        self.raw.shrink_to(min_capacity);
    }

    #[inline]
    pub fn into_boxed_slice(self) -> Box<TypedSlice<I, T>> {
        // SAFETY: TypedSlice is repr(transparent) over [T].
        unsafe { core::mem::transmute(self.raw.into_boxed_slice()) }
    }

    #[inline]
    pub fn truncate(&mut self, len: I::Scalar) {
        self.raw.truncate(len.to_usize());
    }

    #[inline]
    pub fn as_slice(&self) -> &TypedSlice<I, T> {
        // SAFETY: The length of the slice is guaranteed to be in bounds for I.
        unsafe { TypedSlice::from_slice_unchecked(self.raw.as_slice()) }
    }

    #[inline]
    pub fn as_mut_slice(&mut self) -> &mut TypedSlice<I, T> {
        // SAFETY: The length of the slice is guaranteed to be in bounds for I.
        unsafe { TypedSlice::from_slice_unchecked_mut(self.raw.as_mut_slice()) }
    }

    #[inline]
    pub const fn as_ptr(&self) -> *const T {
        self.raw.as_ptr()
    }

    /// Sets the length of the `TypedVec`.
    ///
    /// # Safety
    ///
    /// See `Vec::set_len`.
    #[inline]
    pub unsafe fn set_len(&mut self, new_len: I) {
        // SAFETY: The caller must ensure that the new length is valid.
        unsafe { self.raw.set_len(new_len.to_scalar().to_usize()) };
    }

    #[inline]
    pub fn swap_remove(&mut self, index: I) -> T {
        self.raw.swap_remove(index.to_raw_index())
    }

    /// Inserts an element at position `index` within the `TypedVec`.
    ///
    /// This is a wrapper around `Vec::insert` that checks if the new length is within bounds for `I`.
    #[inline]
    pub fn insert(&mut self, index: I, element: T) -> Result<(), I::IndexTooBigError> {
        let _new_potential_len = index.checked_add_scalar(<I::Scalar as IndexScalarType>::ONE)?;
        self.raw.insert(index.to_raw_index(), element);
        Ok(())
    }

    #[inline]
    pub fn remove(&mut self, index: I) -> T {
        self.raw.remove(index.to_raw_index())
    }

    #[inline]
    pub fn retain<F>(&mut self, f: F)
    where
        F: FnMut(&T) -> bool,
    {
        self.raw.retain(f)
    }

    #[inline]
    pub fn retain_mut<F>(&mut self, f: F)
    where
        F: FnMut(&mut T) -> bool,
    {
        self.raw.retain_mut(f)
    }

    #[inline]
    pub fn dedup_by_key<F, K>(&mut self, key: F)
    where
        F: FnMut(&mut T) -> K,
        K: PartialEq,
    {
        self.raw.dedup_by_key(key);
    }

    #[inline]
    pub fn dedup_by<F>(&mut self, same_bucket: F)
    where
        F: FnMut(&mut T, &mut T) -> bool,
    {
        self.raw.dedup_by(same_bucket);
    }

    #[inline]
    pub fn pop(&mut self) -> Option<T> {
        self.raw.pop()
    }

    #[inline]
    pub fn pop_if(&mut self, predicate: impl FnOnce(&mut T) -> bool) -> Option<T> {
        self.raw.pop_if(predicate)
    }

    #[inline]
    pub fn clear(&mut self) {
        self.raw.clear();
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.raw.is_empty()
    }

    #[inline]
    pub fn split_off(&mut self, at: I) -> Self {
        let new_vec = self.raw.split_off(at.to_raw_index());
        // SAFETY: The new vector's length must be less than or equal to the original vector's length,
        // which was already guaranteed to be in bounds for I.
        unsafe { Self::from_vec_unchecked(new_vec) }
    }

    #[inline]
    pub fn resize_with<F>(&mut self, new_len: I, f: F)
    where
        F: FnMut() -> T,
    {
        self.raw.resize_with(new_len.to_scalar().to_usize(), f);
    }

    #[inline]
    pub fn leak<'a>(self) -> &'a mut [T] {
        self.raw.leak()
    }

    #[inline]
    pub fn drain<R>(&mut self, range: R) -> alloc::vec::Drain<'_, T>
    where
        R: core::ops::RangeBounds<I>,
    {
        self.raw.drain(range_bounds_to_raw(range))
    }

    #[inline]
    pub fn splice<R, X>(&mut self, range: R, replace_with: X) -> alloc::vec::Splice<'_, X::IntoIter>
    where
        R: core::ops::RangeBounds<I>,
        X: IntoIterator<Item = T>,
    {
        self.raw.splice(range_bounds_to_raw(range), replace_with)
    }

    /// Tries to extend the `TypedVec` with the contents of an iterator.
    ///
    /// This is a wrapper around `Vec::extend` that checks if the new length is within bounds for `I`.
    #[inline]
    pub fn try_extend<X: IntoIterator<Item = T>>(
        &mut self,
        iter: X,
    ) -> Result<(), I::IndexTooBigError> {
        let orig_raw_len = self.raw.len();
        self.raw.extend(iter);
        match I::try_from_raw_index(self.raw.len()) {
            Ok(_) => Ok(()),
            Err(err) => {
                self.raw.truncate(orig_raw_len);
                Err(err)
            }
        }
    }
}
impl<I: IndexType, T: Copy> TypedVec<I, T> {
    #[inline]
    pub fn try_extend_copy<'a, X: IntoIterator<Item = &'a T>>(
        &mut self,
        iter: X,
    ) -> Result<(), I::IndexTooBigError>
    where
        T: 'a,
    {
        let orig_raw_len = self.raw.len();
        self.raw.extend(iter);
        match I::try_from_raw_index(self.raw.len()) {
            Ok(_) => Ok(()),
            Err(err) => {
                self.raw.truncate(orig_raw_len);
                Err(err)
            }
        }
    }
}
impl<I: IndexType, T: PartialEq> TypedVec<I, T> {
    #[inline]
    pub fn dedup(&mut self) {
        self.raw.dedup();
    }
}
impl<I: IndexType, T: Clone> TypedVec<I, T> {
    #[inline]
    pub fn extend_from_slice(&mut self, other: &TypedSlice<I, T>) {
        self.raw.extend_from_slice(other.as_slice())
    }

    #[inline]
    pub fn extend_from_within<R>(&mut self, src: R)
    where
        R: core::ops::RangeBounds<I>,
    {
        self.raw.extend_from_within(range_bounds_to_raw(src));
    }

    #[inline]
    pub fn extract_if<F, R>(&mut self, range: R, filter: F) -> alloc::vec::ExtractIf<'_, T, F>
    where
        F: FnMut(&mut T) -> bool,
        R: core::ops::RangeBounds<I>,
    {
        self.raw.extract_if(range_bounds_to_raw(range), filter)
    }

    #[inline]
    pub fn resize(&mut self, new_len: I, value: T) {
        self.raw.resize(new_len.to_raw_index(), value);
    }
}

impl<I: IndexType, T, const N: usize> TypedVec<I, [T; N]> {
    /// Converts a `TypedVec<I, [T; N]>` into a `TypedVec<I, T>`.
    pub fn into_flattened(self) -> Result<TypedVec<I, T>, I::IndexTooBigError> {
        let _new_len = self.len_as_index().checked_mul_scalar(
            <I::Scalar as IndexScalarType>::try_from_usize(N)
                .ok_or(<I::IndexTooBigError as IndexTooBigError>::new())?,
        )?;
        // SAFETY: We checked that the new length is in bounds for I.
        Ok(unsafe { TypedVec::from_vec_unchecked(self.raw.into_flattened()) })
    }
}
impl<I: IndexType, T: core::fmt::Debug> core::fmt::Debug for TypedVec<I, T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::Debug::fmt(&self.raw, f)
    }
}
impl<I: IndexType, T: PartialEq> PartialEq for TypedVec<I, T> {
    fn eq(&self, other: &Self) -> bool {
        PartialEq::eq(&self.raw, &other.raw)
    }
}
impl<I: IndexType, T: Eq> Eq for TypedVec<I, T> {}
impl<I: IndexType, T: Clone> Clone for TypedVec<I, T> {
    fn clone(&self) -> Self {
        Self {
            raw: Clone::clone(&self.raw),
            phantom: PhantomData,
        }
    }

    fn clone_from(&mut self, source: &Self) {
        Clone::clone_from(&mut self.raw, &source.raw);
    }
}
impl<I: IndexType, T: core::hash::Hash> core::hash::Hash for TypedVec<I, T> {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        core::hash::Hash::hash(&self.raw, state);
    }
}
impl<I: IndexType, T> Default for TypedVec<I, T> {
    fn default() -> Self {
        Self {
            raw: Default::default(),
            phantom: PhantomData,
        }
    }
}
impl<I: IndexType, T> Deref for TypedVec<I, T> {
    type Target = TypedSlice<I, T>;

    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}
impl<I: IndexType, T> DerefMut for TypedVec<I, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut_slice()
    }
}
impl<I: IndexType, T> AsRef<TypedSlice<I, T>> for TypedVec<I, T> {
    fn as_ref(&self) -> &TypedSlice<I, T> {
        self.as_slice()
    }
}
impl<I: IndexType, T> AsMut<TypedSlice<I, T>> for TypedVec<I, T> {
    fn as_mut(&mut self) -> &mut TypedSlice<I, T> {
        self.as_mut_slice()
    }
}
impl<I: IndexType, T> AsRef<TypedVec<I, T>> for TypedVec<I, T> {
    fn as_ref(&self) -> &TypedVec<I, T> {
        self
    }
}
impl<I: IndexType, T> AsMut<TypedVec<I, T>> for TypedVec<I, T> {
    fn as_mut(&mut self) -> &mut TypedVec<I, T> {
        self
    }
}
impl<I: IndexType, T> Borrow<TypedSlice<I, T>> for TypedVec<I, T> {
    fn borrow(&self) -> &TypedSlice<I, T> {
        self.as_slice()
    }
}
impl<I: IndexType, T> BorrowMut<TypedSlice<I, T>> for TypedVec<I, T> {
    fn borrow_mut(&mut self) -> &mut TypedSlice<I, T> {
        self.as_mut_slice()
    }
}
impl<'a, I: IndexType, T: Clone> From<&'a TypedSlice<I, T>> for TypedVec<I, T> {
    fn from(value: &'a TypedSlice<I, T>) -> Self {
        // SAFETY: The length of the slice is already guaranteed to be in bounds for I.
        unsafe { Self::from_vec_unchecked(Vec::from(value.as_slice())) }
    }
}
impl<I: IndexType, T: Clone> IntoIterator for TypedVec<I, T> {
    type Item = T;

    type IntoIter = alloc::vec::IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        self.raw.into_iter()
    }
}
impl<'a, I: IndexType, T: Clone> IntoIterator for &'a TypedVec<I, T> {
    type Item = &'a T;

    type IntoIter = core::slice::Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        (&self.raw).into_iter()
    }
}
impl<'a, I: IndexType, T: Clone> IntoIterator for &'a mut TypedVec<I, T> {
    type Item = &'a mut T;

    type IntoIter = core::slice::IterMut<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        (&mut self.raw).into_iter()
    }
}
impl<I: IndexType, T: PartialEq> PartialEq<TypedSlice<I, T>> for TypedVec<I, T> {
    fn eq(&self, other: &TypedSlice<I, T>) -> bool {
        PartialEq::eq(&self.raw, other.as_slice())
    }
}
impl<'a, I: IndexType, T: PartialEq> PartialEq<&'a TypedSlice<I, T>> for TypedVec<I, T> {
    fn eq(&self, other: &&'a TypedSlice<I, T>) -> bool {
        PartialEq::eq(&self.raw, other.as_slice())
    }
}
impl<'a, I: IndexType, T: PartialEq> PartialEq<&'a mut TypedSlice<I, T>> for TypedVec<I, T> {
    fn eq(&self, other: &&'a mut TypedSlice<I, T>) -> bool {
        PartialEq::eq(&self.raw, other.as_slice())
    }
}
impl<I: IndexType, T: PartialOrd> PartialOrd for TypedVec<I, T> {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        PartialOrd::partial_cmp(&self.raw, &other.raw)
    }
}
impl<I: IndexType, T: Ord> Ord for TypedVec<I, T> {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        Ord::cmp(&self.raw, &other.raw)
    }
}
