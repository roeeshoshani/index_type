use core::marker::PhantomData;

use alloc::{boxed::Box, collections::TryReserveError, vec::Vec};

use crate::{
    IndexScalarType, IndexTooBigError, IndexType, typed_slice::TypedSlice,
    utils::range_bounds_to_raw,
};

#[repr(transparent)]
pub struct TypedVec<I: IndexType, T> {
    raw: Vec<T>,
    phantom: PhantomData<fn(&I)>,
}
impl<I: IndexType, T> TypedVec<I, T> {
    #[inline(always)]
    pub const fn new() -> Self {
        Self {
            raw: Vec::new(),
            phantom: PhantomData,
        }
    }

    #[inline]
    pub fn try_from_vec(vec: Vec<T>) -> Result<Self, IndexTooBigError> {
        let _ = I::try_from_raw_index(vec.len())?;
        let res = Self {
            raw: vec,
            phantom: PhantomData,
        };
        Ok(res)
    }

    #[inline]
    pub unsafe fn from_vec_unchecked(vec: Vec<T>) -> Self {
        Self {
            raw: vec,
            phantom: PhantomData,
        }
    }

    #[inline]
    pub unsafe fn from_raw_parts(
        ptr: *mut T,
        length: usize,
        capacity: usize,
    ) -> Result<Self, IndexTooBigError> {
        let _ = I::try_from_raw_index(length)?;
        Ok(Self {
            raw: unsafe { Vec::from_raw_parts(ptr, length, capacity) },
            phantom: PhantomData,
        })
    }

    #[inline]
    pub fn into_vec(self) -> Vec<T> {
        self.raw
    }

    #[inline(always)]
    pub fn len(&self) -> I::Scalar {
        unsafe { <I::Scalar as IndexScalarType>::from_usize_unchecked(self.raw.len()) }
    }

    #[inline(always)]
    pub fn len_as_index(&self) -> I {
        unsafe { I::from_raw_index_unchecked(self.raw.len()) }
    }

    #[inline(always)]
    pub fn capacity(&self) -> usize {
        self.raw.capacity()
    }

    #[inline]
    pub fn push(&mut self, value: T) -> Result<I, IndexTooBigError> {
        let res = self.len_as_index();
        let _new_len = res.checked_add_scalar(<I::Scalar as IndexScalarType>::ONE)?;
        self.raw.push(value);
        Ok(res)
    }

    #[inline]
    pub fn append(&mut self, other: &mut TypedVec<I, T>) -> Result<(), IndexTooBigError> {
        let _new_len = self.len_as_index().checked_add_scalar(other.len())?;
        self.raw.append(&mut other.raw);
        Ok(())
    }

    #[inline(always)]
    pub const fn as_mut_ptr(&mut self) -> *mut T {
        self.raw.as_mut_ptr()
    }

    #[inline(always)]
    pub fn reserve(&mut self, additional: usize) {
        self.raw.reserve(additional);
    }

    #[inline(always)]
    pub fn reserve_exact(&mut self, additional: usize) {
        self.raw.reserve_exact(additional)
    }

    #[inline(always)]
    pub fn try_reserve(&mut self, additional: usize) -> Result<(), TryReserveError> {
        self.raw.try_reserve(additional)
    }

    #[inline(always)]
    pub fn try_reserve_exact(&mut self, additional: usize) -> Result<(), TryReserveError> {
        self.raw.try_reserve_exact(additional)
    }

    #[inline(always)]
    pub fn shrink_to_fit(&mut self) {
        self.raw.shrink_to_fit();
    }

    #[inline(always)]
    pub fn shrink_to(&mut self, min_capacity: usize) {
        self.raw.shrink_to(min_capacity);
    }

    #[inline(always)]
    pub fn into_boxed_slice(self) -> Box<TypedSlice<I, T>> {
        unsafe { core::mem::transmute(self.raw.into_boxed_slice()) }
    }

    #[inline(always)]
    pub fn truncate(&mut self, len: I::Scalar) {
        self.raw.truncate(len.to_usize());
    }

    #[inline(always)]
    pub fn as_slice(&self) -> &TypedSlice<I, T> {
        unsafe { TypedSlice::from_slice_unchecked(self.raw.as_slice()) }
    }

    #[inline(always)]
    pub fn as_mut_slice(&mut self) -> &mut TypedSlice<I, T> {
        unsafe { TypedSlice::from_slice_unchecked_mut(self.raw.as_mut_slice()) }
    }

    #[inline(always)]
    pub const fn as_ptr(&self) -> *const T {
        self.raw.as_ptr()
    }

    #[inline(always)]
    pub unsafe fn set_len(&mut self, new_len: I) {
        unsafe { self.raw.set_len(new_len.to_scalar().to_usize()) };
    }

    #[inline(always)]
    pub fn swap_remove(&mut self, index: I) -> T {
        self.raw.swap_remove(index.to_raw_index())
    }

    #[inline(always)]
    pub fn insert(&mut self, index: I, element: T) -> Result<(), IndexTooBigError> {
        let _new_potential_len = index.checked_add_scalar(<I::Scalar as IndexScalarType>::ONE)?;
        self.raw.insert(index.to_raw_index(), element);
        Ok(())
    }

    #[inline(always)]
    pub fn remove(&mut self, index: I) -> T {
        self.raw.remove(index.to_raw_index())
    }

    #[inline(always)]
    pub fn retain<F>(&mut self, f: F)
    where
        F: FnMut(&T) -> bool,
    {
        self.raw.retain(f)
    }

    #[inline(always)]
    pub fn retain_mut<F>(&mut self, f: F)
    where
        F: FnMut(&mut T) -> bool,
    {
        self.raw.retain_mut(f)
    }

    #[inline(always)]
    pub fn dedup_by_key<F, K>(&mut self, key: F)
    where
        F: FnMut(&mut T) -> K,
        K: PartialEq,
    {
        self.raw.dedup_by_key(key);
    }

    #[inline(always)]
    pub fn dedup_by<F>(&mut self, same_bucket: F)
    where
        F: FnMut(&mut T, &mut T) -> bool,
    {
        self.raw.dedup_by(same_bucket);
    }

    #[inline(always)]
    pub fn pop(&mut self) -> Option<T> {
        self.raw.pop()
    }

    #[inline(always)]
    pub fn pop_if(&mut self, predicate: impl FnOnce(&mut T) -> bool) -> Option<T> {
        self.raw.pop_if(predicate)
    }

    #[inline(always)]
    pub fn clear(&mut self) {
        self.raw.clear();
    }

    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.raw.is_empty()
    }

    #[inline(always)]
    pub fn split_off(&mut self, at: I) -> Self {
        let new_vec = self.raw.split_off(at.to_raw_index());
        unsafe { Self::from_vec_unchecked(new_vec) }
    }

    #[inline(always)]
    pub fn resize_with<F>(&mut self, new_len: I, f: F)
    where
        F: FnMut() -> T,
    {
        self.raw.resize_with(new_len.to_scalar().to_usize(), f);
    }

    #[inline(always)]
    pub fn leak<'a>(self) -> &'a mut [T] {
        self.raw.leak()
    }

    #[inline(always)]
    pub fn drain<R>(&mut self, range: R) -> alloc::vec::Drain<'_, T>
    where
        R: core::ops::RangeBounds<I>,
    {
        self.raw.drain(range_bounds_to_raw(range))
    }

    #[inline(always)]
    pub fn splice<R, X>(&mut self, range: R, replace_with: X) -> alloc::vec::Splice<'_, X::IntoIter>
    where
        R: core::ops::RangeBounds<I>,
        X: IntoIterator<Item = T>,
    {
        self.raw.splice(range_bounds_to_raw(range), replace_with)
    }
}
impl<I: IndexType, T: PartialEq> TypedVec<I, T> {
    #[inline(always)]
    pub fn dedup(&mut self) {
        self.raw.dedup();
    }
}
impl<I: IndexType, T: Clone> TypedVec<I, T> {
    #[inline(always)]
    pub fn extend_from_slice(&mut self, other: &TypedSlice<I, T>) {
        self.raw.extend_from_slice(other.to_slice())
    }

    #[inline(always)]
    pub fn extend_from_within<R>(&mut self, src: R)
    where
        R: core::ops::RangeBounds<I>,
    {
        self.raw.extend_from_within(range_bounds_to_raw(src));
    }

    #[inline(always)]
    pub fn extract_if<F, R>(&mut self, range: R, filter: F) -> alloc::vec::ExtractIf<'_, T, F>
    where
        F: FnMut(&mut T) -> bool,
        R: core::ops::RangeBounds<I>,
    {
        self.raw.extract_if(range_bounds_to_raw(range), filter)
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
