use core::{
    array::TryFromSliceError,
    marker::PhantomData,
    ops::{Index, IndexMut},
};

use crate::{IndexTooBigError, IndexType, typed_slice::TypedSlice};

#[repr(transparent)]
pub struct TypedArray<I: IndexType, T, const N: usize> {
    raw: [T; N],
    phantom: PhantomData<fn(&I)>,
}

impl<I: IndexType, T, const N: usize> TypedArray<I, T, N> {
    #[inline]
    pub const fn as_slice(&self) -> &TypedSlice<I, T> {
        unsafe { TypedSlice::from_slice_unchecked(&self.raw) }
    }

    #[inline]
    pub const fn as_mut_slice(&mut self) -> &mut TypedSlice<I, T> {
        unsafe { TypedSlice::from_slice_unchecked_mut(&mut self.raw) }
    }

    #[inline]
    pub const fn as_array(&self) -> &[T; N] {
        &self.raw
    }

    #[inline]
    pub const fn as_mut_array(&mut self) -> &mut [T; N] {
        &mut self.raw
    }

    #[inline]
    pub fn into_array(self) -> [T; N] {
        self.raw
    }

    #[inline]
    pub fn try_from_array(array: [T; N]) -> Result<Self, IndexTooBigError> {
        let _ = I::try_from_raw_index(N)?;
        Ok(TypedArray {
            raw: array,
            phantom: PhantomData,
        })
    }

    #[inline]
    pub fn try_from_array_ref(array: &[T; N]) -> Result<&TypedArray<I, T, N>, IndexTooBigError> {
        let _ = I::try_from_raw_index(N)?;
        Ok(unsafe { core::mem::transmute::<&[T; N], &TypedArray<I, T, N>>(array) })
    }

    #[inline]
    pub fn try_from_array_mut(
        array: &mut [T; N],
    ) -> Result<&mut TypedArray<I, T, N>, IndexTooBigError> {
        let _ = I::try_from_raw_index(N)?;
        Ok(unsafe { core::mem::transmute::<&mut [T; N], &mut TypedArray<I, T, N>>(array) })
    }

    #[inline]
    pub const unsafe fn from_array_unchecked(array: [T; N]) -> Self {
        TypedArray {
            raw: array,
            phantom: PhantomData,
        }
    }

    #[inline]
    pub const unsafe fn from_array_ref_unchecked(array: &[T; N]) -> &TypedArray<I, T, N> {
        unsafe { core::mem::transmute::<&[T; N], &TypedArray<I, T, N>>(array) }
    }

    #[inline]
    pub const unsafe fn from_array_mut_unchecked(array: &mut [T; N]) -> &mut TypedArray<I, T, N> {
        unsafe { core::mem::transmute::<&mut [T; N], &mut TypedArray<I, T, N>>(array) }
    }

    #[inline]
    pub const fn each_ref(&self) -> TypedArray<I, &T, N> {
        let refs = self.raw.each_ref();
        unsafe { TypedArray::from_array_unchecked(refs) }
    }

    #[inline]
    pub fn each_mut(&mut self) -> TypedArray<I, &mut T, N> {
        let refs = self.raw.each_mut();
        unsafe { TypedArray::from_array_unchecked(refs) }
    }

    #[inline]
    pub fn map<U, F: FnMut(T) -> U>(self, f: F) -> TypedArray<I, U, N> {
        unsafe { TypedArray::from_array_unchecked(self.raw.map(f)) }
    }
}

impl<I: IndexType, T: PartialEq, const N: usize> PartialEq for TypedArray<I, T, N> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        PartialEq::eq(&self.raw, &other.raw)
    }
}

impl<I: IndexType, T: Eq, const N: usize> Eq for TypedArray<I, T, N> {}

impl<I: IndexType, T: PartialOrd, const N: usize> PartialOrd for TypedArray<I, T, N> {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        self.raw.partial_cmp(&other.raw)
    }
}

impl<I: IndexType, T: Ord, const N: usize> Ord for TypedArray<I, T, N> {
    #[inline]
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.raw.cmp(&other.raw)
    }
}

impl<I: IndexType, T: core::hash::Hash, const N: usize> core::hash::Hash for TypedArray<I, T, N> {
    #[inline]
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.raw.hash(state);
    }
}

impl<I: IndexType, T: core::fmt::Debug, const N: usize> core::fmt::Debug for TypedArray<I, T, N> {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::Debug::fmt(&self.raw, f)
    }
}

impl<I: IndexType, T: Clone, const N: usize> Clone for TypedArray<I, T, N> {
    #[inline]
    fn clone(&self) -> Self {
        unsafe { Self::from_array_unchecked(self.raw.clone()) }
    }

    #[inline]
    fn clone_from(&mut self, source: &Self) {
        self.raw.clone_from(source.as_array())
    }
}

impl<I: IndexType, T: Default, const N: usize> Default for TypedArray<I, T, N> {
    #[inline]
    fn default() -> Self {
        TypedArray {
            raw: core::array::from_fn(|_| T::default()),
            phantom: PhantomData,
        }
    }
}

impl<I: IndexType, T, const N: usize, X> Index<X> for TypedArray<I, T, N>
where
    TypedSlice<I, T>: Index<X>,
{
    type Output = <TypedSlice<I, T> as Index<X>>::Output;

    #[inline]
    fn index(&self, index: X) -> &Self::Output {
        self.as_slice().index(index)
    }
}

impl<I: IndexType, T, const N: usize, X> IndexMut<X> for TypedArray<I, T, N>
where
    TypedSlice<I, T>: IndexMut<X>,
{
    #[inline]
    fn index_mut(&mut self, index: X) -> &mut Self::Output {
        self.as_mut_slice().index_mut(index)
    }
}

impl<I: IndexType, T, const N: usize> AsRef<TypedSlice<I, T>> for TypedArray<I, T, N> {
    #[inline]
    fn as_ref(&self) -> &TypedSlice<I, T> {
        unsafe { TypedSlice::from_slice_unchecked(&self.raw) }
    }
}

impl<I: IndexType, T, const N: usize> AsMut<TypedSlice<I, T>> for TypedArray<I, T, N> {
    #[inline]
    fn as_mut(&mut self) -> &mut TypedSlice<I, T> {
        unsafe { TypedSlice::from_slice_unchecked_mut(&mut self.raw) }
    }
}

impl<I: IndexType, T, const N: usize> core::borrow::Borrow<TypedSlice<I, T>>
    for TypedArray<I, T, N>
{
    #[inline]
    fn borrow(&self) -> &TypedSlice<I, T> {
        unsafe { TypedSlice::from_slice_unchecked(&self.raw) }
    }
}

impl<I: IndexType, T, const N: usize> core::borrow::BorrowMut<TypedSlice<I, T>>
    for TypedArray<I, T, N>
{
    #[inline]
    fn borrow_mut(&mut self) -> &mut TypedSlice<I, T> {
        unsafe { TypedSlice::from_slice_unchecked_mut(&mut self.raw) }
    }
}

impl<'a, I: IndexType, T, const N: usize> IntoIterator for &'a TypedArray<I, T, N> {
    type Item = &'a T;
    type IntoIter = core::slice::Iter<'a, T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.raw.iter()
    }
}

impl<'a, I: IndexType, T, const N: usize> IntoIterator for &'a mut TypedArray<I, T, N> {
    type Item = &'a mut T;
    type IntoIter = core::slice::IterMut<'a, T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.raw.iter_mut()
    }
}

impl<I: IndexType, T, const N: usize> IntoIterator for TypedArray<I, T, N> {
    type Item = T;
    type IntoIter = core::array::IntoIter<T, N>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.raw.into_iter()
    }
}

impl<'a, I: IndexType, T, const N: usize> TryFrom<&'a TypedSlice<I, T>>
    for &'a TypedArray<I, T, N>
{
    type Error = TryFromSliceError;

    #[inline]
    fn try_from(slice: &'a TypedSlice<I, T>) -> Result<&'a TypedArray<I, T, N>, TryFromSliceError> {
        Ok(unsafe { TypedArray::from_array_ref_unchecked(slice.as_slice().try_into()?) })
    }
}

impl<'a, I: IndexType, T, const N: usize> TryFrom<&'a mut TypedSlice<I, T>>
    for &'a mut TypedArray<I, T, N>
{
    type Error = TryFromSliceError;

    #[inline]
    fn try_from(
        slice: &'a mut TypedSlice<I, T>,
    ) -> Result<&'a mut TypedArray<I, T, N>, TryFromSliceError> {
        Ok(unsafe { TypedArray::from_array_mut_unchecked(slice.as_mut_slice().try_into()?) })
    }
}

impl<'a, I: IndexType, T: Copy, const N: usize> TryFrom<&'a TypedSlice<I, T>>
    for TypedArray<I, T, N>
{
    type Error = TryFromSliceError;

    #[inline]
    fn try_from(slice: &'a TypedSlice<I, T>) -> Result<TypedArray<I, T, N>, TryFromSliceError> {
        Ok(unsafe { TypedArray::from_array_unchecked(slice.as_slice().try_into()?) })
    }
}

impl<'a, I: IndexType, T: Copy, const N: usize> TryFrom<&'a mut TypedSlice<I, T>>
    for TypedArray<I, T, N>
{
    type Error = TryFromSliceError;

    #[inline]
    fn try_from(slice: &'a mut TypedSlice<I, T>) -> Result<TypedArray<I, T, N>, TryFromSliceError> {
        Ok(unsafe { TypedArray::from_array_unchecked(slice.as_mut_slice().try_into()?) })
    }
}

impl<'a, I: IndexType, T, const N: usize> core::ops::Deref for TypedArray<I, T, N> {
    type Target = TypedSlice<I, T>;

    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

impl<'a, I: IndexType, T, const N: usize> core::ops::DerefMut for TypedArray<I, T, N> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut_slice()
    }
}
