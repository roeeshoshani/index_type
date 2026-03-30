//! A growable vector with typed indexing.
//!
//! This module provides [`TypedVec`], a wrapper around [`alloc::vec::Vec`] that uses a custom
//! [`IndexType`] for all indexing operations. This provides compile-time guarantees that
//! indices from one `TypedVec` cannot be accidentally used with another.
//!
//! # Example
//!
//! ```
//! use index_type::IndexType;
//! use index_type::typed_vec::TypedVec;
//!
//! #[derive(IndexType, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
//! struct NodeId(u32);
//!
//! let mut nodes: TypedVec<NodeId, String> = TypedVec::new();
//! let id0 = nodes.push("Alice".to_string());
//! let id1 = nodes.push("Bob".to_string());
//!
//! assert_eq!(nodes[id0], "Alice");
//! assert_eq!(nodes[id1], "Bob");
//! ```
//!
//! # Capacity and Growth
//!
//! `TypedVec` has the same growth behavior as [`Vec`]. Operations that would cause the length
//! to exceed `I::MAX_RAW_INDEX` return an error or panic, depending on whether you use
//! the fallible or infallible variant.

use core::{
    borrow::{Borrow, BorrowMut},
    iter::FusedIterator,
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

use alloc::{boxed::Box, collections::TryReserveError, vec::Vec};

use crate::{
    typed_enumerate::UncheckedTypedEnumerate,
    typed_range_iter::{TypedRangeIter, TypedRangeIterExt},
    typed_slice::TypedSlice,
    utils::{range_bounds_to_raw, resolve_range_bounds},
    IndexScalarType, IndexTooBigError, IndexType,
};

#[cold]
fn panic_index_too_big<I: IndexType>(error: I::IndexTooBigError) -> ! {
    panic!("{}", error)
}

/// A growable vector with typed indexing.
///
/// `TypedVec<I, T>` is a wrapper around `Vec<T>` that uses the custom index type `I`
/// for all indexing operations. This provides compile-time guarantees that indices
/// cannot be accidentally used with the wrong collection.
///
/// # Type Parameters
///
/// - `I`: The index type that implements [`IndexType`]
/// - `T`: The element type stored in the vector
///
/// # Example
///
/// ```
/// use index_type::IndexType;
/// use index_type::typed_vec::TypedVec;
///
/// #[derive(IndexType, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
/// struct RowId(u32);
///
/// let mut rows: TypedVec<RowId, String> = TypedVec::new();
/// let id0 = rows.push("Row 0".to_string());
/// let id1 = rows.push("Row 1".to_string());
///
/// assert_eq!(rows[id0], "Row 0");
/// ```
#[repr(transparent)]
pub struct TypedVec<I: IndexType, T> {
    raw: Vec<T>,
    phantom: PhantomData<fn(&I)>,
}

impl<I: IndexType, T> TypedVec<I, T> {
    /// Creates a new, empty `TypedVec`.
    ///
    /// The vector will not allocate until elements are pushed.
    ///
    /// # Example
    ///
    /// ```
    /// use index_type::IndexType;
    /// use index_type::typed_vec::TypedVec;
    ///
    /// #[derive(IndexType, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    /// struct Idx(u32);
    ///
    /// let vec: TypedVec<Idx, i32> = TypedVec::new();
    /// assert!(vec.is_empty());
    /// ```
    #[inline]
    pub const fn new() -> Self {
        Self {
            raw: Vec::new(),
            phantom: PhantomData,
        }
    }

    /// Creates a new `TypedVec` with the specified capacity.
    ///
    /// The vector will be able to hold at least `capacity` elements without reallocating.
    ///
    /// # Example
    ///
    /// ```
    /// use index_type::IndexType;
    /// use index_type::typed_vec::TypedVec;
    ///
    /// #[derive(IndexType, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    /// struct Idx(u32);
    ///
    /// let vec: TypedVec<Idx, i32> = TypedVec::with_capacity(10);
    /// assert!(vec.capacity() >= 10);
    /// ```
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            raw: Vec::with_capacity(capacity),
            phantom: PhantomData,
        }
    }

    /// Attempts to create a `TypedVec` from a `Vec`.
    ///
    /// Returns an error if the `Vec`'s length exceeds `I::MAX_RAW_INDEX`.
    ///
    /// # Example
    ///
    /// ```
    /// use index_type::IndexType;
    /// use index_type::typed_vec::TypedVec;
    ///
    /// #[derive(IndexType, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    /// struct Idx(u8);
    ///
    /// let std_vec: Vec<i32> = vec![1, 2, 3];
    /// let typed: Result<TypedVec<Idx, i32>, _> = TypedVec::try_from_vec(std_vec);
    /// assert!(typed.is_ok());
    /// ```
    #[inline]
    pub fn try_from_vec(vec: Vec<T>) -> Result<Self, I::IndexTooBigError> {
        let _ = I::try_from_raw_index(vec.len())?;
        let res = Self {
            raw: vec,
            phantom: PhantomData,
        };
        Ok(res)
    }

    /// Creates a `TypedVec` from a `Vec`.
    ///
    /// # Panics
    ///
    /// Panics if the `Vec`'s length exceeds `I::MAX_RAW_INDEX`.
    ///
    /// # Example
    ///
    /// ```
    /// use index_type::IndexType;
    /// use index_type::typed_vec::TypedVec;
    ///
    /// #[derive(IndexType, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    /// struct Idx(u32);
    ///
    /// let std_vec = vec![1, 2, 3];
    /// let typed: TypedVec<Idx, i32> = TypedVec::from_vec(std_vec);
    /// assert_eq!(typed.len().to_raw_index(), 3);
    /// ```
    #[inline]
    pub fn from_vec(vec: Vec<T>) -> Self {
        Self::try_from_vec(vec).unwrap_or_else(|error| panic_index_too_big::<I>(error))
    }

    /// Creates a `TypedVec` from a `Vec` without checking bounds.
    ///
    /// # Safety
    ///
    /// The `Vec`'s length must not exceed `I::MAX_RAW_INDEX`.
    #[inline]
    pub unsafe fn from_vec_unchecked(vec: Vec<T>) -> Self {
        Self {
            raw: vec,
            phantom: PhantomData,
        }
    }

    /// Attempts to create a `TypedVec` from raw parts.
    ///
    /// # Safety
    ///
    /// Same as [`Vec::from_raw_parts`], plus the length must not exceed `I::MAX_RAW_INDEX`.
    #[inline]
    pub unsafe fn try_from_raw_parts(
        ptr: *mut T,
        length: usize,
        capacity: usize,
    ) -> Result<Self, I::IndexTooBigError> {
        let _ = I::try_from_raw_index(length)?;
        Ok(unsafe { Self::from_raw_parts_unchecked(ptr, length, capacity) })
    }

    /// Creates a `TypedVec` from raw parts without checking bounds.
    ///
    /// # Safety
    ///
    /// Same as [`Vec::from_raw_parts`], plus the length must not exceed `I::MAX_RAW_INDEX`.
    #[inline]
    pub unsafe fn from_raw_parts_unchecked(ptr: *mut T, length: usize, capacity: usize) -> Self {
        Self {
            raw: unsafe { Vec::from_raw_parts(ptr, length, capacity) },
            phantom: PhantomData,
        }
    }

    /// Creates a `TypedVec` from raw parts.
    ///
    /// # Safety
    ///
    /// Same as [`Vec::from_raw_parts`].
    #[inline]
    pub unsafe fn from_raw_parts(ptr: *mut T, length: I, capacity: usize) -> Self {
        unsafe { Self::from_raw_parts_unchecked(ptr, length.to_raw_index(), capacity) }
    }

    /// Decomposes the `TypedVec` into its raw parts.
    ///
    /// Returns the pointer, length, and capacity of the underlying `Vec`.
    #[inline]
    pub fn into_raw_parts(self) -> (*mut T, usize, usize) {
        self.raw.into_raw_parts()
    }

    /// Converts the `TypedVec` into a `Vec`.
    #[inline]
    pub fn into_vec(self) -> Vec<T> {
        self.raw
    }

    /// Returns the length of the vector as an index.
    #[inline]
    pub fn len(&self) -> I {
        unsafe { I::from_raw_index_unchecked(self.raw.len()) }
    }

    /// Returns the length of the vector as a `usize`.
    #[inline]
    pub const fn len_usize(&self) -> usize {
        self.raw.len()
    }

    /// Returns the total capacity of the vector (in elements).
    ///
    /// Note: This returns the raw `usize` capacity, not the typed capacity.
    ///
    /// # Design Decision
    ///
    /// Unlike [`len()`](Self::len) which returns a typed index `I`, this method returns a raw
    /// `usize`. This is an intentional design choice: `TypedVec` may have a capacity that exceeds
    /// what the index type `I` can represent.
    ///
    /// If we limited capacity to `I::MAX_RAW_INDEX`, strange behaviors would occur. For example,
    /// with a `u8` index type (max 255), consider this scenario:
    /// - Vector starts with capacity 100
    /// - After some pushes, it reallocates and doubles to capacity 200
    /// - The next push (201st element) would require reallocation to capacity 400, which exceeds
    ///   `u8::MAX_RAW_INDEX` (255), so this push would fail
    ///
    /// This would be surprising: a vector with a `u8` index could suddenly fail to push even though
    /// it should be able to hold up to 255 elements. By allowing capacity to exceed the index
    /// type's range, we ensure that the vector can always grow to accommodate up to 255 elements,
    /// even if it temporarily has excess capacity.
    ///
    /// Use [`len()`](Self::len) when you need the typed length, and [`remaining_capacity`](Self::remaining_capacity)
    /// when you need to know how many more elements can be added before reaching the index limit.
    #[inline]
    pub fn capacity(&self) -> usize {
        self.raw.capacity()
    }

    /// Returns the remaining capacity until the vector would exceed the index type's limit.
    ///
    /// This is the maximum number of additional elements that can be pushed before the
    /// index type's maximum would be exceeded, regardless of the underlying allocation.
    ///
    /// # Example
    ///
    /// ```
    /// use index_type::IndexType;
    /// use index_type::typed_vec::TypedVec;
    ///
    /// #[derive(IndexType, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    /// struct Idx(u8);
    ///
    /// let vec: TypedVec<Idx, i32> = TypedVec::with_capacity(10);
    /// // remaining_capacity is based on index type, not allocation
    /// assert_eq!(vec.remaining_capacity().to_raw_index(), 255);
    /// ```
    #[inline]
    pub fn remaining_capacity(&self) -> I {
        // This is safe because remaining capacity is always within I's range
        unsafe {
            I::from_raw_index_unchecked(I::MAX_RAW_INDEX.saturating_sub(self.len().to_raw_index()))
        }
    }

    /// Returns an iterator over the valid indices of this vector.
    ///
    /// # Example
    ///
    /// ```
    /// use index_type::IndexType;
    /// use index_type::typed_vec::TypedVec;
    ///
    /// #[derive(IndexType, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    /// struct Idx(u32);
    ///
    /// let vec: TypedVec<Idx, i32> = TypedVec::from_vec(vec![10, 20, 30]);
    /// for idx in vec.indices() {
    ///     println!("{}: {}", idx.to_raw_index(), vec[idx]);
    /// }
    /// ```
    #[inline]
    pub fn indices(&self) -> TypedRangeIter<I> {
        (I::ZERO..self.len()).iter()
    }

    /// Returns an iterator over the elements with their indices.
    #[inline]
    pub fn iter_enumerated(&self) -> UncheckedTypedEnumerate<I, core::slice::Iter<'_, T>> {
        // SAFETY: `self.raw.iter()` yields exactly `self.len()` items, which already fit in `I`.
        unsafe { UncheckedTypedEnumerate::new(self.raw.iter()) }
    }

    /// Returns an iterator over the elements with their mutable references and indices.
    #[inline]
    pub fn iter_mut_enumerated(
        &mut self,
    ) -> UncheckedTypedEnumerate<I, core::slice::IterMut<'_, T>> {
        // SAFETY: `self.raw.iter_mut()` yields exactly `self.len()` items, which already fit in `I`.
        unsafe { UncheckedTypedEnumerate::new(self.raw.iter_mut()) }
    }

    /// Consumes the vector and returns an iterator over the elements with their indices.
    #[inline]
    pub fn into_iter_enumerated(self) -> UncheckedTypedEnumerate<I, alloc::vec::IntoIter<T>> {
        // SAFETY: `self.raw.into_iter()` yields exactly the vector length, which already fits in `I`.
        unsafe { UncheckedTypedEnumerate::new(self.raw.into_iter()) }
    }

    /// Attempts to append an element to the back of the vector.
    ///
    /// Returns the index of the appended element, or an error if the length
    /// would exceed `I::MAX_RAW_INDEX`.
    ///
    /// # Example
    ///
    /// ```
    /// use index_type::IndexType;
    /// use index_type::typed_vec::TypedVec;
    ///
    /// #[derive(IndexType, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    /// struct Idx(u32);
    ///
    /// let mut vec: TypedVec<Idx, i32> = TypedVec::new();
    /// let idx = vec.try_push(42).unwrap();
    /// assert_eq!(vec[idx], 42);
    /// ```
    #[inline]
    pub fn try_push(&mut self, value: T) -> Result<I, I::IndexTooBigError> {
        let res = self.len();
        let _new_len = res.checked_add_scalar(I::Scalar::ONE)?;
        self.raw.push(value);
        Ok(res)
    }

    /// Appends an element to the back of the vector.
    ///
    /// Returns the index of the appended element.
    ///
    /// # Panics
    ///
    /// Panics if the length would exceed `I::MAX_RAW_INDEX`.
    #[inline]
    pub fn push(&mut self, value: T) -> I {
        self.try_push(value)
            .unwrap_or_else(|error| panic_index_too_big::<I>(error))
    }

    /// Attempts to append all elements from another `TypedVec` to this one.
    ///
    /// Returns an error if the combined length would exceed `I::MAX_RAW_INDEX`.
    /// The source vector is emptied after the operation.
    #[inline]
    pub fn try_append(&mut self, other: &mut TypedVec<I, T>) -> Result<(), I::IndexTooBigError> {
        let _new_len = self.len().checked_add_scalar(other.len().to_scalar())?;
        self.raw.append(&mut other.raw);
        Ok(())
    }

    /// Appends all elements from another `TypedVec` to this one.
    ///
    /// The source vector is emptied after the operation.
    ///
    /// # Panics
    ///
    /// Panics if the combined length would exceed `I::MAX_RAW_INDEX`.
    #[inline]
    pub fn append(&mut self, other: &mut TypedVec<I, T>) {
        self.try_append(other)
            .unwrap_or_else(|error| panic_index_too_big::<I>(error))
    }

    /// Returns a raw pointer to the vector's buffer.
    #[inline]
    pub const fn as_mut_ptr(&mut self) -> *mut T {
        self.raw.as_mut_ptr()
    }

    /// Reserves capacity for at least `additional` more elements.
    ///
    /// See [`Vec::reserve`] for details.
    #[inline]
    pub fn reserve(&mut self, additional: usize) {
        self.raw.reserve(additional);
    }

    /// Reserves the exact capacity for `additional` more elements.
    ///
    /// See [`Vec::reserve_exact`] for details.
    #[inline]
    pub fn reserve_exact(&mut self, additional: usize) {
        self.raw.reserve_exact(additional)
    }

    /// Attempts to reserve capacity for at least `additional` more elements.
    ///
    /// See [`Vec::try_reserve`] for details.
    #[inline]
    pub fn try_reserve(&mut self, additional: usize) -> Result<(), TryReserveError> {
        self.raw.try_reserve(additional)
    }

    /// Attempts to reserve the exact capacity for `additional` more elements.
    ///
    /// See [`Vec::try_reserve_exact`] for details.
    #[inline]
    pub fn try_reserve_exact(&mut self, additional: usize) -> Result<(), TryReserveError> {
        self.raw.try_reserve_exact(additional)
    }

    /// Reduces the capacity to fit the current length.
    ///
    /// See [`Vec::shrink_to_fit`] for details.
    #[inline]
    pub fn shrink_to_fit(&mut self) {
        self.raw.shrink_to_fit();
    }

    /// Shrinks the capacity to at least the specified minimum.
    ///
    /// See [`Vec::shrink_to`] for details.
    #[inline]
    pub fn shrink_to(&mut self, min_capacity: usize) {
        self.raw.shrink_to(min_capacity);
    }

    /// Converts the vector into a boxed slice.
    ///
    /// The resulting slice has the same lifetime as the original vector.
    #[inline]
    pub fn into_boxed_slice(self) -> Box<TypedSlice<I, T>> {
        // SAFETY: TypedSlice is repr(transparent) over [T].
        unsafe { core::mem::transmute(self.raw.into_boxed_slice()) }
    }

    /// Shortens the vector to the specified length.
    ///
    /// If `len` is greater than the current length, this has no effect.
    #[inline]
    pub fn truncate(&mut self, len: I) {
        self.raw.truncate(len.to_raw_index());
    }

    /// Returns the vector as a typed slice reference.
    #[inline]
    pub fn as_slice(&self) -> &TypedSlice<I, T> {
        unsafe { TypedSlice::from_slice_unchecked(self.raw.as_slice()) }
    }

    /// Returns a mutable typed slice reference.
    #[inline]
    pub fn as_mut_slice(&mut self) -> &mut TypedSlice<I, T> {
        unsafe { TypedSlice::from_slice_unchecked_mut(self.raw.as_mut_slice()) }
    }

    /// Casts the index type of the `TypedVec`.
    #[inline]
    pub fn cast_index_type<I2: IndexType>(self) -> Result<TypedVec<I2, T>, I2::IndexTooBigError> {
        if I::MAX_RAW_INDEX <= I2::MAX_RAW_INDEX {
            Ok(unsafe { TypedVec::from_vec_unchecked(self.raw) })
        } else {
            TypedVec::try_from_vec(self.raw)
        }
    }

    /// Returns a raw pointer to the vector's buffer.
    #[inline]
    pub const fn as_ptr(&self) -> *const T {
        self.raw.as_ptr()
    }

    /// Sets the length of the vector.
    ///
    /// # Safety
    ///
    /// Same as [`Vec::set_len`], plus the new length must not exceed `I::MAX_RAW_INDEX`.
    #[inline]
    pub unsafe fn set_len(&mut self, new_len: I) {
        unsafe { self.raw.set_len(new_len.to_scalar().to_usize()) };
    }

    /// Removes and returns the element at `index`, swapping the last element into that position.
    ///
    /// This operation is O(1).
    ///
    /// # Panics
    ///
    /// Panics if `index` is out of bounds.
    #[inline]
    pub fn swap_remove(&mut self, index: I) -> T {
        self.raw.swap_remove(index.to_raw_index())
    }

    /// Attempts to insert an element at `index`, shifting all elements after it to the right.
    ///
    /// Returns an error if the new length would exceed `I::MAX_RAW_INDEX`.
    ///
    /// # Panics
    ///
    /// Panics if `index > len`.
    #[inline]
    pub fn try_insert(&mut self, index: I, element: T) -> Result<(), I::IndexTooBigError> {
        let _new_potential_len = self.len().checked_add_scalar(I::Scalar::ONE)?;
        self.raw.insert(index.to_raw_index(), element);
        Ok(())
    }

    /// Inserts an element at `index`, shifting all elements after it to the right.
    ///
    /// # Panics
    ///
    /// Panics if `index > len` or if the new length would exceed `I::MAX_RAW_INDEX`.
    #[inline]
    pub fn insert(&mut self, index: I, element: T) {
        self.try_insert(index, element)
            .unwrap_or_else(|error| panic_index_too_big::<I>(error))
    }

    /// Removes and returns the element at `index`, shifting all elements after it to the left.
    ///
    /// This operation is O(n).
    ///
    /// # Panics
    ///
    /// Panics if `index` is out of bounds.
    #[inline]
    pub fn remove(&mut self, index: I) -> T {
        self.raw.remove(index.to_raw_index())
    }

    /// Retains only elements that satisfy the predicate.
    ///
    /// See [`Vec::retain`] for details.
    #[inline]
    pub fn retain<F>(&mut self, f: F)
    where
        F: FnMut(&T) -> bool,
    {
        self.raw.retain(f)
    }

    /// Retains only elements that satisfy the predicate, passing a mutable reference.
    ///
    /// See [`Vec::retain_mut`] for details.
    #[inline]
    pub fn retain_mut<F>(&mut self, f: F)
    where
        F: FnMut(&mut T) -> bool,
    {
        self.raw.retain_mut(f)
    }

    /// Removes consecutive duplicate elements, using `key` to determine equality.
    ///
    /// See [`Vec::dedup_by_key`] for details.
    #[inline]
    pub fn dedup_by_key<F, K>(&mut self, key: F)
    where
        F: FnMut(&mut T) -> K,
        K: PartialEq,
    {
        self.raw.dedup_by_key(key);
    }

    /// Removes consecutive duplicate elements, using `same_bucket` to determine equality.
    ///
    /// See [`Vec::dedup_by`] for details.
    #[inline]
    pub fn dedup_by<F>(&mut self, same_bucket: F)
    where
        F: FnMut(&mut T, &mut T) -> bool,
    {
        self.raw.dedup_by(same_bucket);
    }

    /// Removes and returns the last element, or `None` if the vector is empty.
    #[inline]
    pub fn pop(&mut self) -> Option<T> {
        self.raw.pop()
    }

    /// Removes and returns the last element if `predicate` returns `true`.
    ///
    /// See [`Vec::pop_if`] for details.
    #[inline]
    pub fn pop_if(&mut self, predicate: impl FnOnce(&mut T) -> bool) -> Option<T> {
        self.raw.pop_if(predicate)
    }

    /// Removes all elements from the vector.
    #[inline]
    pub fn clear(&mut self) {
        self.raw.clear();
    }

    /// Returns `true` if the vector contains no elements.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.raw.is_empty()
    }

    /// Splits the vector into two at the given index.
    ///
    /// Returns everything after the split point.
    #[inline]
    pub fn split_off(&mut self, at: I) -> Self {
        let new_vec = self.raw.split_off(at.to_raw_index());
        unsafe { Self::from_vec_unchecked(new_vec) }
    }

    /// Grows the vector in place, filling new positions with the result of `f`.
    ///
    /// See [`Vec::resize_with`] for details.
    #[inline]
    pub fn resize_with<F>(&mut self, new_len: I, f: F)
    where
        F: FnMut() -> T,
    {
        self.raw.resize_with(new_len.to_scalar().to_usize(), f);
    }

    /// Leaks the vector and returns a mutable reference to its contents.
    ///
    /// See [`Vec::leak`] for details.
    #[inline]
    pub fn leak<'a>(self) -> &'a mut TypedSlice<I, T> {
        let raw = self.raw.leak();
        // SAFETY: The leaked slice has the same length as the vector, which was valid for `I`.
        unsafe { TypedSlice::from_slice_unchecked_mut(raw) }
    }

    /// Creates a draining iterator that removes the specified range.
    ///
    /// See [`Vec::drain`] for details.
    #[inline]
    pub fn drain<R>(&mut self, range: R) -> alloc::vec::Drain<'_, T>
    where
        R: core::ops::RangeBounds<I>,
    {
        self.raw.drain(range_bounds_to_raw(&range))
    }

    /// Creates a splicing iterator that removes the specified range and replaces it.
    ///
    /// See [`Vec::splice`] for details.
    #[inline]
    pub fn splice<R, X>(
        &mut self,
        range: R,
        replace_with: X,
    ) -> alloc::vec::Splice<'_, BoundedSpliceIter<I, X::IntoIter>>
    where
        R: core::ops::RangeBounds<I>,
        X: IntoIterator<Item = T>,
    {
        let resolved_range = resolve_range_bounds(&range, self.len());
        let range_len = resolved_range
            .end
            .checked_sub_index(resolved_range.start)
            .expect("invalid range");
        let remaining_len = self
            .len()
            .checked_sub_scalar(range_len)
            .expect("range out of bounds");
        let replace_with_max_allowed_len =
            unsafe { I::MAX_INDEX.unchecked_sub_index(remaining_len) };
        self.raw.splice(
            range_bounds_to_raw(&range),
            BoundedSpliceIter::new(replace_with.into_iter(), replace_with_max_allowed_len),
        )
    }

    /// Attempts to extend the vector with the contents of an iterator.
    ///
    /// Returns an error if the new length would exceed `I::MAX_RAW_INDEX`.
    /// On error, the vector is unchanged.
    #[inline]
    pub fn try_extend<X: IntoIterator<Item = T>>(
        &mut self,
        iter: X,
    ) -> Result<(), I::IndexTooBigError> {
        let iter = iter.into_iter();

        if let Some(upper_bound) = iter.size_hint().1 {
            let _ = self.len().checked_add_scalar(
                I::Scalar::try_from_usize(upper_bound).ok_or(I::IndexTooBigError::new())?,
            )?;
        }

        let orig_len = self.raw.len();
        for item in iter {
            self.try_push(item).inspect_err(|_err| {
                self.raw.truncate(orig_len);
            })?;
        }
        Ok(())
    }
}

/// An iterator adapter used by [`TypedVec::splice`] to cap replacement growth.
///
/// This wrapper forwards items from an underlying iterator while tracking how many
/// replacement elements may still be yielded without making the final vector length
/// exceed the index type's maximum.
///
/// Once the allowed replacement count is exhausted, further iteration panics.
#[derive(Debug)]
pub struct BoundedSpliceIter<I: IndexType, Iter> {
    inner: Iter,
    remaining: I::Scalar,
}

impl<I: IndexType, Iter> BoundedSpliceIter<I, Iter> {
    #[inline]
    fn new(inner: Iter, remaining: I::Scalar) -> Self {
        Self { inner, remaining }
    }

    #[inline]
    fn take_one(&mut self) {
        self.remaining = self
            .remaining
            .checked_sub_scalar(I::Scalar::ONE)
            .expect("splice would exceed the index type's maximum length");
    }
}

impl<I: IndexType, T, Iter: Iterator<Item = T>> Iterator for BoundedSpliceIter<I, Iter> {
    type Item = T;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let item = self.inner.next()?;
        self.take_one();
        Some(item)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let (lower, upper) = self.inner.size_hint();
        let remaining = self.remaining.to_usize();
        (
            lower.min(remaining),
            upper.map(|upper| upper.min(remaining)),
        )
    }
}

impl<I: IndexType, T, Iter: DoubleEndedIterator<Item = T>> DoubleEndedIterator
    for BoundedSpliceIter<I, Iter>
{
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        let item = self.inner.next_back()?;
        self.take_one();
        Some(item)
    }
}

impl<I: IndexType, T, Iter: ExactSizeIterator<Item = T>> ExactSizeIterator
    for BoundedSpliceIter<I, Iter>
{
    #[inline]
    fn len(&self) -> usize {
        self.inner.len().min(self.remaining.to_usize())
    }
}

impl<I: IndexType, T, Iter: FusedIterator<Item = T>> FusedIterator for BoundedSpliceIter<I, Iter> {}

impl<I: IndexType, T: PartialEq> TypedVec<I, T> {
    /// Removes consecutive duplicate elements.
    ///
    /// See [`Vec::dedup`] for details.
    #[inline]
    pub fn dedup(&mut self) {
        self.raw.dedup();
    }
}

impl<I: IndexType, T: Clone> TypedVec<I, T> {
    /// Extends the vector by cloning elements from a typed slice.
    ///
    /// See [`Vec::extend_from_slice`] for details.
    #[inline]
    pub fn extend_from_slice(&mut self, other: &TypedSlice<I, T>) {
        self.try_extend_from_slice(other)
            .unwrap_or_else(|error| panic_index_too_big::<I>(error))
    }

    /// Attempts to extend the vector by cloning elements from a typed slice.
    ///
    /// Returns an error if the resulting length would exceed `I::MAX_RAW_INDEX`.
    #[inline]
    pub fn try_extend_from_slice(
        &mut self,
        other: &TypedSlice<I, T>,
    ) -> Result<(), I::IndexTooBigError> {
        let _new_len = self.len().checked_add_scalar(other.len().to_scalar())?;
        self.raw.extend_from_slice(other.as_slice());
        Ok(())
    }

    /// Attempts to copy elements from the specified range to the end of the vector.
    ///
    /// Returns an error if the resulting length would exceed `I::MAX_RAW_INDEX`.
    ///
    /// See [`Vec::extend_from_within`] for details.
    #[inline]
    pub fn try_extend_from_within<R>(&mut self, src: R) -> Result<(), I::IndexTooBigError>
    where
        R: core::ops::RangeBounds<I>,
    {
        let src_range = resolve_range_bounds(&src, self.len());
        let src_range_len = src_range
            .end
            .checked_sub_index(src_range.start)
            .expect("invalid range");
        let _ = self.len().checked_add_scalar(src_range_len)?;
        self.raw.extend_from_within(range_bounds_to_raw(&src));
        Ok(())
    }

    /// Copies elements from the specified range to the end of the vector.
    ///
    /// See [`Vec::extend_from_within`] for details.
    #[inline]
    pub fn extend_from_within<R>(&mut self, src: R)
    where
        R: core::ops::RangeBounds<I>,
    {
        self.try_extend_from_within(src)
            .unwrap_or_else(|error| panic_index_too_big::<I>(error))
    }

    /// Creates an iterator that filters and transforms elements, removing them in place.
    ///
    /// See [`Vec::extract_if`] for details.
    #[inline]
    pub fn extract_if<F, R>(&mut self, range: R, filter: F) -> alloc::vec::ExtractIf<'_, T, F>
    where
        F: FnMut(&mut T) -> bool,
        R: core::ops::RangeBounds<I>,
    {
        self.raw.extract_if(range_bounds_to_raw(&range), filter)
    }

    /// Resizes the vector to the specified length, filling new positions with `value`.
    ///
    /// See [`Vec::resize`] for details.
    #[inline]
    pub fn resize(&mut self, new_len: I, value: T) {
        self.raw.resize(new_len.to_raw_index(), value);
    }
}

impl<I: IndexType, T, const N: usize> TypedVec<I, [T; N]> {
    /// Attempts to flatten the vector of arrays into a vector of elements.
    ///
    /// Returns an error if the flattened length would exceed `I::MAX_RAW_INDEX`.
    ///
    /// # Example
    ///
    /// ```
    /// use index_type::IndexType;
    /// use index_type::typed_vec::TypedVec;
    ///
    /// #[derive(IndexType, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    /// struct Idx(u16);
    ///
    /// let vec: TypedVec<Idx, [u8; 2]> = TypedVec::from_vec(vec![[1, 2], [3, 4]]);
    /// let flat: TypedVec<Idx, u8> = vec.try_into_flattened().unwrap();
    /// assert_eq!(flat.len_usize(), 4);
    /// ```
    pub fn try_into_flattened(self) -> Result<TypedVec<I, T>, I::IndexTooBigError> {
        let _new_len = self
            .len()
            .checked_mul_scalar(I::Scalar::try_from_usize(N).ok_or(I::IndexTooBigError::new())?)?;
        Ok(unsafe { TypedVec::from_vec_unchecked(self.raw.into_flattened()) })
    }

    /// Flattens the vector of arrays into a vector of elements.
    ///
    /// # Panics
    ///
    /// Panics if the flattened length would exceed `I::MAX_RAW_INDEX`.
    pub fn into_flattened(self) -> TypedVec<I, T> {
        self.try_into_flattened()
            .unwrap_or_else(|error| panic_index_too_big::<I>(error))
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
impl<I: IndexType, T> IntoIterator for TypedVec<I, T> {
    type Item = T;

    type IntoIter = alloc::vec::IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        self.raw.into_iter()
    }
}
impl<'a, I: IndexType, T> IntoIterator for &'a TypedVec<I, T> {
    type Item = &'a T;

    type IntoIter = core::slice::Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.raw.iter()
    }
}
impl<'a, I: IndexType, T> IntoIterator for &'a mut TypedVec<I, T> {
    type Item = &'a mut T;

    type IntoIter = core::slice::IterMut<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.raw.iter_mut()
    }
}
impl<I: IndexType, T> Extend<T> for TypedVec<I, T> {
    fn extend<X: IntoIterator<Item = T>>(&mut self, iter: X) {
        self.try_extend(iter)
            .unwrap_or_else(|error| panic_index_too_big::<I>(error))
    }
}

impl<I: IndexType, T> FromIterator<T> for TypedVec<I, T> {
    fn from_iter<X: IntoIterator<Item = T>>(iter: X) -> Self {
        Self::from_vec(Vec::from_iter(iter))
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
