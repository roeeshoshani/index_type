//! A fixed-capacity, typed vector.
//!
//! This module provides [`TypedArrayVec`], which is a wrapper around a fixed-size array
//! that uses a custom [`IndexType`] for indexing and to store its length.

use core::{
    mem::MaybeUninit,
};

use crate::{IndexType, IndexScalarType, IndexTooBigError, typed_slice::TypedSlice};

/// A fixed-capacity, typed vector.
pub struct TypedArrayVec<I: IndexType, T, const N: usize> {
    storage: [MaybeUninit<T>; N],
    len: I,
}

impl<I: IndexType, T, const N: usize> TypedArrayVec<I, T, N> {
    const _ASSERT_CAPACITY_IN_INDEX_BOUNDS: () = if N > I::MAX_RAW_INDEX {
        panic!("capacity is not in bounds of the index type");
    };

    /// Creates a new, empty `TypedArrayVec`.
    #[inline]
    pub fn new() -> Self {
        let _ = Self::_ASSERT_CAPACITY_IN_INDEX_BOUNDS;
        Self {
            storage: unsafe { MaybeUninit::uninit().assume_init() },
            len: I::ZERO,
        }
    }

    /// Creates a new, empty `TypedArrayVec` (const fn).
    #[inline]
    pub const fn new_const() -> Self {
        let _ = Self::_ASSERT_CAPACITY_IN_INDEX_BOUNDS;
        Self {
            storage: unsafe { MaybeUninit::uninit().assume_init() },
            len: I::ZERO,
        }
    }

    /// Returns the number of elements in the `TypedArrayVec` as a scalar.
    #[inline]
    pub fn len(&self) -> I::Scalar {
        self.len.to_scalar()
    }

    /// Returns the number of elements in the `TypedArrayVec` as an index.
    #[inline]
    pub fn len_as_index(&self) -> I {
        self.len
    }

    /// Returns the capacity of the `TypedArrayVec` as a scalar.
    #[inline]
    pub fn capacity(&self) -> I::Scalar {
        // SAFETY: The capacity N is guaranteed to be in bounds for I by the type system and our check.
        unsafe { I::Scalar::from_usize_unchecked(N) }
    }

    /// Returns true if the `TypedArrayVec` is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len == I::ZERO
    }

    /// Returns true if the `TypedArrayVec` is full.
    #[inline]
    pub fn is_full(&self) -> bool {
        self.len.to_raw_index() == N
    }

    /// Returns a `TypedSlice` of the elements that have been initialized.
    #[inline]
    pub fn as_typed_slice(&self) -> &TypedSlice<I, T> {
        self.as_slice()
    }

    /// Returns a mutable `TypedSlice` of the elements that have been initialized.
    #[inline]
    pub fn as_mut_typed_slice(&mut self) -> &mut TypedSlice<I, T> {
        self.as_mut_slice()
    }

    /// Returns the `TypedArrayVec` as a `TypedSlice`.
    #[inline]
    pub fn as_slice(&self) -> &TypedSlice<I, T> {
        // SAFETY: The storage is initialized up to self.len.
        unsafe {
            let slice = core::slice::from_raw_parts(self.storage.as_ptr().cast(), self.len.to_raw_index());
            TypedSlice::from_slice_unchecked(slice)
        }
    }

    /// Returns the `TypedArrayVec` as a mutable `TypedSlice`.
    #[inline]
    pub fn as_mut_slice(&mut self) -> &mut TypedSlice<I, T> {
        // SAFETY: The storage is initialized up to self.len.
        unsafe {
            let slice = core::slice::from_raw_parts_mut(self.storage.as_mut_ptr().cast(), self.len.to_raw_index());
            TypedSlice::from_slice_unchecked_mut(slice)
        }
    }

    /// Appends an element to the back of the `TypedArrayVec`.
    ///
    /// # Panics
    ///
    /// Panics if the `TypedArrayVec` is full.
    #[inline]
    pub fn push(&mut self, element: T) {
        if let Err(err) = self.try_push(element) {
            panic!("{}", err);
        }
    }

    /// Tries to append an element to the back of the `TypedArrayVec`.
    ///
    /// Returns an error if the `TypedArrayVec` is full.
    #[inline]
    pub fn try_push(&mut self, element: T) -> Result<(), crate::GenericIndexTooBigError> {
        if self.is_full() {
            return Err(<crate::GenericIndexTooBigError as IndexTooBigError>::new());
        }
        // SAFETY: The capacity is not exceeded.
        unsafe {
            self.storage.get_unchecked_mut(self.len.to_raw_index()).write(element);
            self.len = self.len.unchecked_add_scalar(I::Scalar::ONE);
        }
        Ok(())
    }

    /// Removes the last element from the `TypedArrayVec` and returns it, or `None` if it is empty.
    #[inline]
    pub fn pop(&mut self) -> Option<T> {
        if self.is_empty() {
            return None;
        }
        // SAFETY: The vector is not empty.
        unsafe {
            let new_len_raw = self.len.to_raw_index() - 1;
            self.len = I::from_raw_index_unchecked(new_len_raw);
            Some(self.storage.get_unchecked(new_len_raw).assume_init_read())
        }
    }

    /// Clears the `TypedArrayVec`, removing all elements.
    #[inline]
    pub fn clear(&mut self) {
        self.truncate(I::Scalar::ZERO);
    }

    /// Shortens the `TypedArrayVec`, keeping the first `len` elements and dropping the rest.
    #[inline]
    pub fn truncate(&mut self, len: I::Scalar) {
        let new_len = len.to_usize();
        let old_len = self.len.to_raw_index();
        if new_len < old_len {
            // SAFETY: storage is initialized up to old_len.
            unsafe {
                let tail = core::slice::from_raw_parts_mut(self.storage.as_mut_ptr().add(new_len).cast::<T>(), old_len - new_len);
                core::ptr::drop_in_place(tail);
                self.len = I::from_raw_index_unchecked(new_len);
            }
        }
    }

    /// Inserts an element at position `index` within the `TypedArrayVec`, shifting all elements after it to the right.
    ///
    /// # Panics
    ///
    /// Panics if the `TypedArrayVec` is full or if `index > len`.
    #[inline]
    pub fn insert(&mut self, index: I, element: T) {
        let idx = index.to_raw_index();
        let old_len = self.len.to_raw_index();
        assert!(idx <= old_len, "index out of bounds");
        assert!(old_len < N, "TypedArrayVec is full");

        // SAFETY: We checked bounds and capacity.
        unsafe {
            let p = self.storage.as_mut_ptr().add(idx).cast::<T>();
            core::ptr::copy(p, p.add(1), old_len - idx);
            core::ptr::write(p, element);
            self.len = I::from_raw_index_unchecked(old_len + 1);
        }
    }

    /// Removes and returns the element at position `index` within the `TypedArrayVec`, shifting all elements after it to the left.
    ///
    /// # Panics
    ///
    /// Panics if `index >= len`.
    #[inline]
    pub fn remove(&mut self, index: I) -> T {
        let idx = index.to_raw_index();
        let old_len = self.len.to_raw_index();
        assert!(idx < old_len, "index out of bounds");

        // SAFETY: We checked bounds.
        unsafe {
            let p = self.storage.as_mut_ptr().add(idx).cast::<T>();
            let result = core::ptr::read(p);
            core::ptr::copy(p.add(1), p, old_len - idx - 1);
            self.len = I::from_raw_index_unchecked(old_len - 1);
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
        let idx = index.to_raw_index();
        let old_len = self.len.to_raw_index();
        assert!(idx < old_len, "index out of bounds");

        // SAFETY: We checked bounds.
        unsafe {
            let result = self.storage.get_unchecked(idx).assume_init_read();
            let last = self.storage.get_unchecked(old_len - 1).assume_init_read();
            if idx < old_len - 1 {
                self.storage.get_unchecked_mut(idx).write(last);
            }
            self.len = I::from_raw_index_unchecked(old_len - 1);
            result
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
        let other_len = other.len().to_raw_index();
        let old_len = self.len.to_raw_index();
        assert!(old_len + other_len <= N, "TypedArrayVec capacity exceeded");

        for item in other.as_slice() {
            // SAFETY: We checked capacity.
            unsafe {
                self.storage.get_unchecked_mut(self.len.to_raw_index()).write(item.clone());
                self.len = I::from_raw_index_unchecked(self.len.to_raw_index() + 1);
            }
        }
    }

    /// Retains only the elements specified by the predicate.
    #[inline]
    pub fn retain<F>(&mut self, mut f: F)
    where
        F: FnMut(&T) -> bool,
    {
        let old_len = self.len.to_raw_index();
        let mut new_len = 0;
        for i in 0..old_len {
            // SAFETY: Elements are initialized up to old_len.
            let keep = unsafe { f(&*self.storage.get_unchecked(i).as_ptr()) };
            if keep {
                if i != new_len {
                    // SAFETY: Move element to new_len.
                    unsafe {
                        let src = self.storage.get_unchecked(i).as_ptr();
                        let dst = self.storage.get_unchecked_mut(new_len).as_mut_ptr();
                        core::ptr::copy_nonoverlapping(src, dst, 1);
                    }
                }
                new_len += 1;
            } else {
                // SAFETY: Drop element that is not kept.
                unsafe {
                    core::ptr::drop_in_place(self.storage.get_unchecked_mut(i).as_mut_ptr());
                }
            }
        }
        // SAFETY: new_len <= old_len which was valid for I.
        unsafe {
            self.len = I::from_raw_index_unchecked(new_len);
        }
    }

    /// Returns a draining iterator that removes the specified range in the vector and yields the removed items.
    #[inline]
    pub fn drain<R>(&mut self, range: R) -> Drain<'_, I, T, N>
    where
        R: core::ops::RangeBounds<I>,
    {
        let old_len = self.len.to_raw_index();
        let (start, end) = crate::utils::range_bounds_to_raw(range);
        let start = match start {
            core::ops::Bound::Included(i) => i,
            core::ops::Bound::Excluded(i) => i + 1,
            core::ops::Bound::Unbounded => 0,
        };
        let end = match end {
            core::ops::Bound::Included(i) => i + 1,
            core::ops::Bound::Excluded(i) => i,
            core::ops::Bound::Unbounded => old_len,
        };
        assert!(start <= end && end <= old_len, "drain range out of bounds");

        // SAFETY: We set the length to start, elements from start to end will be moved out by Drain.
        // Elements from end to old_len will be moved back after Drain is dropped.
        unsafe {
            self.len = I::from_raw_index_unchecked(start);
        }

        Drain {
            inner: self,
            index: start,
            end,
            old_len,
        }
    }
}

pub struct Drain<'a, I: IndexType, T, const N: usize> {
    inner: &'a mut TypedArrayVec<I, T, N>,
    index: usize,
    end: usize,
    old_len: usize,
}

impl<I: IndexType, T, const N: usize> Iterator for Drain<'_, I, T, N> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.end {
            let res = unsafe {
                self.inner.storage.get_unchecked(self.index).assume_init_read()
            };
            self.index += 1;
            Some(res)
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.end - self.index;
        (remaining, Some(remaining))
    }
}

impl<I: IndexType, T, const N: usize> Drop for Drain<'_, I, T, N> {
    fn drop(&mut self) {
        // Drop remaining elements in the range.
        while self.next().is_some() {}

        // Move the tail back.
        let tail_len = self.old_len - self.end;
        let head_len = self.inner.len.to_raw_index();
        if tail_len > 0 {
            unsafe {
                let src = self.inner.storage.as_ptr().add(self.end);
                let dst = self.inner.storage.as_mut_ptr().add(head_len);
                core::ptr::copy(src, dst, tail_len);
            }
        }
        // Update the length.
        unsafe {
            self.inner.len = I::from_raw_index_unchecked(head_len + tail_len);
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

impl<I: IndexType, T, const N: usize> core::ops::Index<I> for TypedArrayVec<I, T, N> {
    type Output = T;

    #[inline]
    fn index(&self, index: I) -> &Self::Output {
        &self.as_slice()[index]
    }
}

impl<I: IndexType, T, const N: usize> core::ops::IndexMut<I> for TypedArrayVec<I, T, N> {
    #[inline]
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        &mut self.as_mut_slice()[index]
    }
}

impl<I: IndexType, T: Clone, const N: usize> Clone for TypedArrayVec<I, T, N> {
    fn clone(&self) -> Self {
        let mut new = Self::new();
        for item in self.as_slice().as_slice() {
            new.push(item.clone());
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

impl<I: IndexType, T: core::fmt::Debug, const N: usize> core::fmt::Debug for TypedArrayVec<I, T, N> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::Debug::fmt(self.as_slice(), f)
    }
}

impl<I: IndexType, T: PartialEq, const N: usize> PartialEq for TypedArrayVec<I, T, N> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.as_slice() == other.as_slice()
    }
}

impl<I: IndexType, T: Eq, const N: usize> Eq for TypedArrayVec<I, T, N> {}

impl<I: IndexType, T: PartialOrd, const N: usize> PartialOrd for TypedArrayVec<I, T, N> {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        self.as_slice().as_slice().partial_cmp(other.as_slice().as_slice())
    }
}

impl<I: IndexType, T: Ord, const N: usize> Ord for TypedArrayVec<I, T, N> {
    #[inline]
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.as_slice().as_slice().cmp(other.as_slice().as_slice())
    }
}

impl<I: IndexType, T: core::hash::Hash, const N: usize> core::hash::Hash for TypedArrayVec<I, T, N> {
    #[inline]
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.as_slice().hash(state);
    }
}

impl<I: IndexType, T, const N: usize> IntoIterator for TypedArrayVec<I, T, N> {
    type Item = T;
    type IntoIter = IntoIter<I, T, N>;

    #[inline]
    fn into_iter(mut self) -> Self::IntoIter {
        let len = self.len.to_raw_index();
        // Set length to zero so that the original TypedArrayVec doesn't drop its elements.
        self.len = I::ZERO;
        IntoIter {
            inner: self,
            index: 0,
            len,
        }
    }
}

pub struct IntoIter<I: IndexType, T, const N: usize> {
    inner: TypedArrayVec<I, T, N>,
    index: usize,
    len: usize,
}

impl<I: IndexType, T, const N: usize> Iterator for IntoIter<I, T, N> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.len {
            let res = unsafe {
                self.inner.storage.get_unchecked(self.index).assume_init_read()
            };
            self.index += 1;
            Some(res)
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.len - self.index;
        (remaining, Some(remaining))
    }
}

impl<I: IndexType, T, const N: usize> Drop for IntoIter<I, T, N> {
    fn drop(&mut self) {
        // Drop remaining elements manually.
        let remaining_ptr = unsafe { self.inner.storage.as_mut_ptr().add(self.index).cast::<T>() };
        let remaining_len = self.len - self.index;
        // SAFETY: These elements are still initialized and have not been moved out.
        unsafe {
            core::ptr::drop_in_place(core::slice::from_raw_parts_mut(remaining_ptr, remaining_len));
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

impl<I: IndexType, T, const N: usize> From<crate::typed_array::TypedArray<I, T, N>> for TypedArrayVec<I, T, N> {
    fn from(array: crate::typed_array::TypedArray<I, T, N>) -> Self {
        let mut new = Self::new();
        for item in array {
            new.push(item);
        }
        new
    }
}
