//! A fixed-capacity vector with typed indexing.
//!
//! This module provides [`TypedArrayVec`], a vector with a fixed maximum capacity that uses a
//! custom [`IndexType`] for both indexing and storing the length. This is ideal for embedded
//! systems or scenarios where you need predictable memory usage.
//!
//! # No Heap Allocation After Creation
//!
//! Unlike `TypedVec`, `TypedArrayVec` has a fixed capacity determined at compile time. Once created,
//! it will never allocate additional memory. Operations that would exceed capacity return errors
//! or panic.
//!
//! # Compile-Time Capacity Check
//!
//! The capacity `N` is checked at compile time to ensure it fits within the index type `I`'s
//! representable range.
//!
//! # Example
//!
//! ```
//! use index_type::IndexType;
//! use index_type::typed_array_vec::TypedArrayVec;
//!
//! #[derive(IndexType, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
//! struct BufferIdx(u8);
//!
//! let mut buffer: TypedArrayVec<BufferIdx, u8, 16> = TypedArrayVec::new();
//! buffer.push(42);
//! assert_eq!(buffer.len().to_raw_index(), 1);
//! assert!(!buffer.is_full());
//! ```

use core::{
    iter::FusedIterator,
    mem::MaybeUninit,
    ops::{Index, IndexMut},
};

use crate::{
    IndexScalarType, IndexType,
    typed_array::TypedArray,
    typed_iter_enumerate::TypedIterEnumerate,
    typed_range_iter::{TypedRangeIter, TypedRangeIterExt},
    typed_slice::TypedSlice,
    utils::resolve_range_bounds,
};

#[cold]
fn panic_insufficient_capacity() -> ! {
    panic!("insufficient capacity")
}

/// A fixed-capacity, typed vector.
pub struct TypedArrayVec<I: IndexType, T, const N: usize> {
    storage: TypedArray<I, MaybeUninit<T>, N>,
    len: I,
}

impl<I: IndexType, T, const N: usize> TypedArrayVec<I, T, N> {
    const _ASSERT_CAPACITY_IN_INDEX_BOUNDS: () = if N > I::MAX_RAW_INDEX {
        panic!("capacity is not in bounds of the index type");
    };

    /// Creates a new, empty `TypedArrayVec`.
    #[inline]
    pub const fn new() -> Self {
        let _ = Self::_ASSERT_CAPACITY_IN_INDEX_BOUNDS;
        Self {
            storage: TypedArray::from_array([const { MaybeUninit::uninit() }; N]),
            len: I::ZERO,
        }
    }

    /// Returns the number of elements in the `TypedArrayVec` as an index.
    #[inline]
    pub const fn len(&self) -> I {
        self.len
    }

    /// Returns the number of elements in the `TypedArrayVec` as a `usize`.
    #[inline]
    pub fn len_usize(&self) -> usize {
        self.len.to_raw_index()
    }

    /// Returns an iterator over the valid indices of this vector.
    #[inline]
    pub fn indices(&self) -> TypedRangeIter<I> {
        (I::ZERO..self.len()).iter()
    }

    /// Returns an iterator over the elements with their indices.
    #[inline]
    pub fn iter_enumerated(&self) -> TypedIterEnumerate<I, T, core::slice::Iter<'_, T>> {
        // SAFETY: `self.as_slice().iter()` yields exactly `self.len()` items, which already fit in `I`.
        unsafe { TypedIterEnumerate::new(self.as_slice().iter()) }
    }

    /// Returns an iterator over the elements with their mutable references and indices.
    #[inline]
    pub fn iter_mut_enumerated(&mut self) -> TypedIterEnumerate<I, T, core::slice::IterMut<'_, T>> {
        // SAFETY: `self.as_mut_slice().iter_mut()` yields exactly `self.len()` items, which already fit in `I`.
        unsafe { TypedIterEnumerate::new(self.as_mut_slice().iter_mut()) }
    }

    /// Consumes the vector and returns an iterator over the elements with their indices.
    #[inline]
    pub fn into_iter_enumerated(self) -> TypedIterEnumerate<I, T, IntoIter<I, T, N>> {
        // SAFETY: `self.into_iter()` yields exactly the vector length, which already fits in `I`.
        unsafe { TypedIterEnumerate::new(self.into_iter()) }
    }

    /// Returns the capacity of the `TypedArrayVec` as an index.
    #[inline]
    pub fn capacity(&self) -> I {
        // SAFETY: The capacity N is guaranteed to be in bounds for I by the type system and our check.
        unsafe { I::from_raw_index_unchecked(N) }
    }

    /// Returns the remaining capacity of the `TypedArrayVec` as an index.
    #[inline]
    pub fn remaining_capacity(&self) -> I {
        let diff = unsafe { self.capacity().unchecked_sub_index(self.len()) };
        unsafe { I::from_scalar_unchecked(diff) }
    }

    /// Returns true if the `TypedArrayVec` is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len == I::ZERO
    }

    /// Returns true if the `TypedArrayVec` is full.
    #[inline]
    pub fn is_full(&self) -> bool {
        self.len == self.capacity()
    }

    /// Returns the `TypedArrayVec` as a `TypedSlice`.
    #[inline]
    pub fn as_slice(&self) -> &TypedSlice<I, T> {
        // SAFETY: The storage is initialized up to self.len.
        unsafe { TypedSlice::from_raw_parts(self.storage.as_ptr().cast(), self.len) }
    }

    /// Returns the `TypedArrayVec` as a mutable `TypedSlice`.
    #[inline]
    pub fn as_mut_slice(&mut self) -> &mut TypedSlice<I, T> {
        // SAFETY: The storage is initialized up to self.len.
        unsafe { TypedSlice::from_raw_parts_mut(self.storage.as_mut_ptr().cast(), self.len) }
    }

    /// Casts the index type of the `TypedArrayVec`.
    #[inline]
    pub fn cast_index_type<I2: IndexType>(
        self,
    ) -> Result<TypedArrayVec<I2, T, N>, I2::IndexTooBigError> {
        let len = self.len;
        let storage = unsafe { core::ptr::read(&self.storage) };
        let casted_storage = storage.cast_index_type()?;
        let result = TypedArrayVec {
            storage: casted_storage,
            len: unsafe { I2::from_raw_index_unchecked(len.to_raw_index()) },
        };
        core::mem::forget(self);
        Ok(result)
    }

    /// Appends an element to the back of the `TypedArrayVec`.
    ///
    /// # Panics
    ///
    /// Panics if the `TypedArrayVec` is full.
    #[inline]
    pub fn push(&mut self, element: T) -> I {
        self.try_push(element)
            .unwrap_or_else(|_| panic_insufficient_capacity())
    }

    /// Tries to append an element to the back of the `TypedArrayVec`.
    ///
    /// Returns the index of the inserted element, or an error if the `TypedArrayVec` is full.
    #[inline]
    pub fn try_push(&mut self, element: T) -> Result<I, CapacityError<T>> {
        if self.is_full() {
            return Err(CapacityError::new(element));
        }
        let idx = self.len;
        // SAFETY: The capacity is not exceeded.
        unsafe {
            self.storage.get_unchecked_mut(self.len).write(element);
            self.len = self.len.unchecked_add_scalar(I::Scalar::ONE);
        }
        Ok(idx)
    }

    /// Appends elements from a `TypedSlice` to the `TypedArrayVec`.
    ///
    /// # Panics
    ///
    /// Panics if the `TypedArrayVec` does not have enough capacity.
    #[inline]
    pub fn extend_from_slice(&mut self, other: &TypedSlice<I, T>)
    where
        T: Clone,
    {
        self.try_extend_from_slice(other)
            .unwrap_or_else(|_| panic_insufficient_capacity())
    }

    /// Tries to append elements from a `TypedSlice` to the `TypedArrayVec`.
    ///
    /// Returns an error if the `TypedArrayVec` does not have enough capacity.
    #[inline]
    pub fn try_extend_from_slice(
        &mut self,
        other: &TypedSlice<I, T>,
    ) -> Result<(), CapacityError<()>>
    where
        T: Clone,
    {
        let new_len = self
            .len()
            .checked_add_scalar(other.len().to_scalar())
            .map_err(|_| CapacityError::new(()))?;

        if new_len > self.capacity() {
            return Err(CapacityError::new(()));
        }

        for item in other.as_slice() {
            // SAFETY: We checked capacity.
            unsafe {
                self.storage.get_unchecked_mut(self.len).write(item.clone());
                self.len = self.len.unchecked_add_scalar(I::Scalar::ONE);
            }
        }
        Ok(())
    }

    /// Removes the last element from the `TypedArrayVec` and returns it, or `None` if it is empty.
    #[inline]
    pub fn pop(&mut self) -> Option<T> {
        let new_len = self.len.checked_sub_scalar(I::Scalar::ONE)?;
        self.len = new_len;
        // SAFETY: The subtraction succeeded, so the element at new_len is initialized.
        unsafe { Some(self.storage.get_unchecked(new_len).assume_init_read()) }
    }

    /// Clears the `TypedArrayVec`, removing all elements.
    #[inline]
    pub fn clear(&mut self) {
        self.truncate(I::ZERO);
    }

    /// Shortens the `TypedArrayVec`, keeping the first `len` elements and dropping the rest.
    #[inline]
    pub fn truncate(&mut self, len: I) {
        if len >= self.len {
            return;
        }

        let old_len = core::mem::replace(&mut self.len, len);

        // SAFETY: storage is initialized up to old_len.
        unsafe {
            let tail_len = old_len.unchecked_sub_index(len);
            let tail = core::slice::from_raw_parts_mut(
                self.storage.as_mut_ptr().add(len.to_raw_index()),
                tail_len.to_usize(),
            );
            core::ptr::drop_in_place(tail);
        }
    }

    /// Inserts an element at position `index` within the `TypedArrayVec`, shifting all elements after it to the right.
    ///
    /// # Panics
    ///
    /// Panics if the `TypedArrayVec` is full or if `index > len`.
    #[inline]
    pub fn insert(&mut self, index: I, element: T) {
        self.try_insert(index, element)
            .unwrap_or_else(|_| panic_insufficient_capacity())
    }

    /// Tries to insert an element at position `index` within the `TypedArrayVec`, shifting all elements after it to the right.
    ///
    /// Returns an error if the `TypedArrayVec` is full.
    ///
    /// # Panics
    ///
    /// Panics if `index > len`.
    #[inline]
    pub fn try_insert(&mut self, index: I, element: T) -> Result<(), CapacityError<T>> {
        let old_len = self.len;

        assert!(index <= old_len, "index out of bounds");

        if self.is_full() {
            return Err(CapacityError::new(element));
        }

        // SAFETY: We checked bounds and capacity.
        unsafe {
            let p = self.storage.as_mut_ptr().add(index.to_raw_index());
            core::ptr::copy(p, p.add(1), old_len.unchecked_sub_index(index).to_usize());
            core::ptr::write(p, MaybeUninit::new(element));
            self.len = self.len.unchecked_add_scalar(I::Scalar::ONE);
        }
        Ok(())
    }

    /// Removes and returns the element at position `index` within the `TypedArrayVec`, shifting all elements after it to the left.
    ///
    /// # Panics
    ///
    /// Panics if `index >= len`.
    #[inline]
    pub fn remove(&mut self, index: I) -> T {
        let old_len = self.len;

        assert!(index < old_len, "index out of bounds");

        // SAFETY: We checked bounds.
        unsafe {
            let new_len = old_len.unchecked_sub_scalar(I::Scalar::ONE);

            let p = self.storage.as_mut_ptr().add(index.to_raw_index());
            let result = p.read().assume_init();
            core::ptr::copy(p.add(1), p, new_len.unchecked_sub_index(index).to_usize());

            self.len = new_len;

            result
        }
    }

    /// Removes an element from the `TypedArrayVec` and returns it, replacing it with the last element.
    ///
    /// # Panics
    ///
    /// Panics if `index >= len`.
    #[inline]
    pub fn swap_remove(&mut self, index: I) -> T {
        let old_len = self.len;

        assert!(index < old_len, "index out of bounds");

        // SAFETY: We checked bounds.
        unsafe {
            let last_idx = old_len.unchecked_sub_scalar(I::Scalar::ONE);

            self.len = last_idx;

            let last = self.storage.get_unchecked(last_idx).assume_init_read();

            if index < last_idx {
                core::mem::replace(
                    &mut *self.storage.get_unchecked_mut(index).as_mut_ptr(),
                    last,
                )
            } else {
                last
            }
        }
    }

    #[inline]
    pub fn as_ptr(&self) -> *const T {
        self.storage.as_ptr().cast()
    }

    #[inline]
    pub fn as_mut_ptr(&mut self) -> *mut T {
        self.storage.as_mut_ptr().cast()
    }

    /// Retains only the elements specified by the predicate.
    pub fn retain<F>(&mut self, mut f: F)
    where
        F: FnMut(&T) -> bool,
    {
        // NOTE: the implementation was copied from the stdlib implementation of `Vec::retain_mut`.

        let old_len = self.len();

        if old_len == I::ZERO {
            // Empty case: explicit return allows better optimization, vs letting compiler infer it
            return;
        }

        // Vec: [Kept, Kept, Hole, Hole, Hole, Hole, Unchecked, Unchecked]
        //      |            ^- write                ^- read             |
        //      |<-              original_len                          ->|
        // Kept: Elements which predicate returns true on.
        // Hole: Moved or dropped element slot.
        // Unchecked: Unchecked valid elements.
        //
        // This drop guard will be invoked when predicate or `drop` of element panicked.
        // It shifts unchecked elements to cover holes and `set_len` to the correct length.
        // In cases when predicate and `drop` never panick, it will be optimized out.
        struct PanicGuard<'a, I: IndexType, T, const N: usize> {
            v: &'a mut TypedArrayVec<I, T, N>,
            read: I,
            write: I,
            original_len: I,
        }

        impl<'a, I: IndexType, T, const N: usize> Drop for PanicGuard<'a, I, T, N> {
            #[cold]
            fn drop(&mut self) {
                let remaining = unsafe { self.original_len.unchecked_sub_index(self.read) };

                // SAFETY: Trailing unchecked items must be valid since we never touch them.
                unsafe {
                    core::ptr::copy(
                        self.v.as_ptr().add(self.read.to_raw_index()),
                        self.v.as_mut_ptr().add(self.write.to_raw_index()),
                        remaining.to_usize(),
                    );
                }
                // SAFETY: After filling holes, all items are in contiguous memory.
                unsafe {
                    self.v.set_len(self.write.unchecked_add_scalar(remaining));
                }
            }
        }

        let mut read = I::ZERO;
        loop {
            // SAFETY: read < original_len
            let cur = unsafe { self.get_unchecked_mut(read) };
            if !f(cur) {
                break;
            }
            read = unsafe { read.unchecked_add_scalar(I::Scalar::ONE) };
            if read == old_len {
                // All elements are kept, return early.
                return;
            }
        }

        // Critical section starts here and at least one element is going to be removed.
        // Advance `g.read` early to avoid double drop if `drop_in_place` panicked.
        let mut g = PanicGuard {
            v: self,
            read: unsafe { read.unchecked_add_scalar(I::Scalar::ONE) },
            write: read,
            original_len: old_len,
        };
        unsafe { core::ptr::drop_in_place(&mut *g.v.as_mut_ptr().add(read.to_raw_index())) };

        while g.read < g.original_len {
            let cur = unsafe { &mut *g.v.as_mut_ptr().add(g.read.to_raw_index()) };
            if !f(cur) {
                // Advance `read` early to avoid double drop if `drop_in_place` panicked.
                g.read = unsafe { g.read.unchecked_add_scalar(I::Scalar::ONE) };
                unsafe { core::ptr::drop_in_place(cur) };
            } else {
                // We use copy for move, and never touch the source element again.
                unsafe {
                    let hole = g.v.as_mut_ptr().add(g.write.to_raw_index());
                    core::ptr::copy_nonoverlapping(cur, hole, 1);
                }
                g.write = unsafe { g.write.unchecked_add_scalar(I::Scalar::ONE) };
                g.read = unsafe { g.read.unchecked_add_scalar(I::Scalar::ONE) };
            }
        }

        // We are leaving the critical section and no panic happened,
        // Commit the length change and forget the guard.
        unsafe { g.v.set_len(g.write) };
        core::mem::forget(g);
    }

    /// Set the vector’s length without dropping or moving out elements
    ///
    /// This method is `unsafe` because it changes the notion of the
    /// number of “valid” elements in the vector. Use with care.
    ///
    /// This method uses *debug assertions* to check that `length` is
    /// not greater than the capacity.
    #[inline]
    pub unsafe fn set_len(&mut self, length: I) {
        debug_assert!(length <= self.capacity());
        self.len = length;
    }

    /// Returns a draining iterator that removes the specified range in the vector and yields the removed items.
    pub fn drain<R>(&mut self, range: R) -> Drain<'_, I, T, N>
    where
        R: core::ops::RangeBounds<I>,
    {
        let range = resolve_range_bounds(&range, self.len);
        let old_len = self.len;

        assert!(
            range.start <= range.end && range.end <= self.len,
            "drain range out of bounds"
        );

        // We set the length to start, elements from start to end will be moved out by Drain.
        // Elements from end to old_len will be moved back after Drain is dropped.
        self.len = range.start;

        Drain {
            cur_start: range.start,
            cur_end: range.end,
            tail_start: range.end,
            old_len,
            inner: self,
        }
    }
}

#[derive(Debug)]
pub struct Drain<'a, I: IndexType, T, const N: usize> {
    inner: &'a mut TypedArrayVec<I, T, N>,
    cur_start: I,
    cur_end: I,
    tail_start: I,
    old_len: I,
}

impl<I: IndexType, T, const N: usize> Iterator for Drain<'_, I, T, N> {
    type Item = T;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.cur_start < self.cur_end {
            let res = unsafe {
                self.inner
                    .storage
                    .get_unchecked(self.cur_start)
                    .assume_init_read()
            };
            self.cur_start = unsafe { self.cur_start.unchecked_add_scalar(I::Scalar::ONE) };
            Some(res)
        } else {
            None
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = unsafe { self.cur_end.unchecked_sub_index(self.cur_start) }.to_usize();
        (remaining, Some(remaining))
    }
}

impl<I: IndexType, T, const N: usize> DoubleEndedIterator for Drain<'_, I, T, N> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.cur_start < self.cur_end {
            self.cur_end = unsafe { self.cur_end.unchecked_sub_scalar(I::Scalar::ONE) };
            let res = unsafe {
                self.inner
                    .storage
                    .get_unchecked(self.cur_end)
                    .assume_init_read()
            };
            Some(res)
        } else {
            None
        }
    }
}

impl<I: IndexType, T, const N: usize> ExactSizeIterator for Drain<'_, I, T, N> {}
impl<I: IndexType, T, const N: usize> FusedIterator for Drain<'_, I, T, N> {}

impl<I: IndexType, T, const N: usize> Drop for Drain<'_, I, T, N> {
    fn drop(&mut self) {
        // Drop remaining elements in the range.
        while self.next().is_some() {}

        // Move the tail back.
        let tail_len = unsafe { self.old_len.unchecked_sub_index(self.tail_start) };
        if tail_len > I::Scalar::ZERO {
            unsafe {
                let storage_ptr = self.inner.storage.as_mut_ptr();
                let src = storage_ptr.add(self.tail_start.to_raw_index());
                let dst = storage_ptr.add(self.inner.len.to_raw_index());
                core::ptr::copy(src, dst, tail_len.to_usize());
            }
        }

        // Update the length.
        unsafe {
            self.inner.len = self.inner.len.unchecked_add_scalar(tail_len);
        }
    }
}

impl<I: IndexType, T, const N: usize> Drop for TypedArrayVec<I, T, N> {
    fn drop(&mut self) {
        self.clear();
    }
}

impl<I: IndexType, T, const N: usize> core::ops::Deref for TypedArrayVec<I, T, N> {
    type Target = TypedSlice<I, T>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

impl<I: IndexType, T, const N: usize> core::ops::DerefMut for TypedArrayVec<I, T, N> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut_slice()
    }
}

impl<I: IndexType, T, const N: usize> Index<I> for TypedArrayVec<I, T, N> {
    type Output = T;

    #[inline]
    fn index(&self, index: I) -> &Self::Output {
        &self.as_slice()[index]
    }
}

impl<I: IndexType, T, const N: usize> IndexMut<I> for TypedArrayVec<I, T, N> {
    #[inline]
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        &mut self.as_mut_slice()[index]
    }
}

impl<I: IndexType, T: Clone, const N: usize> Clone for TypedArrayVec<I, T, N> {
    fn clone(&self) -> Self {
        let mut new = Self::new();
        for item in self.as_slice().as_slice() {
            let _ = new.push(item.clone());
        }
        new
    }
}

impl<I: IndexType, T, const N: usize> Default for TypedArrayVec<I, T, N> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<I: IndexType, T: core::fmt::Debug, const N: usize> core::fmt::Debug
    for TypedArrayVec<I, T, N>
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::Debug::fmt(self.as_slice().as_slice(), f)
    }
}

impl<I: IndexType, T: PartialEq, const N: usize> PartialEq for TypedArrayVec<I, T, N> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.as_slice().as_slice() == other.as_slice().as_slice()
    }
}

impl<I: IndexType, T: Eq, const N: usize> Eq for TypedArrayVec<I, T, N> {}

impl<I: IndexType, T: PartialOrd, const N: usize> PartialOrd for TypedArrayVec<I, T, N> {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        self.as_slice()
            .as_slice()
            .partial_cmp(other.as_slice().as_slice())
    }
}

impl<I: IndexType, T: Ord, const N: usize> Ord for TypedArrayVec<I, T, N> {
    #[inline]
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.as_slice().as_slice().cmp(other.as_slice().as_slice())
    }
}

impl<I: IndexType, T: core::hash::Hash, const N: usize> core::hash::Hash
    for TypedArrayVec<I, T, N>
{
    #[inline]
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.as_slice().as_slice().hash(state);
    }
}

impl<I: IndexType, T, const N: usize> AsRef<TypedSlice<I, T>> for TypedArrayVec<I, T, N> {
    #[inline]
    fn as_ref(&self) -> &TypedSlice<I, T> {
        self.as_slice()
    }
}

impl<I: IndexType, T, const N: usize> AsMut<TypedSlice<I, T>> for TypedArrayVec<I, T, N> {
    #[inline]
    fn as_mut(&mut self) -> &mut TypedSlice<I, T> {
        self.as_mut_slice()
    }
}

impl<I: IndexType, T, const N: usize> core::borrow::Borrow<TypedSlice<I, T>>
    for TypedArrayVec<I, T, N>
{
    #[inline]
    fn borrow(&self) -> &TypedSlice<I, T> {
        self.as_slice()
    }
}

impl<I: IndexType, T, const N: usize> core::borrow::BorrowMut<TypedSlice<I, T>>
    for TypedArrayVec<I, T, N>
{
    #[inline]
    fn borrow_mut(&mut self) -> &mut TypedSlice<I, T> {
        self.as_mut_slice()
    }
}

impl<I: IndexType, T, const N: usize> IntoIterator for TypedArrayVec<I, T, N> {
    type Item = T;
    type IntoIter = IntoIter<I, T, N>;

    #[inline]
    fn into_iter(mut self) -> Self::IntoIter {
        let len = self.len;
        // Set length to zero so that the original TypedArrayVec doesn't drop its elements.
        self.len = I::ZERO;
        // SAFETY: We are moving the storage out of self.
        let storage = unsafe { core::ptr::read(&self.storage) };
        core::mem::forget(self);
        IntoIter {
            storage,
            index: I::ZERO,
            len,
        }
    }
}

pub struct IntoIter<I: IndexType, T, const N: usize> {
    storage: TypedArray<I, MaybeUninit<T>, N>,
    index: I,
    len: I,
}

impl<I: IndexType, T, const N: usize> Iterator for IntoIter<I, T, N> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.len {
            let res = unsafe { self.storage.get_unchecked(self.index).assume_init_read() };
            self.index = unsafe { self.index.unchecked_add_scalar(I::Scalar::ONE) };
            Some(res)
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = unsafe { self.len.unchecked_sub_index(self.index).to_usize() };
        (remaining, Some(remaining))
    }
}

impl<I: IndexType, T, const N: usize> DoubleEndedIterator for IntoIter<I, T, N> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.index < self.len {
            self.len = self.len.checked_sub_scalar(I::Scalar::ONE).unwrap();
            let res = unsafe { self.storage.get_unchecked(self.len).assume_init_read() };
            Some(res)
        } else {
            None
        }
    }
}

impl<I: IndexType, T, const N: usize> ExactSizeIterator for IntoIter<I, T, N> {}
impl<I: IndexType, T, const N: usize> core::iter::FusedIterator for IntoIter<I, T, N> {}

impl<I: IndexType, T, const N: usize> Drop for IntoIter<I, T, N> {
    fn drop(&mut self) {
        // Drop remaining elements manually.
        let remaining_ptr = unsafe {
            self.storage
                .as_mut_ptr()
                .add(self.index.to_raw_index())
                .cast::<T>()
        };
        let remaining_len = unsafe { self.len.unchecked_sub_index(self.index).to_usize() };
        // SAFETY: These elements are still initialized and have not been moved out.
        unsafe {
            core::ptr::drop_in_place(core::slice::from_raw_parts_mut(
                remaining_ptr,
                remaining_len,
            ));
        }
    }
}

impl<'a, I: IndexType, T, const N: usize> IntoIterator for &'a TypedArrayVec<I, T, N> {
    type Item = &'a T;
    type IntoIter = core::slice::Iter<'a, T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.as_slice().iter()
    }
}

impl<'a, I: IndexType, T, const N: usize> IntoIterator for &'a mut TypedArrayVec<I, T, N> {
    type Item = &'a mut T;
    type IntoIter = core::slice::IterMut<'a, T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.as_mut_slice().iter_mut()
    }
}

impl<I: IndexType, T, const N: usize> Extend<T> for TypedArrayVec<I, T, N> {
    fn extend<X: IntoIterator<Item = T>>(&mut self, iter: X) {
        for item in iter {
            self.push(item);
        }
    }
}

impl<I: IndexType, T, const N: usize> FromIterator<T> for TypedArrayVec<I, T, N> {
    fn from_iter<X: IntoIterator<Item = T>>(iter: X) -> Self {
        let mut new = Self::new();
        new.extend(iter);
        new
    }
}

impl<I: IndexType, T, const N: usize> From<crate::typed_array::TypedArray<I, T, N>>
    for TypedArrayVec<I, T, N>
{
    fn from(array: crate::typed_array::TypedArray<I, T, N>) -> Self {
        // Can't use `transmute` here since the types depend on generic, so we use `transmute_copy` and forget the original.
        let storage: TypedArray<I, MaybeUninit<T>, N> =
            unsafe { core::mem::transmute_copy(&array) };
        core::mem::forget(array);

        Self {
            storage: storage,
            len: unsafe { I::from_raw_index_unchecked(N) },
        }
    }
}

/// An error type used when a collection is at full capacity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CapacityError<T> {
    element: T,
}

impl<T> CapacityError<T> {
    /// Creates a new `CapacityError` with the element that could not be added.
    pub const fn new(element: T) -> Self {
        Self { element }
    }

    /// Returns the element that could not be added.
    pub fn element(self) -> T {
        self.element
    }
}

impl<T> core::fmt::Display for CapacityError<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "insufficient capacity")
    }
}

impl<T: core::fmt::Debug> core::error::Error for CapacityError<T> {}
