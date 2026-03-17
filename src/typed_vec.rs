use alloc::vec::Vec;
use uniq::Unique;

use crate::{IndexTooBigError, IndexType};

pub struct TypedVec<I: IndexType, T> {
    ptr: Unique<T>,
    len: I,
    cap: I,
}
impl<I: IndexType, T> TypedVec<I, T> {
    pub const fn new() -> Self {
        Self {
            ptr: Unique::dangling(),
            len: I::ZERO,
            cap: I::ZERO,
        }
    }

    pub fn into_vec(self) -> Vec<T> {
        unsafe { Vec::from_raw_parts(self.ptr.as_ptr(), self.len.to_index(), self.cap.to_index()) }
    }

    pub fn modify_as_vec<F, R>(&mut self, f: F) -> Result<R, IndexTooBigError>
    where
        F: FnOnce(&mut Vec<T>) -> R,
    {
        let mut vec = core::mem::take(self).into_vec();
        let res = f(&mut vec);
        let (new_ptr, new_len, new_cap) = vec.into_raw_parts();
        *self = TypedVec {
            ptr: unsafe {
                // SAFETY: the pointer of a vec is never null. it is stored internally as a non-null pointer.
                Unique::new_unchecked(new_ptr)
            },
            len: I::try_from_index(new_len)?,
            cap: I::try_from_index(new_cap)?,
        };
        Ok(res)
    }

    pub fn push(&mut self, value: T) -> Result<I, IndexTooBigError> {
        let res = self.len;
        self.modify_as_vec(|v| {
            v.push(value);
        })?;
        Ok(res)
    }

    pub fn append(&mut self, other: &mut TypedVec<I, T>) -> Result<(), IndexTooBigError> {
        self.modify_as_vec(|self_vec| {
            other.modify_as_vec(|other_vec| {
                self_vec.append(other_vec);
            })
        })?
    }

    pub const fn as_mut_ptr(&mut self) -> *mut T {
        self.ptr.as_ptr()
    }

    pub const fn capacity(&self) -> usize {
        todo!()
    }
    pub fn reserve(&mut self, additional: usize) {
        todo!()
    }
    pub fn reserve_exact(&mut self, additional: usize) {
        todo!()
    }
    pub fn try_reserve(&mut self, additional: usize) -> Result<(), TryReserveError> {
        todo!()
    }
    pub fn try_reserve_exact(&mut self, additional: usize) -> Result<(), TryReserveError> {
        todo!()
    }
    pub fn shrink_to_fit(&mut self) {
        todo!()
    }
    pub fn shrink_to(&mut self, min_capacity: usize) {
        todo!()
    }
    pub fn into_boxed_slice(mut self) -> Box<[T], A> {
        todo!()
    }
    pub fn truncate(&mut self, len: usize) {
        todo!()
    }
    pub const fn as_slice(&self) -> &[T] {
        todo!()
    }
    pub const fn as_mut_slice(&mut self) -> &mut [T] {
        todo!()
    }
    pub const fn as_ptr(&self) -> *const T {
        todo!()
    }
    pub const fn as_non_null(&mut self) -> NonNull<T> {
        todo!()
    }
    pub fn allocator(&self) -> &A {
        todo!()
    }
    pub unsafe fn set_len(&mut self, new_len: usize) {
        todo!()
    }
    pub fn swap_remove(&mut self, index: usize) -> T {
        todo!()
    }
    pub fn insert(&mut self, index: usize, element: T) {
        todo!()
    }
    pub fn insert_mut(&mut self, index: usize, element: T) -> &mut T {
        todo!()
    }
    pub fn remove(&mut self, index: usize) -> T {
        todo!()
    }
    pub fn try_remove(&mut self, index: usize) -> Option<T> {
        todo!()
    }
    pub fn retain<F>(&mut self, mut f: F) {
        todo!()
    }
    pub fn retain_mut<F>(&mut self, mut f: F) {
        todo!()
    }
    pub fn dedup_by_key<F, K>(&mut self, mut key: F) {
        todo!()
    }
    pub fn dedup_by<F>(&mut self, mut same_bucket: F) {
        todo!()
    }
    pub fn push_within_capacity(&mut self, value: T) -> Result<&mut T, T> {
        todo!()
    }
    pub fn push_mut(&mut self, value: T) -> &mut T {
        todo!()
    }
    pub fn pop(&mut self) -> Option<T> {
        todo!()
    }
    pub fn pop_if(&mut self, predicate: impl FnOnce(&mut T) -> bool) -> Option<T> {
        todo!()
    }
    pub fn peek_mut(&mut self) -> Option<PeekMut<'_, T, A>> {
        todo!()
    }
    pub fn drain<R>(&mut self, range: R) -> Drain<'_, T, A> {
        todo!()
    }
    pub fn clear(&mut self) {
        todo!()
    }
    pub const fn len(&self) -> usize {
        todo!()
    }
    pub const fn is_empty(&self) -> bool {
        todo!()
    }
    pub fn split_off(&mut self, at: usize) -> Self {
        todo!()
    }
    pub fn resize_with<F>(&mut self, new_len: usize, f: F) {
        todo!()
    }
    pub fn leak<'a>(self) -> &'a mut [T] {
        todo!()
    }
    pub fn spare_capacity_mut(&mut self) -> &mut [MaybeUninit<T>] {
        todo!()
    }
    pub fn split_at_spare_mut(&mut self) -> (&mut [T], &mut [MaybeUninit<T>]) {
        todo!()
    }
    pub fn into_chunks<const N: usize>(mut self) -> Vec<[T; N], A> {
        todo!()
    }
    pub fn recycle<U>(mut self) -> Vec<U, A> {
        todo!()
    }
}
impl<I: IndexType, T> Drop for TypedVec<I, T> {
    fn drop(&mut self) {
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
