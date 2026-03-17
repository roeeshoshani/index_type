use core::{marker::PhantomData, ptr::NonNull};

use alloc::{boxed::Box, collections::TryReserveError, vec::Vec};
use thiserror_no_std::Error;

use crate::{IndexTooBigError, IndexType, typed_slice::TypedSlice};

#[derive(Debug, Error)]
pub enum TypedVecTryReserveError {
    #[error(transparent)]
    TryReserveError(#[from] TryReserveError),

    #[error(transparent)]
    IndexTooBigError(#[from] IndexTooBigError),
}

pub struct TypedVec<I: IndexType, T> {
    raw: Vec<T>,
    phantom: PhantomData<fn(&I)>,
}
impl<I: IndexType, T> TypedVec<I, T> {
    pub const fn new() -> Self {
        Self {
            raw: Vec::new(),
            phantom: PhantomData,
        }
    }

    fn check_len_in_bounds(&self) -> Result<(), IndexTooBigError> {
        let _ = I::try_from_index(self.raw.len())?;
        Ok(())
    }

    pub fn try_from_vec(vec: Vec<T>) -> Result<Self, IndexTooBigError> {
        let res = Self {
            raw: vec,
            phantom: PhantomData,
        };
        res.check_len_in_bounds()?;
        Ok(res)
    }

    pub unsafe fn from_vec_unchecked(vec: Vec<T>) -> Self {
        Self {
            raw: vec,
            phantom: PhantomData,
        }
    }

    pub fn into_vec(self) -> Vec<T> {
        self.raw
    }

    pub fn push(&mut self, value: T) -> Result<I, IndexTooBigError> {
        let _ = self.len().checked_add_usize(1)?;
        let res = self.len();
        self.raw.push(value);
        Ok(res)
    }

    pub fn append(&mut self, other: &mut TypedVec<I, T>) -> Result<(), IndexTooBigError> {
        // let _ = self.len().checked_add_usize(rhs)
        self.append(other)
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
        self.raw.capacity()
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

    pub fn resize_with<F>(&mut self, new_len: usize, f: F) -> Result<(), IndexTooBigError>
    where
        F: FnMut() -> T,
    {
        self.modify_as_vec(|v| {
            v.resize_with(new_len, f);
        })
    }

    pub fn leak<'a>(self) -> &'a mut [T] {
        self.into_vec().leak()
    }
}
impl<I: IndexType, T: core::fmt::Debug> core::fmt::Debug for TypedVec<I, T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::Debug::fmt(&self.raw, f)
    }
}
impl<I: IndexType, T: PartialEq> PartialEq for TypedVec<I, T> {
    fn eq(&self, other: &Self) -> bool {
        self.raw == other.raw
    }
}
impl<I: IndexType, T: Eq> Eq for TypedVec<I, T> {}
impl<I: IndexType, T: Clone> Clone for TypedVec<I, T> {
    fn clone(&self) -> Self {
        Self {
            raw: self.raw.clone(),
            phantom: PhantomData,
        }
    }
}
impl<I: IndexType, T: core::hash::Hash> core::hash::Hash for TypedVec<I, T> {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.raw.hash(state);
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
