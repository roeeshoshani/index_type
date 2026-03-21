use core::{
    marker::PhantomData,
    ops::{Index, IndexMut},
};

use crate::{
    IndexTooBigError, IndexType,
    typed_slice::{TypedSlice, TypedSliceIndex},
};

#[repr(transparent)]
pub struct TypedArray<I: IndexType, T, const SIZE: usize> {
    raw: [T; SIZE],
    phantom: PhantomData<fn(&I)>,
}

impl<I: IndexType, T, const SIZE: usize> TypedArray<I, T, SIZE> {
    #[inline]
    pub const fn as_slice(&self) -> &TypedSlice<I, T> {
        unsafe { TypedSlice::from_slice_unchecked(&self.raw) }
    }

    #[inline]
    pub const fn as_mut_slice(&mut self) -> &mut TypedSlice<I, T> {
        unsafe { TypedSlice::from_slice_unchecked_mut(&mut self.raw) }
    }

    #[inline]
    pub fn as_array(&self) -> &[T; SIZE] {
        &self.raw
    }

    #[inline]
    pub fn as_mut_array(&mut self) -> &mut [T; SIZE] {
        &mut self.raw
    }

    #[inline]
    pub fn into_array(self) -> [T; SIZE] {
        self.raw
    }

    #[inline]
    pub fn try_from_array(raw: [T; SIZE]) -> Result<Self, IndexTooBigError> {
        let _ = I::try_from_raw_index(SIZE)?;
        Ok(TypedArray {
            raw,
            phantom: PhantomData,
        })
    }

    #[inline]
    pub fn try_from_array_ref(
        raw: &[T; SIZE],
    ) -> Result<&TypedArray<I, T, SIZE>, IndexTooBigError> {
        let _ = I::try_from_raw_index(SIZE)?;
        Ok(unsafe { core::mem::transmute::<&[T; SIZE], &TypedArray<I, T, SIZE>>(raw) })
    }

    #[inline]
    pub fn try_from_array_mut(
        raw: &mut [T; SIZE],
    ) -> Result<&mut TypedArray<I, T, SIZE>, IndexTooBigError> {
        let _ = I::try_from_raw_index(SIZE)?;
        Ok(unsafe { core::mem::transmute::<&mut [T; SIZE], &mut TypedArray<I, T, SIZE>>(raw) })
    }

    #[inline]
    pub const unsafe fn from_array_unchecked(raw: [T; SIZE]) -> Self {
        TypedArray {
            raw,
            phantom: PhantomData,
        }
    }

    #[inline]
    pub const unsafe fn from_array_ref_unchecked(raw: &[T; SIZE]) -> &TypedArray<I, T, SIZE> {
        unsafe { core::mem::transmute::<&[T; SIZE], &TypedArray<I, T, SIZE>>(raw) }
    }

    #[inline]
    pub unsafe fn from_array_mut_unchecked(raw: &mut [T; SIZE]) -> &mut TypedArray<I, T, SIZE> {
        unsafe { core::mem::transmute::<&mut [T; SIZE], &mut TypedArray<I, T, SIZE>>(raw) }
    }

    #[inline]
    pub const fn each_ref(&self) -> TypedArray<I, &T, SIZE> {
        let refs = self.raw.each_ref();
        unsafe { TypedArray::from_array_unchecked(refs) }
    }

    #[inline]
    pub fn each_mut(&mut self) -> TypedArray<I, &mut T, SIZE> {
        let refs = self.raw.each_mut();
        unsafe { TypedArray::from_array_unchecked(refs) }
    }
}

impl<I: IndexType, T: PartialEq, const SIZE: usize> PartialEq for TypedArray<I, T, SIZE> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.raw == other.raw
    }
}

impl<I: IndexType, T: Eq, const SIZE: usize> Eq for TypedArray<I, T, SIZE> {}

impl<I: IndexType, T: PartialOrd, const SIZE: usize> PartialOrd for TypedArray<I, T, SIZE> {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        self.raw.partial_cmp(&other.raw)
    }
}

impl<I: IndexType, T: Ord, const SIZE: usize> Ord for TypedArray<I, T, SIZE> {
    #[inline]
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.raw.cmp(&other.raw)
    }
}

impl<I: IndexType, T: core::hash::Hash, const SIZE: usize> core::hash::Hash
    for TypedArray<I, T, SIZE>
{
    #[inline]
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.raw.hash(state);
    }
}

impl<I: IndexType, T: core::fmt::Debug, const SIZE: usize> core::fmt::Debug
    for TypedArray<I, T, SIZE>
{
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.raw.fmt(f)
    }
}

impl<I: IndexType, T: Clone, const SIZE: usize> Clone for TypedArray<I, T, SIZE> {
    #[inline]
    fn clone(&self) -> Self {
        TypedArray {
            raw: self.raw.clone(),
            phantom: PhantomData,
        }
    }
}

impl<I: IndexType, T: Clone, const SIZE: usize> TypedArray<I, T, SIZE> {
    #[inline]
    pub fn map<U, F: FnMut(T) -> U>(self, mut f: F) -> TypedArray<I, U, SIZE> {
        let mapped = core::array::from_fn(|i| f(unsafe { core::ptr::read(&self.raw[i]) }));
        TypedArray {
            raw: mapped,
            phantom: PhantomData,
        }
    }

    #[inline]
    pub fn try_map<U, F: FnMut(T) -> Result<U, E>, E>(
        self,
        mut f: F,
    ) -> Result<TypedArray<I, U, SIZE>, E> {
        let mut result = core::array::from_fn(|_| None);
        for (i, elem) in self.raw.into_iter().enumerate() {
            result[i] = Some(f(elem)?);
        }
        let mapped = unsafe { result.map(|x| x.unwrap_unchecked()) };
        Ok(TypedArray {
            raw: mapped,
            phantom: PhantomData,
        })
    }
}

impl<I: IndexType, T: Default, const SIZE: usize> Default for TypedArray<I, T, SIZE> {
    #[inline]
    fn default() -> Self {
        // Using array initialization that works with non-Copy types
        TypedArray {
            raw: core::array::from_fn(|_| T::default()),
            phantom: PhantomData,
        }
    }
}

impl<I: IndexType, T, const SIZE: usize, X: TypedSliceIndex<TypedSlice<I, T>>> Index<X>
    for TypedArray<I, T, SIZE>
{
    type Output = X::Output;

    #[inline]
    fn index(&self, index: X) -> &Self::Output {
        index.index(unsafe { TypedSlice::from_slice_unchecked(&self.raw) })
    }
}

impl<I: IndexType, T, const SIZE: usize, X: TypedSliceIndex<TypedSlice<I, T>>> IndexMut<X>
    for TypedArray<I, T, SIZE>
{
    #[inline]
    fn index_mut(&mut self, index: X) -> &mut Self::Output {
        index.index_mut(unsafe { TypedSlice::from_slice_unchecked_mut(&mut self.raw) })
    }
}

impl<I: IndexType, T, const SIZE: usize> AsRef<[T]> for TypedArray<I, T, SIZE> {
    #[inline]
    fn as_ref(&self) -> &[T] {
        &self.raw
    }
}

impl<I: IndexType, T, const SIZE: usize> AsMut<[T]> for TypedArray<I, T, SIZE> {
    #[inline]
    fn as_mut(&mut self) -> &mut [T] {
        &mut self.raw
    }
}

impl<I: IndexType, T, const SIZE: usize> core::borrow::Borrow<[T]> for TypedArray<I, T, SIZE> {
    #[inline]
    fn borrow(&self) -> &[T] {
        &self.raw
    }
}

impl<I: IndexType, T, const SIZE: usize> core::borrow::BorrowMut<[T]> for TypedArray<I, T, SIZE> {
    #[inline]
    fn borrow_mut(&mut self) -> &mut [T] {
        &mut self.raw
    }
}

impl<'a, I: IndexType, T, const SIZE: usize> IntoIterator for &'a TypedArray<I, T, SIZE> {
    type Item = &'a T;
    type IntoIter = core::slice::Iter<'a, T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.raw.iter()
    }
}

impl<'a, I: IndexType, T, const SIZE: usize> IntoIterator for &'a mut TypedArray<I, T, SIZE> {
    type Item = &'a mut T;
    type IntoIter = core::slice::IterMut<'a, T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.raw.iter_mut()
    }
}

impl<I: IndexType, T, const SIZE: usize> IntoIterator for TypedArray<I, T, SIZE> {
    type Item = T;
    type IntoIter = core::array::IntoIter<T, SIZE>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.raw.into_iter()
    }
}
