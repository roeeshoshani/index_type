use core::{mem::MaybeUninit, ptr::NonNull};

use alloc::{boxed::Box, collections::TryReserveError, vec::Vec};
use thiserror_no_std::Error;
use uniq::Unique;

use crate::{IndexTooBigError, IndexType, typed_slice::TypedSlice};

#[derive(Debug, Error)]
pub enum TypedVecTryReserveError {
    #[error(transparent)]
    TryReserveError(#[from] TryReserveError),

    #[error(transparent)]
    IndexTooBigError(#[from] IndexTooBigError),
}

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

    pub unsafe fn from_vec_unchecked(vec: Vec<T>) -> Self {
        let (new_ptr, new_len, new_cap) = vec.into_raw_parts();
        Self {
            ptr: unsafe {
                // SAFETY: the pointer of a vec is never null. it is stored internally as a non-null pointer.
                Unique::new_unchecked(new_ptr)
            },
            len: unsafe { I::from_index_unchecked(new_len) },
            cap: unsafe { I::from_index_unchecked(new_cap) },
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
        *self = Self::try_from_vec(vec)?;
        Ok(res)
    }

    pub unsafe fn modify_as_vec_unchecked<F, R>(&mut self, f: F) -> R
    where
        F: FnOnce(&mut Vec<T>) -> R,
    {
        let mut vec = core::mem::take(self).into_vec();
        let res = f(&mut vec);
        *self = unsafe { Self::from_vec_unchecked(vec) };
        res
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

    pub const fn capacity(&self) -> I {
        self.cap
    }

    pub fn reserve(&mut self, additional: usize) -> Result<(), IndexTooBigError> {
        self.modify_as_vec(|v| {
            v.reserve(additional);
        })
    }

    pub fn reserve_exact(&mut self, additional: usize) -> Result<(), IndexTooBigError> {
        self.modify_as_vec(|v| {
            v.reserve_exact(additional);
        })
    }

    pub fn try_reserve(&mut self, additional: usize) -> Result<(), TypedVecTryReserveError> {
        Ok(self.modify_as_vec(|v| v.try_reserve(additional))??)
    }

    pub fn try_reserve_exact(&mut self, additional: usize) -> Result<(), TypedVecTryReserveError> {
        Ok(self.modify_as_vec(|v| v.try_reserve_exact(additional))??)
    }

    pub fn shrink_to_fit(&mut self) {
        unsafe {
            self.modify_as_vec_unchecked(|v| {
                v.shrink_to_fit();
            })
        }
    }

    pub fn shrink_to(&mut self, min_capacity: I) {
        unsafe {
            self.modify_as_vec_unchecked(|v| {
                v.shrink_to(min_capacity.to_index());
            })
        }
    }

    pub fn into_boxed_slice(self) -> Box<TypedSlice<I, T>> {
        unsafe { core::mem::transmute(self.into_vec().into_boxed_slice()) }
    }

    pub fn truncate(&mut self, len: I) {
        unsafe {
            self.modify_as_vec_unchecked(|v| {
                v.truncate(len.to_index());
            })
        }
    }

    pub fn as_slice(&self) -> &TypedSlice<I, T> {
        unsafe { TypedSlice::from_raw_parts(self.ptr.as_ptr(), self.len) }
    }

    pub fn as_mut_slice(&mut self) -> &mut TypedSlice<I, T> {
        unsafe { TypedSlice::from_raw_parts_mut(self.ptr.as_ptr(), self.len) }
    }

    pub const fn as_ptr(&self) -> *const T {
        self.ptr.as_ptr()
    }

    pub const fn as_non_null(&mut self) -> NonNull<T> {
        self.ptr.as_non_null_ptr()
    }

    pub unsafe fn set_len(&mut self, new_len: I) -> Result<(), IndexTooBigError> {
        self.modify_as_vec(|v| {
            unsafe { v.set_len(new_len.to_index()) };
        })
    }

    pub fn swap_remove(&mut self, index: I) -> T {
        unsafe { self.modify_as_vec_unchecked(|v| v.swap_remove(index.to_index())) }
    }

    pub fn insert(&mut self, index: I, element: T) {
        unsafe { self.modify_as_vec_unchecked(|v| v.insert(index.to_index(), element)) }
    }

    pub fn remove(&mut self, index: I) -> T {
        unsafe { self.modify_as_vec_unchecked(|v| v.remove(index.to_index())) }
    }

    pub fn retain<F>(&mut self, f: F)
    where
        F: FnMut(&T) -> bool,
    {
        unsafe { self.modify_as_vec_unchecked(|v| v.retain(f)) }
    }

    pub fn retain_mut<F>(&mut self, f: F)
    where
        F: FnMut(&mut T) -> bool,
    {
        unsafe { self.modify_as_vec_unchecked(|v| v.retain_mut(f)) }
    }

    pub fn dedup_by_key<F, K>(&mut self, key: F)
    where
        F: FnMut(&mut T) -> K,
        K: PartialEq,
    {
        unsafe { self.modify_as_vec_unchecked(|v| v.dedup_by_key(key)) }
    }

    pub fn dedup_by<F>(&mut self, same_bucket: F)
    where
        F: FnMut(&mut T, &mut T) -> bool,
    {
        unsafe { self.modify_as_vec_unchecked(|v| v.dedup_by(same_bucket)) }
    }

    pub fn pop(&mut self) -> Option<T> {
        unsafe { self.modify_as_vec_unchecked(|v| v.pop()) }
    }

    pub fn pop_if(&mut self, predicate: impl FnOnce(&mut T) -> bool) -> Option<T> {
        unsafe { self.modify_as_vec_unchecked(|v| v.pop_if(predicate)) }
    }

    pub fn clear(&mut self) {
        unsafe { self.modify_as_vec_unchecked(|v| v.clear()) }
    }

    pub const fn len(&self) -> I {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == I::ZERO
    }

    pub fn split_off(&mut self, at: I) -> Self {
        let raw_vec = unsafe { self.modify_as_vec_unchecked(|v| v.split_off(at.to_index())) };
        unsafe { Self::from_vec_unchecked(raw_vec) }
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

    pub fn into_chunks<const N: usize>(mut self) -> TypedVec<I, [T; N]> {
        todo!()
    }

    pub fn recycle<U>(mut self) -> TypedVec<I, U> {
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
