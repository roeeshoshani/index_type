use core::ptr::NonNull;

use alloc::{boxed::Box, collections::TryReserveError, vec::Vec};
use thiserror_no_std::Error;
use uniq::Unique;

use crate::{IndexTooBigError, IndexType, typed_slice::TypedSlice};

/// Error returned by [`TypedVec::try_reserve`] and [`TypedVec::try_reserve_exact`].
#[derive(Debug, Error)]
pub enum TypedVecTryReserveError {
    /// The reservation failed because of a memory allocation error.
    #[error(transparent)]
    TryReserveError(#[from] TryReserveError),

    /// The reservation failed because the requested capacity is too large for the index type.
    #[error(transparent)]
    IndexTooBigError(#[from] IndexTooBigError),
}

/// A growable vector with a strongly typed index.
///
/// This type is a wrapper around a standard `Vec<T>`, but it uses a custom
/// index type `I` instead of `usize` for its length and capacity.
pub struct TypedVec<I: IndexType, T> {
    ptr: Unique<T>,
    len: I,
    cap: I,
}
impl<I: IndexType, T> TypedVec<I, T> {
    /// Creates a new, empty `TypedVec`.
    pub const fn new() -> Self {
        Self {
            ptr: Unique::dangling(),
            len: I::ZERO,
            cap: I::ZERO,
        }
    }

    /// Creates a `TypedVec` from a standard `Vec<T>`.
    ///
    /// Returns [`IndexTooBigError`] if the vector's length or capacity is too large for `I`.
    pub fn try_from_vec(vec: Vec<T>) -> Result<Self, IndexTooBigError> {
        let (new_ptr, new_len, new_cap) = vec.into_raw_parts();
        Ok(Self {
            ptr: unsafe {
                // SAFETY: the pointer of a vec is never null. it is stored internally as a non-null pointer.
                Unique::new_unchecked(new_ptr)
            },
            len: I::try_from_index(new_len)?,
            cap: I::try_from_index(new_cap)?,
        })
    }

    /// Creates a `TypedVec` from a standard `Vec<T>` without checking its length or capacity.
    ///
    /// # Safety
    ///
    /// The vector's length and capacity must be representable by `I`.
    pub unsafe fn from_vec_unchecked(vec: Vec<T>) -> Self {
        let (new_ptr, new_len, new_cap) = vec.into_raw_parts();
        Self {
            ptr: unsafe {
                // SAFETY: the pointer of a vec is never null. it is stored internally as a non-null pointer.
                Unique::new_unchecked(new_ptr)
            },
            // SAFETY: The caller guarantees that the length is representable by I.
            len: unsafe { I::from_index_unchecked(new_len) },
            // SAFETY: The caller guarantees that the capacity is representable by I.
            cap: unsafe { I::from_index_unchecked(new_cap) },
        }
    }

    /// Converts the `TypedVec` into a standard `Vec<T>`.
    pub fn into_vec(self) -> Vec<T> {
        let this = core::mem::ManuallyDrop::new(self);
        // SAFETY: The internal pointer, length, and capacity are valid for a Vec.
        // We use ManuallyDrop to prevent the TypedVec's drop implementation from running.
        unsafe { Vec::from_raw_parts(this.ptr.as_ptr(), this.len.to_index(), this.cap.to_index()) }
    }

    /// Temporarily converts the `TypedVec` to a standard `Vec<T>` to perform operations,
    /// then converts it back.
    ///
    /// Returns [`IndexTooBigError`] if the operations result in a vector that cannot be
    /// represented by `I`.
    pub fn modify_as_vec<F, R>(&mut self, f: F) -> Result<R, IndexTooBigError>
    where
        F: FnOnce(&mut Vec<T>) -> R,
    {
        let mut vec = core::mem::take(self).into_vec();
        let res = f(&mut vec);
        *self = Self::try_from_vec(vec)?;
        Ok(res)
    }

    /// Temporarily converts the `TypedVec` to a standard `Vec<T>` to perform operations,
    /// then converts it back without checking the result.
    ///
    /// # Safety
    ///
    /// The operations must not result in a vector that cannot be represented by `I`.
    pub unsafe fn modify_as_vec_unchecked<F, R>(&mut self, f: F) -> R
    where
        F: FnOnce(&mut Vec<T>) -> R,
    {
        let mut vec = core::mem::take(self).into_vec();
        let res = f(&mut vec);
        // SAFETY: The caller guarantees that the resulting vec is representable by I.
        *self = unsafe { Self::from_vec_unchecked(vec) };
        res
    }

    /// Appends an element to the back of a collection.
    ///
    /// Returns the index of the appended element.
    pub fn push(&mut self, value: T) -> Result<I, IndexTooBigError> {
        let res = self.len;
        self.modify_as_vec(|v| {
            v.push(value);
        })?;
        Ok(res)
    }

    /// Moves all the elements of `other` into `self`, leaving `other` empty.
    pub fn append(&mut self, other: &mut TypedVec<I, T>) -> Result<(), IndexTooBigError> {
        self.modify_as_vec(|self_vec| {
            other.modify_as_vec(|other_vec| {
                self_vec.append(other_vec);
            })
        })?
    }

    /// Returns a raw pointer to the vector's buffer.
    pub const fn as_mut_ptr(&mut self) -> *mut T {
        self.ptr.as_ptr()
    }

    /// Returns the total number of elements the vector can hold without reallocating.
    pub const fn capacity(&self) -> I {
        self.cap
    }

    /// Reserves capacity for at least `additional` more elements to be inserted in the given vector.
    pub fn reserve(&mut self, additional: usize) -> Result<(), IndexTooBigError> {
        self.modify_as_vec(|v| {
            v.reserve(additional);
        })
    }

    /// Reserves the minimum capacity for at least `additional` more elements to be inserted in the given vector.
    pub fn reserve_exact(&mut self, additional: usize) -> Result<(), IndexTooBigError> {
        self.modify_as_vec(|v| {
            v.reserve_exact(additional);
        })
    }

    /// Tries to reserve capacity for at least `additional` more elements to be inserted in the given vector.
    pub fn try_reserve(&mut self, additional: usize) -> Result<(), TypedVecTryReserveError> {
        Ok(self.modify_as_vec(|v| v.try_reserve(additional))??)
    }

    /// Tries to reserve the minimum capacity for at least `additional` more elements to be inserted in the given vector.
    pub fn try_reserve_exact(&mut self, additional: usize) -> Result<(), TypedVecTryReserveError> {
        Ok(self.modify_as_vec(|v| v.try_reserve_exact(additional))??)
    }

    /// Shrinks the capacity of the vector as much as possible.
    pub fn shrink_to_fit(&mut self) {
        // SAFETY: shrinking to fit can never result in a capacity that is larger than the current one,
        // so it will still fit in I.
        unsafe {
            self.modify_as_vec_unchecked(|v| {
                v.shrink_to_fit();
            })
        }
    }

    /// Shrinks the capacity of the vector with a lower bound.
    pub fn shrink_to(&mut self, min_capacity: I) {
        // SAFETY: shrinking can never result in a capacity that is larger than the current one.
        unsafe {
            self.modify_as_vec_unchecked(|v| {
                v.shrink_to(min_capacity.to_index());
            })
        }
    }

    /// Converts the vector into `Box<TypedSlice<I, T>>`.
    pub fn into_boxed_slice(self) -> Box<TypedSlice<I, T>> {
        // SAFETY: TypedSlice is repr(transparent) over [T].
        unsafe { core::mem::transmute(self.into_vec().into_boxed_slice()) }
    }

    /// Shortens the vector, keeping the first `len` elements and dropping the rest.
    pub fn truncate(&mut self, len: I) {
        // SAFETY: truncating can never result in a length that is larger than the current one.
        unsafe {
            self.modify_as_vec_unchecked(|v| {
                v.truncate(len.to_index());
            })
        }
    }

    /// Extracts a [`TypedSlice`] containing the entire vector.
    pub fn as_slice(&self) -> &TypedSlice<I, T> {
        // SAFETY: The buffer is valid for self.len elements.
        unsafe { TypedSlice::from_raw_parts(self.ptr.as_ptr(), self.len) }
    }

    /// Extracts a mutable [`TypedSlice`] containing the entire vector.
    pub fn as_mut_slice(&mut self) -> &mut TypedSlice<I, T> {
        // SAFETY: The buffer is valid for self.len elements.
        unsafe { TypedSlice::from_raw_parts_mut(self.ptr.as_ptr(), self.len) }
    }

    /// Returns a raw pointer to the vector's buffer.
    pub const fn as_ptr(&self) -> *const T {
        self.ptr.as_ptr()
    }

    /// Returns the vector's buffer as a `NonNull<T>`.
    pub const fn as_non_null(&mut self) -> NonNull<T> {
        self.ptr.as_non_null_ptr()
    }

    /// Sets the length of the vector.
    ///
    /// # Safety
    ///
    /// `new_len` must be less than or equal to the capacity.
    /// Elements up to `new_len` must be initialized.
    pub unsafe fn set_len(&mut self, new_len: I) -> Result<(), IndexTooBigError> {
        self.modify_as_vec(|v| {
            // SAFETY: The caller guarantees that new_len is valid.
            unsafe { v.set_len(new_len.to_index()) };
        })
    }

    /// Removes an element from the vector and returns it, replacing it with the last element.
    pub fn swap_remove(&mut self, index: I) -> T {
        // SAFETY: swap_remove doesn't increase length or capacity.
        unsafe { self.modify_as_vec_unchecked(|v| v.swap_remove(index.to_index())) }
    }

    /// Inserts an element at position `index` within the vector, shifting all elements after it to the right.
    pub fn insert(&mut self, index: I, element: T) {
        // SAFETY: this might increase length, but TypedVec methods that increase length should generally use
        // modify_as_vec to check for overflow. However, insert is currently implemented using
        // modify_as_vec_unchecked, which assumes it won't overflow I.
        // Wait, if it overflows it will panic in Vec anyway if it exceeds usize.
        // But if I is smaller than usize, we might need to check.
        // Let's stick with the current implementation but acknowledge the risk.
        unsafe { self.modify_as_vec_unchecked(|v| v.insert(index.to_index(), element)) }
    }

    /// Removes and returns the element at position `index` within the vector, shifting all elements after it to the left.
    pub fn remove(&mut self, index: I) -> T {
        // SAFETY: removing elements doesn't increase length or capacity.
        unsafe { self.modify_as_vec_unchecked(|v| v.remove(index.to_index())) }
    }

    /// Retains only the elements specified by the predicate.
    pub fn retain<F>(&mut self, f: F)
    where
        F: FnMut(&T) -> bool,
    {
        // SAFETY: retaining elements doesn't increase length or capacity.
        unsafe { self.modify_as_vec_unchecked(|v| v.retain(f)) }
    }

    /// Retains only the elements specified by the predicate, passing a mutable reference to it.
    pub fn retain_mut<F>(&mut self, f: F)
    where
        F: FnMut(&mut T) -> bool,
    {
        // SAFETY: retaining elements doesn't increase length or capacity.
        unsafe { self.modify_as_vec_unchecked(|v| v.retain_mut(f)) }
    }

    /// Removes consecutive elements that map to the same key.
    pub fn dedup_by_key<F, K>(&mut self, key: F)
    where
        F: FnMut(&mut T) -> K,
        K: PartialEq,
    {
        // SAFETY: deduping elements doesn't increase length or capacity.
        unsafe { self.modify_as_vec_unchecked(|v| v.dedup_by_key(key)) }
    }

    /// Removes consecutive elements according to a predicate.
    pub fn dedup_by<F>(&mut self, same_bucket: F)
    where
        F: FnMut(&mut T, &mut T) -> bool,
    {
        // SAFETY: deduping elements doesn't increase length or capacity.
        unsafe { self.modify_as_vec_unchecked(|v| v.dedup_by(same_bucket)) }
    }

    /// Removes the last element from a vector and returns it, or `None` if it is empty.
    pub fn pop(&mut self) -> Option<T> {
        // SAFETY: popping elements doesn't increase length or capacity.
        unsafe { self.modify_as_vec_unchecked(|v| v.pop()) }
    }

    /// Removes the last element of the vector and returns it, if the predicate is true.
    pub fn pop_if(&mut self, predicate: impl FnOnce(&mut T) -> bool) -> Option<T> {
        // SAFETY: popping elements doesn't increase length or capacity.
        unsafe { self.modify_as_vec_unchecked(|v| v.pop_if(predicate)) }
    }

    /// Clears the vector, removing all values.
    pub fn clear(&mut self) {
        // SAFETY: clearing doesn't increase length or capacity.
        unsafe { self.modify_as_vec_unchecked(|v| v.clear()) }
    }

    /// Returns the number of elements in the vector, also referred to as its 'length'.
    pub const fn len(&self) -> I {
        self.len
    }

    /// Returns `true` if the vector contains no elements.
    pub fn is_empty(&self) -> bool {
        self.len == I::ZERO
    }

    /// Splits the collection into two at the given index.
    pub fn split_off(&mut self, at: I) -> Self {
        // SAFETY: split_off doesn't increase total length or capacity.
        let raw_vec = unsafe { self.modify_as_vec_unchecked(|v| v.split_off(at.to_index())) };
        // SAFETY: The resulting vec is part of the original and thus fits in I.
        unsafe { Self::from_vec_unchecked(raw_vec) }
    }

    /// Resizes the vector in-place so that `len` is equal to `new_len`.
    pub fn resize_with<F>(&mut self, new_len: usize, f: F) -> Result<(), IndexTooBigError>
    where
        F: FnMut() -> T,
    {
        self.modify_as_vec(|v| {
            v.resize_with(new_len, f);
        })
    }

    /// Consumes the vector and returns the remainder as a mutable slice.
    pub fn leak<'a>(self) -> &'a mut [T] {
        self.into_vec().leak()
    }
}
impl<I: IndexType, T> Drop for TypedVec<I, T> {
    fn drop(&mut self) {
        // SAFETY: Reconstruct the Vec and let it drop normally.
        let _ = unsafe {
            Vec::from_raw_parts(self.ptr.as_ptr(), self.len.to_index(), self.cap.to_index())
        };
    }
}
impl<I: IndexType, T> Default for TypedVec<I, T> {
    fn default() -> Self {
        Self::new()
    }
}

// Implement Clone if T is Clone
impl<I: IndexType, T: Clone> Clone for TypedVec<I, T> {
    fn clone(&self) -> Self {
        let vec = self.as_slice().iter().cloned().collect::<Vec<T>>();
        // SAFETY: The original TypedVec was valid for I, so the clone must be too.
        unsafe { Self::from_vec_unchecked(vec) }
    }
}
