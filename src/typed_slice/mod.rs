use core::{
    marker::PhantomData,
    mem::MaybeUninit,
    ops::{Index, IndexMut},
};

use crate::{
    IndexScalarType, IndexTooBigError, IndexType, typed_array::TypedArray, typed_vec::TypedVec,
    utils::range_bounds_to_raw,
};

mod index;

pub use index::TypedSliceIndex;

/// A slice wrapper that uses a custom index type.
#[repr(transparent)]
pub struct TypedSlice<I: IndexType, T> {
    phantom: PhantomData<fn(&I)>,
    raw: [T],
}

/// This function is logically unsafe, but is not marked as such so it can be passed as a callback
/// to `Iterator::map` directly.
///
/// # Safety
///
/// The length of the slice must be less than or equal to `I::MAX_RAW_INDEX`.
#[inline]
fn unsafe_typed_slice_from_slice_unchecked<I: IndexType, T>(slice: &[T]) -> &TypedSlice<I, T> {
    // SAFETY: The length of the slice is checked or guaranteed to be in bounds for I before calling this.
    unsafe { TypedSlice::from_slice_unchecked(slice) }
}

/// This function is logically unsafe, but is not marked as such so it can be passed as a callback
/// to `Iterator::map` directly.
///
/// # Safety
///
/// The length of the slice must be less than or equal to `I::MAX_RAW_INDEX`.
#[inline]
fn unsafe_typed_slice_from_slice_unchecked_mut<I: IndexType, T>(
    slice: &mut [T],
) -> &mut TypedSlice<I, T> {
    // SAFETY: The length of the slice is checked or guaranteed to be in bounds for I before calling this.
    unsafe { TypedSlice::from_slice_unchecked_mut(slice) }
}

impl<I: IndexType, T> TypedSlice<I, T> {
    /// Tries to create a `TypedSlice` from a raw slice.
    #[inline]
    pub fn try_from_slice(slice: &[T]) -> Result<&Self, I::IndexTooBigError> {
        let _ = I::try_from_raw_index(slice.len())?;
        // SAFETY: The length of the slice is checked to be in bounds for I.
        Ok(unsafe { Self::from_slice_unchecked(slice) })
    }

    /// Tries to create a mutable `TypedSlice` from a mutable raw slice.
    #[inline]
    pub fn try_from_slice_mut(slice: &mut [T]) -> Result<&mut Self, I::IndexTooBigError> {
        let _ = I::try_from_raw_index(slice.len())?;
        // SAFETY: The length of the slice is checked to be in bounds for I.
        Ok(unsafe { Self::from_slice_unchecked_mut(slice) })
    }

    /// Creates a `TypedSlice` from raw parts.
    ///
    /// # Safety
    ///
    /// See `core::slice::from_raw_parts`.
    #[inline]
    pub unsafe fn from_raw_parts<'a>(data: *const T, len: I) -> &'a TypedSlice<I, T> {
        // SAFETY: The caller ensures the pointer and length are valid.
        let slice = unsafe { core::slice::from_raw_parts(data, len.to_raw_index()) };
        // SAFETY: The length of the slice is guaranteed to be in bounds for I.
        unsafe { Self::from_slice_unchecked(slice) }
    }

    /// Creates a mutable `TypedSlice` from raw parts.
    ///
    /// # Safety
    ///
    /// See `core::slice::from_raw_parts_mut`.
    #[inline]
    pub unsafe fn from_raw_parts_mut<'a>(data: *mut T, len: I) -> &'a mut TypedSlice<I, T> {
        // SAFETY: The caller ensures the pointer and length are valid.
        let slice = unsafe { core::slice::from_raw_parts_mut(data, len.to_raw_index()) };
        // SAFETY: The length of the slice is guaranteed to be in bounds for I.
        unsafe { Self::from_slice_unchecked_mut(slice) }
    }

    /// Creates a `TypedSlice` from a raw slice without checking if its length is in bounds for I.
    ///
    /// # Safety
    ///
    /// The length of the slice must be less than or equal to `I::MAX_RAW_INDEX`.
    #[inline]
    pub const unsafe fn from_slice_unchecked(slice: &[T]) -> &Self {
        // SAFETY: TypedSlice is repr(transparent) over [T].
        unsafe { core::mem::transmute(slice) }
    }

    /// Creates a mutable `TypedSlice` from a mutable raw slice without checking if its length is in bounds for I.
    ///
    /// # Safety
    ///
    /// The length of the slice must be less than or equal to `I::MAX_RAW_INDEX`.
    #[inline]
    pub const unsafe fn from_slice_unchecked_mut(slice: &mut [T]) -> &mut Self {
        // SAFETY: TypedSlice is repr(transparent) over [T].
        unsafe { core::mem::transmute(slice) }
    }

    /// Returns the `TypedSlice` as a raw slice reference.
    #[inline]
    pub const fn as_slice(&self) -> &[T] {
        // SAFETY: TypedSlice is repr(transparent) over [T].
        unsafe { core::mem::transmute(self) }
    }

    /// Returns the `TypedSlice` as a mutable raw slice reference.
    #[inline]
    pub const fn as_mut_slice(&mut self) -> &mut [T] {
        // SAFETY: TypedSlice is repr(transparent) over [T].
        unsafe { core::mem::transmute(self) }
    }

    /// Casts the index type of the `TypedSlice`.
    #[inline]
    pub fn cast_index_type<I2: IndexType>(
        &self,
    ) -> Result<&TypedSlice<I2, T>, I2::IndexTooBigError> {
        if I::MAX_RAW_INDEX <= I2::MAX_RAW_INDEX {
            // we know that the length of this slice must be in bounds for the new index type
            // SAFETY: The current length is valid for I, and I2 can represent all values I can.
            Ok(unsafe { TypedSlice::from_slice_unchecked(self.as_slice()) })
        } else {
            TypedSlice::try_from_slice(self.as_slice())
        }
    }

    /// Casts the index type of the mutable `TypedSlice`.
    #[inline]
    pub fn cast_index_type_mut<I2: IndexType>(
        &mut self,
    ) -> Result<&mut TypedSlice<I2, T>, I2::IndexTooBigError> {
        if I::MAX_RAW_INDEX <= I2::MAX_RAW_INDEX {
            // we know that the length of this slice must be in bounds for the new index type
            // SAFETY: The current length is valid for I, and I2 can represent all values I can.
            Ok(unsafe { TypedSlice::from_slice_unchecked_mut(self.as_mut_slice()) })
        } else {
            TypedSlice::try_from_slice_mut(self.as_mut_slice())
        }
    }

    /// Returns the length of the slice as a `usize`.
    #[inline]
    pub const fn len_usize(&self) -> usize {
        self.raw.len()
    }

    /// Returns the length of the slice as an index.
    #[inline]
    pub fn len(&self) -> I {
        // SAFETY: The length of the slice is guaranteed to be in bounds for I.
        unsafe { I::from_raw_index_unchecked(self.raw.len()) }
    }

    /// Returns `true` if the slice is empty.
    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.raw.is_empty()
    }

    #[inline]
    pub const fn first(&self) -> Option<&T> {
        self.raw.first()
    }

    #[inline]
    pub const fn first_mut(&mut self) -> Option<&mut T> {
        self.raw.first_mut()
    }

    #[inline]
    pub const fn split_first(&self) -> Option<(&T, &TypedSlice<I, T>)> {
        match self.raw.split_first() {
            // SAFETY: The rest of the slice must have a length less than or equal to the original, which was already valid for I.
            Some((first, rest)) => Some((first, unsafe { TypedSlice::from_slice_unchecked(rest) })),
            None => None,
        }
    }

    #[inline]
    pub const fn split_first_mut(&mut self) -> Option<(&mut T, &mut TypedSlice<I, T>)> {
        match self.raw.split_first_mut() {
            // SAFETY: The rest of the slice must have a length less than or equal to the original, which was already valid for I.
            Some((first, rest)) => {
                Some((first, unsafe { TypedSlice::from_slice_unchecked_mut(rest) }))
            }
            None => None,
        }
    }

    #[inline]
    pub const fn split_last(&self) -> Option<(&T, &TypedSlice<I, T>)> {
        match self.raw.split_last() {
            // SAFETY: The rest of the slice must have a length less than or equal to the original, which was already valid for I.
            Some((last, rest)) => Some((last, unsafe { TypedSlice::from_slice_unchecked(rest) })),
            None => None,
        }
    }

    #[inline]
    pub const fn split_last_mut(&mut self) -> Option<(&mut T, &mut TypedSlice<I, T>)> {
        match self.raw.split_last_mut() {
            // SAFETY: The rest of the slice must have a length less than or equal to the original, which was already valid for I.
            Some((last, rest)) => {
                Some((last, unsafe { TypedSlice::from_slice_unchecked_mut(rest) }))
            }
            None => None,
        }
    }

    #[inline]
    pub const fn last(&self) -> Option<&T> {
        self.raw.last()
    }

    #[inline]
    pub const fn last_mut(&mut self) -> Option<&mut T> {
        self.raw.last_mut()
    }

    #[inline]
    pub const fn first_chunk<const N: usize>(&self) -> Option<&TypedArray<I, T, N>> {
        match self.raw.first_chunk() {
            // SAFETY: TypedArray is repr(transparent) over [T; N]. N must be valid for I because N <= len and len is valid for I.
            Some(x) => Some(unsafe { TypedArray::from_array_ref_unchecked(x) }),
            None => None,
        }
    }

    #[inline]
    pub const fn first_chunk_mut<const N: usize>(&mut self) -> Option<&mut TypedArray<I, T, N>> {
        match self.raw.first_chunk_mut() {
            // SAFETY: TypedArray is repr(transparent) over [T; N]. N must be valid for I because N <= len and len is valid for I.
            Some(x) => Some(unsafe { TypedArray::from_array_mut_unchecked(x) }),
            None => None,
        }
    }

    #[inline]
    pub const fn split_first_chunk<const N: usize>(
        &self,
    ) -> Option<(&TypedArray<I, T, N>, &TypedSlice<I, T>)> {
        match self.raw.split_first_chunk() {
            // SAFETY: Valid lengths for I.
            Some((chunk, rest)) => unsafe {
                Some((
                    TypedArray::from_array_ref_unchecked(chunk),
                    TypedSlice::from_slice_unchecked(rest),
                ))
            },
            None => None,
        }
    }

    #[inline]
    pub const fn split_first_chunk_mut<const N: usize>(
        &mut self,
    ) -> Option<(&mut TypedArray<I, T, N>, &mut TypedSlice<I, T>)> {
        match self.raw.split_first_chunk_mut() {
            // SAFETY: Valid lengths for I.
            Some((chunk, rest)) => unsafe {
                Some((
                    TypedArray::from_array_mut_unchecked(chunk),
                    TypedSlice::from_slice_unchecked_mut(rest),
                ))
            },
            None => None,
        }
    }

    #[inline]
    pub const fn split_last_chunk<const N: usize>(
        &self,
    ) -> Option<(&TypedSlice<I, T>, &TypedArray<I, T, N>)> {
        match self.raw.split_last_chunk() {
            // SAFETY: Valid lengths for I.
            Some((rest, chunk)) => unsafe {
                Some((
                    TypedSlice::from_slice_unchecked(rest),
                    TypedArray::from_array_ref_unchecked(chunk),
                ))
            },
            None => None,
        }
    }

    #[inline]
    pub const fn split_last_chunk_mut<const N: usize>(
        &mut self,
    ) -> Option<(&mut TypedSlice<I, T>, &mut TypedArray<I, T, N>)> {
        match self.raw.split_last_chunk_mut() {
            // SAFETY: Valid lengths for I.
            Some((rest, chunk)) => unsafe {
                Some((
                    TypedSlice::from_slice_unchecked_mut(rest),
                    TypedArray::from_array_mut_unchecked(chunk),
                ))
            },
            None => None,
        }
    }

    #[inline]
    pub const fn last_chunk<const N: usize>(&self) -> Option<&TypedArray<I, T, N>> {
        match self.raw.last_chunk() {
            // SAFETY: Valid length for I.
            Some(x) => Some(unsafe { TypedArray::from_array_ref_unchecked(x) }),
            None => None,
        }
    }

    #[inline]
    pub const fn last_chunk_mut<const N: usize>(&mut self) -> Option<&mut TypedArray<I, T, N>> {
        match self.raw.last_chunk_mut() {
            // SAFETY: Valid length for I.
            Some(x) => Some(unsafe { TypedArray::from_array_mut_unchecked(x) }),
            None => None,
        }
    }

    #[inline]
    pub fn get<X>(&self, index: X) -> Option<&X::Output>
    where
        X: TypedSliceIndex<Self>,
    {
        index.get(self)
    }

    #[inline]
    pub fn get_mut<X>(&mut self, index: X) -> Option<&mut X::Output>
    where
        X: TypedSliceIndex<Self>,
    {
        index.get_mut(self)
    }

    /// # Safety
    ///
    /// The index must be in bounds.
    #[inline]
    pub unsafe fn get_unchecked<X>(&self, index: X) -> &X::Output
    where
        X: TypedSliceIndex<Self>,
    {
        // SAFETY: The caller ensures the index is in bounds.
        unsafe { &*index.get_unchecked(self) }
    }

    /// # Safety
    ///
    /// The index must be in bounds.
    #[inline]
    pub unsafe fn get_unchecked_mut<X>(&mut self, index: X) -> &mut X::Output
    where
        X: TypedSliceIndex<Self>,
    {
        // SAFETY: The caller ensures the index is in bounds.
        unsafe { &mut *index.get_unchecked_mut(self) }
    }

    #[inline]
    pub const fn as_ptr(&self) -> *const T {
        self.raw.as_ptr()
    }

    #[inline]
    pub const fn as_mut_ptr(&mut self) -> *mut T {
        self.raw.as_mut_ptr()
    }

    #[inline]
    pub const fn as_ptr_range(&self) -> core::ops::Range<*const T> {
        self.raw.as_ptr_range()
    }

    #[inline]
    pub const fn as_mut_ptr_range(&mut self) -> core::ops::Range<*mut T> {
        self.raw.as_mut_ptr_range()
    }

    #[inline]
    pub const fn as_array<const N: usize>(&self) -> Option<&TypedArray<I, T, N>> {
        match self.raw.as_array() {
            // SAFETY: Valid length for I.
            Some(x) => Some(unsafe { TypedArray::from_array_ref_unchecked(x) }),
            None => None,
        }
    }

    #[inline]
    pub const fn as_mut_array<const N: usize>(&mut self) -> Option<&mut TypedArray<I, T, N>> {
        match self.raw.as_mut_array() {
            // SAFETY: Valid length for I.
            Some(x) => Some(unsafe { TypedArray::from_array_mut_unchecked(x) }),
            None => None,
        }
    }

    #[inline]
    pub fn swap(&mut self, a: I, b: I) {
        self.raw.swap(a.to_raw_index(), b.to_raw_index());
    }

    #[inline]
    pub const fn reverse(&mut self) {
        self.raw.reverse();
    }

    #[inline]
    pub fn iter(&self) -> core::slice::Iter<'_, T> {
        self.raw.iter()
    }

    #[inline]
    pub fn iter_mut(&mut self) -> core::slice::IterMut<'_, T> {
        self.raw.iter_mut()
    }

    /// # Safety
    ///
    /// See `core::slice::as_chunks_unchecked`.
    #[inline]
    pub const unsafe fn as_chunks_unchecked<const N: usize>(
        &self,
    ) -> &TypedSlice<I, TypedArray<I, T, N>> {
        // SAFETY: The caller ensures that N divides the length.
        unsafe { typed_slice_from_chunks_unchecked(self.raw.as_chunks_unchecked::<N>()) }
    }

    #[inline]
    pub const fn as_chunks<const N: usize>(
        &self,
    ) -> (&TypedSlice<I, TypedArray<I, T, N>>, &TypedSlice<I, T>) {
        let (chunks, rest) = self.raw.as_chunks::<N>();
        // SAFETY: Lengths are valid for I.
        unsafe {
            (
                typed_slice_from_chunks_unchecked(chunks),
                TypedSlice::from_slice_unchecked(rest),
            )
        }
    }

    #[inline]
    pub const fn as_rchunks<const N: usize>(
        &self,
    ) -> (&TypedSlice<I, T>, &TypedSlice<I, TypedArray<I, T, N>>) {
        let (rest, chunks) = self.raw.as_rchunks::<N>();
        // SAFETY: Lengths are valid for I.
        unsafe {
            (
                TypedSlice::from_slice_unchecked(rest),
                typed_slice_from_chunks_unchecked(chunks),
            )
        }
    }

    /// # Safety
    ///
    /// See `core::slice::as_chunks_unchecked_mut`.
    #[inline]
    pub const unsafe fn as_chunks_unchecked_mut<const N: usize>(
        &mut self,
    ) -> &mut TypedSlice<I, TypedArray<I, T, N>> {
        // SAFETY: The caller ensures that N divides the length.
        unsafe { typed_slice_from_chunks_unchecked_mut(self.raw.as_chunks_unchecked_mut::<N>()) }
    }

    #[inline]
    pub const fn as_chunks_mut<const N: usize>(
        &mut self,
    ) -> (
        &mut TypedSlice<I, TypedArray<I, T, N>>,
        &mut TypedSlice<I, T>,
    ) {
        let (chunks, rest) = self.raw.as_chunks_mut::<N>();
        // SAFETY: Lengths are valid for I.
        unsafe {
            (
                typed_slice_from_chunks_unchecked_mut(chunks),
                TypedSlice::from_slice_unchecked_mut(rest),
            )
        }
    }

    #[inline]
    pub const fn as_rchunks_mut<const N: usize>(
        &mut self,
    ) -> (
        &mut TypedSlice<I, T>,
        &mut TypedSlice<I, TypedArray<I, T, N>>,
    ) {
        let (rest, chunks) = self.raw.as_rchunks_mut::<N>();
        // SAFETY: Lengths are valid for I.
        unsafe {
            (
                TypedSlice::from_slice_unchecked_mut(rest),
                typed_slice_from_chunks_unchecked_mut(chunks),
            )
        }
    }

    /// # Safety
    ///
    /// `mid` must be in bounds.
    #[inline]
    pub unsafe fn split_at_unchecked(&self, mid: I) -> (&TypedSlice<I, T>, &TypedSlice<I, T>) {
        // SAFETY: The caller ensures mid is in bounds.
        let (a, b) = unsafe { self.raw.split_at_unchecked(mid.to_raw_index()) };
        // SAFETY: Lengths are valid for I.
        unsafe { (Self::from_slice_unchecked(a), Self::from_slice_unchecked(b)) }
    }

    /// # Safety
    ///
    /// `mid` must be in bounds.
    #[inline]
    pub unsafe fn split_at_mut_unchecked(
        &mut self,
        mid: I,
    ) -> (&mut TypedSlice<I, T>, &mut TypedSlice<I, T>) {
        // SAFETY: The caller ensures mid is in bounds.
        let (a, b) = unsafe { self.raw.split_at_mut_unchecked(mid.to_raw_index()) };
        // SAFETY: Lengths are valid for I.
        unsafe {
            (
                Self::from_slice_unchecked_mut(a),
                Self::from_slice_unchecked_mut(b),
            )
        }
    }

    #[inline]
    pub fn split_at_checked(&self, mid: I) -> Option<(&TypedSlice<I, T>, &TypedSlice<I, T>)> {
        self.raw
            .split_at_checked(mid.to_raw_index())
            // SAFETY: Lengths are valid for I.
            .map(|(a, b)| unsafe { (Self::from_slice_unchecked(a), Self::from_slice_unchecked(b)) })
    }

    #[inline]
    pub fn split_at_mut_checked(
        &mut self,
        mid: I,
    ) -> Option<(&mut TypedSlice<I, T>, &mut TypedSlice<I, T>)> {
        self.raw
            .split_at_mut_checked(mid.to_raw_index())
            // SAFETY: Lengths are valid for I.
            .map(|(a, b)| unsafe {
                (
                    Self::from_slice_unchecked_mut(a),
                    Self::from_slice_unchecked_mut(b),
                )
            })
    }

    #[inline]
    pub fn splitn<F>(
        &self,
        n: usize,
        pred: F,
    ) -> core::iter::Map<core::slice::SplitN<'_, T, F>, fn(&[T]) -> &TypedSlice<I, T>>
    where
        F: FnMut(&T) -> bool,
    {
        self.raw
            .splitn(n, pred)
            .map(unsafe_typed_slice_from_slice_unchecked::<I, T>)
    }

    #[inline]
    pub fn splitn_mut<F>(
        &mut self,
        n: usize,
        pred: F,
    ) -> core::iter::Map<core::slice::SplitNMut<'_, T, F>, fn(&mut [T]) -> &mut TypedSlice<I, T>>
    where
        F: FnMut(&T) -> bool,
    {
        self.raw
            .splitn_mut(n, pred)
            .map(unsafe_typed_slice_from_slice_unchecked_mut::<I, T>)
    }

    #[inline]
    pub fn rsplitn<F>(
        &self,
        n: usize,
        pred: F,
    ) -> core::iter::Map<core::slice::RSplitN<'_, T, F>, fn(&[T]) -> &TypedSlice<I, T>>
    where
        F: FnMut(&T) -> bool,
    {
        self.raw
            .rsplitn(n, pred)
            .map(unsafe_typed_slice_from_slice_unchecked::<I, T>)
    }

    #[inline]
    pub fn rsplitn_mut<F>(
        &mut self,
        n: usize,
        pred: F,
    ) -> core::iter::Map<core::slice::RSplitNMut<'_, T, F>, fn(&mut [T]) -> &mut TypedSlice<I, T>>
    where
        F: FnMut(&T) -> bool,
    {
        self.raw
            .rsplitn_mut(n, pred)
            .map(unsafe_typed_slice_from_slice_unchecked_mut::<I, T>)
    }

    #[inline]
    pub fn contains(&self, x: &T) -> bool
    where
        T: PartialEq,
    {
        self.raw.contains(x)
    }

    #[inline]
    pub fn binary_search(&self, x: &T) -> Result<I, I>
    where
        T: Ord,
    {
        // SAFETY: binary_search returns indices that are in bounds, which are valid for I.
        unsafe { typify_binary_search_res(self.raw.binary_search(x)) }
    }

    #[inline]
    pub fn binary_search_by<'a, F>(&'a self, f: F) -> Result<I, I>
    where
        F: FnMut(&'a T) -> core::cmp::Ordering,
    {
        // SAFETY: binary_search_by returns indices that are in bounds, which are valid for I.
        unsafe { typify_binary_search_res(self.raw.binary_search_by(f)) }
    }

    #[inline]
    pub fn binary_search_by_key<'a, B, F>(&'a self, b: &B, f: F) -> Result<I, I>
    where
        F: FnMut(&'a T) -> B,
        B: Ord,
    {
        // SAFETY: binary_search_by_key returns indices that are in bounds, which are valid for I.
        unsafe { typify_binary_search_res(self.raw.binary_search_by_key(b, f)) }
    }

    #[inline]
    pub fn sort_unstable(&mut self)
    where
        T: Ord,
    {
        self.raw.sort_unstable()
    }

    #[inline]
    pub fn sort_unstable_by<F>(&mut self, compare: F)
    where
        F: FnMut(&T, &T) -> core::cmp::Ordering,
    {
        self.raw.sort_unstable_by(compare)
    }

    #[inline]
    pub fn sort_unstable_by_key<K, F>(&mut self, f: F)
    where
        F: FnMut(&T) -> K,
        K: Ord,
    {
        self.raw.sort_unstable_by_key(f)
    }

    #[inline]
    pub fn select_nth_unstable(
        &mut self,
        index: I,
    ) -> (&mut TypedSlice<I, T>, &mut T, &mut TypedSlice<I, T>)
    where
        T: Ord,
    {
        // SAFETY: select_nth_unstable returns slices whose lengths are valid for I.
        unsafe { typify_select_nth_res(self.raw.select_nth_unstable(index.to_raw_index())) }
    }

    #[inline]
    pub fn select_nth_unstable_by<F>(
        &mut self,
        index: I,
        compare: F,
    ) -> (&mut TypedSlice<I, T>, &mut T, &mut TypedSlice<I, T>)
    where
        F: FnMut(&T, &T) -> core::cmp::Ordering,
    {
        // SAFETY: select_nth_unstable_by returns slices whose lengths are valid for I.
        unsafe {
            typify_select_nth_res(
                self.raw
                    .select_nth_unstable_by(index.to_raw_index(), compare),
            )
        }
    }

    #[inline]
    pub fn select_nth_unstable_by_key<K, F>(
        &mut self,
        index: I,
        f: F,
    ) -> (&mut TypedSlice<I, T>, &mut T, &mut TypedSlice<I, T>)
    where
        F: FnMut(&T) -> K,
        K: Ord,
    {
        // SAFETY: select_nth_unstable_by_key returns slices whose lengths are valid for I.
        unsafe {
            typify_select_nth_res(self.raw.select_nth_unstable_by_key(index.to_raw_index(), f))
        }
    }

    #[inline]
    pub fn rotate_left(&mut self, mid: I) {
        self.raw.rotate_left(mid.to_raw_index())
    }

    #[inline]
    pub fn rotate_right(&mut self, k: I) {
        self.raw.rotate_right(k.to_raw_index())
    }

    #[inline]
    #[doc(alias = "memset")]
    pub fn fill(&mut self, value: T)
    where
        T: Clone,
    {
        self.raw.fill(value)
    }

    #[inline]
    pub fn fill_with<F>(&mut self, f: F)
    where
        F: FnMut() -> T,
    {
        self.raw.fill_with(f)
    }

    #[inline]
    pub fn copy_within<R: core::ops::RangeBounds<I>>(&mut self, src: R, dest: I)
    where
        T: Copy,
    {
        self.raw
            .copy_within(range_bounds_to_raw(src), dest.to_raw_index())
    }

    #[inline]
    pub fn is_sorted(&self) -> bool
    where
        T: PartialOrd,
    {
        self.raw.is_sorted()
    }

    #[inline]
    pub fn is_sorted_by<'a, F>(&'a self, compare: F) -> bool
    where
        F: FnMut(&'a T, &'a T) -> bool,
    {
        self.raw.is_sorted_by(compare)
    }

    #[inline]
    pub fn is_sorted_by_key<'a, F, K>(&'a self, f: F) -> bool
    where
        F: FnMut(&'a T) -> K,
        K: PartialOrd,
    {
        self.raw.is_sorted_by_key(f)
    }

    #[inline]
    pub fn partition_point<P>(&self, pred: P) -> I
    where
        P: FnMut(&T) -> bool,
    {
        // SAFETY: partition_point returns an index that is at most the length of the slice, which is valid for I.
        unsafe { I::from_raw_index_unchecked(self.raw.partition_point(pred)) }
    }

    #[inline]
    pub fn get_disjoint_mut<X, const N: usize>(
        &mut self,
        indices: [X; N],
    ) -> Result<[&mut X::Output; N], GetDisjointMutError>
    where
        X: GetDisjointMutTypedIndex + TypedSliceIndex<Self>,
    {
        get_disjoint_check_valid(&indices, self.len_usize())?;
        unsafe { Ok(self.get_disjoint_unchecked_mut(indices)) }
    }

    /// # Safety
    ///
    /// All indices must be in bounds and non-overlapping.
    #[inline]
    pub unsafe fn get_disjoint_unchecked_mut<X, const N: usize>(
        &mut self,
        indices: [X; N],
    ) -> [&mut X::Output; N]
    where
        X: TypedSliceIndex<Self>,
    {
        let slice: *mut TypedSlice<I, T> = self;
        let mut arr: MaybeUninit<[&mut X::Output; N]> = MaybeUninit::uninit();
        let arr_ptr = arr.as_mut_ptr().cast::<&mut X::Output>();
        for (i, idx) in indices.into_iter().enumerate() {
            // SAFETY: The caller ensures indices are in bounds and non-overlapping.
            unsafe { arr_ptr.add(i).write(&mut *idx.get_unchecked_mut(slice)) }
        }
        // SAFETY: All elements of the array have been initialized.
        unsafe { arr.assume_init() }
    }

    #[inline]
    pub fn windows(
        &self,
        size: usize,
    ) -> core::iter::Map<core::slice::Windows<'_, T>, fn(&[T]) -> &TypedSlice<I, T>> {
        self.raw
            .windows(size)
            .map(unsafe_typed_slice_from_slice_unchecked::<I, T>)
    }

    #[inline]
    pub fn chunks(
        &self,
        size: usize,
    ) -> core::iter::Map<core::slice::Chunks<'_, T>, fn(&[T]) -> &TypedSlice<I, T>> {
        self.raw
            .chunks(size)
            .map(unsafe_typed_slice_from_slice_unchecked::<I, T>)
    }

    #[inline]
    pub fn chunks_mut(
        &mut self,
        size: usize,
    ) -> core::iter::Map<core::slice::ChunksMut<'_, T>, fn(&mut [T]) -> &mut TypedSlice<I, T>> {
        self.raw
            .chunks_mut(size)
            .map(unsafe_typed_slice_from_slice_unchecked_mut::<I, T>)
    }

    #[inline]
    pub fn rchunks(
        &self,
        size: usize,
    ) -> core::iter::Map<core::slice::RChunks<'_, T>, fn(&[T]) -> &TypedSlice<I, T>> {
        self.raw
            .rchunks(size)
            .map(unsafe_typed_slice_from_slice_unchecked::<I, T>)
    }

    #[inline]
    pub fn rchunks_mut(
        &mut self,
        size: usize,
    ) -> core::iter::Map<core::slice::RChunksMut<'_, T>, fn(&mut [T]) -> &mut TypedSlice<I, T>>
    {
        self.raw
            .rchunks_mut(size)
            .map(unsafe_typed_slice_from_slice_unchecked_mut::<I, T>)
    }

    #[inline]
    pub fn split_at(&self, mid: I) -> (&TypedSlice<I, T>, &TypedSlice<I, T>) {
        let (a, b) = self.raw.split_at(mid.to_raw_index());
        unsafe { (Self::from_slice_unchecked(a), Self::from_slice_unchecked(b)) }
    }

    #[inline]
    pub fn split_at_mut(&mut self, mid: I) -> (&mut TypedSlice<I, T>, &mut TypedSlice<I, T>) {
        let (a, b) = self.raw.split_at_mut(mid.to_raw_index());
        unsafe {
            (
                Self::from_slice_unchecked_mut(a),
                Self::from_slice_unchecked_mut(b),
            )
        }
    }

    #[inline]
    pub fn split<F>(
        &self,
        pred: F,
    ) -> core::iter::Map<core::slice::Split<'_, T, F>, fn(&[T]) -> &TypedSlice<I, T>>
    where
        F: FnMut(&T) -> bool,
    {
        self.raw
            .split(pred)
            .map(unsafe_typed_slice_from_slice_unchecked)
    }

    #[inline]
    pub fn split_mut<F>(
        &mut self,
        pred: F,
    ) -> core::iter::Map<core::slice::SplitMut<'_, T, F>, fn(&mut [T]) -> &mut TypedSlice<I, T>>
    where
        F: FnMut(&T) -> bool,
    {
        self.raw
            .split_mut(pred)
            .map(unsafe_typed_slice_from_slice_unchecked_mut)
    }

    #[inline]
    pub fn split_inclusive<F>(
        &self,
        pred: F,
    ) -> core::iter::Map<core::slice::SplitInclusive<'_, T, F>, fn(&[T]) -> &TypedSlice<I, T>>
    where
        F: FnMut(&T) -> bool,
    {
        self.raw
            .split_inclusive(pred)
            .map(unsafe_typed_slice_from_slice_unchecked)
    }

    #[inline]
    pub fn split_inclusive_mut<F>(
        &mut self,
        pred: F,
    ) -> core::iter::Map<
        core::slice::SplitInclusiveMut<'_, T, F>,
        fn(&mut [T]) -> &mut TypedSlice<I, T>,
    >
    where
        F: FnMut(&T) -> bool,
    {
        self.raw
            .split_inclusive_mut(pred)
            .map(unsafe_typed_slice_from_slice_unchecked_mut)
    }

    #[inline]
    pub fn starts_with(&self, needle: &TypedSlice<I, T>) -> bool
    where
        T: PartialEq,
    {
        self.raw.starts_with(needle.as_slice())
    }

    #[inline]
    pub fn ends_with(&self, needle: &TypedSlice<I, T>) -> bool
    where
        T: PartialEq,
    {
        self.raw.ends_with(needle.as_slice())
    }

    #[inline]
    pub fn clone_from_slice(&mut self, src: &TypedSlice<I, T>)
    where
        T: Clone,
    {
        self.raw.clone_from_slice(src.as_slice())
    }

    #[inline]
    pub fn copy_from_slice(&mut self, src: &TypedSlice<I, T>)
    where
        T: Copy,
    {
        self.raw.copy_from_slice(src.as_slice())
    }

    #[inline]
    pub fn swap_with_slice(&mut self, other: &mut TypedSlice<I, T>) {
        self.raw.swap_with_slice(other.as_mut_slice())
    }

    #[inline]
    pub fn align_to<U>(&self) -> (&TypedSlice<I, T>, &[U], &TypedSlice<I, T>) {
        let (a, b, c) = unsafe { self.raw.align_to::<U>() };
        unsafe {
            (
                Self::from_slice_unchecked(a),
                b,
                Self::from_slice_unchecked(c),
            )
        }
    }

    #[inline]
    pub fn align_to_mut<U>(&mut self) -> (&mut TypedSlice<I, T>, &mut [U], &mut TypedSlice<I, T>) {
        let (a, b, c) = unsafe { self.raw.align_to_mut::<U>() };
        unsafe {
            (
                Self::from_slice_unchecked_mut(a),
                b,
                Self::from_slice_unchecked_mut(c),
            )
        }
    }

    #[inline]
    pub fn chunk_by<F>(
        &self,
        pred: F,
    ) -> core::iter::Map<core::slice::ChunkBy<'_, T, F>, fn(&[T]) -> &TypedSlice<I, T>>
    where
        F: FnMut(&T, &T) -> bool,
    {
        self.raw
            .chunk_by(pred)
            .map(unsafe_typed_slice_from_slice_unchecked)
    }

    #[inline]
    pub fn chunk_by_mut<F>(
        &mut self,
        pred: F,
    ) -> core::iter::Map<core::slice::ChunkByMut<'_, T, F>, fn(&mut [T]) -> &mut TypedSlice<I, T>>
    where
        F: FnMut(&T, &T) -> bool,
    {
        self.raw
            .chunk_by_mut(pred)
            .map(unsafe_typed_slice_from_slice_unchecked_mut)
    }

    #[inline]
    pub fn chunks_exact(
        &self,
        chunk_size: usize,
    ) -> core::iter::Map<core::slice::ChunksExact<'_, T>, fn(&[T]) -> &TypedSlice<I, T>> {
        self.raw
            .chunks_exact(chunk_size)
            .map(unsafe_typed_slice_from_slice_unchecked)
    }

    #[inline]
    pub fn chunks_exact_mut(
        &mut self,
        chunk_size: usize,
    ) -> core::iter::Map<core::slice::ChunksExactMut<'_, T>, fn(&mut [T]) -> &mut TypedSlice<I, T>>
    {
        self.raw
            .chunks_exact_mut(chunk_size)
            .map(unsafe_typed_slice_from_slice_unchecked_mut)
    }

    #[inline]
    pub fn rchunks_exact(
        &self,
        chunk_size: usize,
    ) -> core::iter::Map<core::slice::RChunksExact<'_, T>, fn(&[T]) -> &TypedSlice<I, T>> {
        self.raw
            .rchunks_exact(chunk_size)
            .map(unsafe_typed_slice_from_slice_unchecked)
    }

    #[inline]
    pub fn rchunks_exact_mut(
        &mut self,
        chunk_size: usize,
    ) -> core::iter::Map<core::slice::RChunksExactMut<'_, T>, fn(&mut [T]) -> &mut TypedSlice<I, T>>
    {
        self.raw
            .rchunks_exact_mut(chunk_size)
            .map(unsafe_typed_slice_from_slice_unchecked_mut)
    }

    #[inline]
    pub fn repeat(&self, n: usize) -> Result<TypedVec<I, T>, I::IndexTooBigError>
    where
        T: Copy,
    {
        let _final_len = self.len().checked_mul_scalar(
            <I::Scalar as IndexScalarType>::try_from_usize(n)
                .ok_or(<I::IndexTooBigError as IndexTooBigError>::new())?,
        )?;
        Ok(unsafe { TypedVec::from_vec_unchecked(self.raw.repeat(n)) })
    }

    #[inline]
    pub fn rsplit_mut<F>(
        &mut self,
        pred: F,
    ) -> core::iter::Map<core::slice::RSplitMut<'_, T, F>, fn(&mut [T]) -> &mut TypedSlice<I, T>>
    where
        F: FnMut(&T) -> bool,
    {
        self.raw
            .rsplit_mut(pred)
            .map(unsafe_typed_slice_from_slice_unchecked_mut)
    }

    #[inline]
    pub fn rsplit<F>(
        &self,
        pred: F,
    ) -> core::iter::Map<core::slice::RSplit<'_, T, F>, fn(&[T]) -> &TypedSlice<I, T>>
    where
        F: FnMut(&T) -> bool,
    {
        self.raw
            .rsplit(pred)
            .map(unsafe_typed_slice_from_slice_unchecked)
    }

    #[inline]
    pub fn sort(&mut self)
    where
        T: Ord,
    {
        self.raw.sort();
    }

    #[inline]
    pub fn sort_by<F>(&mut self, compare: F)
    where
        F: FnMut(&T, &T) -> core::cmp::Ordering,
    {
        self.raw.sort_by(compare);
    }

    #[inline]
    pub fn sort_by_key<K, F>(&mut self, f: F)
    where
        F: FnMut(&T) -> K,
        K: Ord,
    {
        self.raw.sort_by_key(f);
    }

    #[inline]
    pub fn sort_by_cached_key<K, F>(&mut self, f: F)
    where
        F: FnMut(&T) -> K,
        K: Ord,
    {
        self.raw.sort_by_cached_key(f);
    }

    #[inline]
    pub fn split_off_first<'a>(self: &mut &'a Self) -> Option<&'a T> {
        let raw: &mut &'a [T] = unsafe { core::mem::transmute(self) };
        raw.split_off_first()
    }

    #[inline]
    pub fn split_off_first_mut<'a>(self: &mut &'a mut Self) -> Option<&'a mut T> {
        let raw: &mut &'a mut [T] = unsafe { core::mem::transmute(self) };
        raw.split_off_first_mut()
    }

    #[inline]
    pub fn split_off_last<'a>(self: &mut &'a Self) -> Option<&'a T> {
        let raw: &mut &'a [T] = unsafe { core::mem::transmute(self) };
        raw.split_off_last()
    }

    #[inline]
    pub fn split_off_last_mut<'a>(self: &mut &'a mut Self) -> Option<&'a mut T> {
        let raw: &mut &'a mut [T] = unsafe { core::mem::transmute(self) };
        raw.split_off_last_mut()
    }

    #[inline]
    pub fn to_vec(&self) -> TypedVec<I, T>
    where
        T: Clone,
    {
        unsafe { TypedVec::from_vec_unchecked(self.raw.to_vec()) }
    }
}

impl<I: IndexType, T, const N: usize> TypedSlice<I, TypedArray<I, T, N>> {
    pub fn as_flattened(&self) -> Result<&TypedSlice<I, T>, I::IndexTooBigError> {
        // SAFETY: N is representable by I::Scalar if N <= MAX_RAW_INDEX, which we check.
        let n = unsafe { <I::Scalar as IndexScalarType>::from_usize_unchecked(N) };
        let flattened_len = self.len().checked_mul_scalar(n)?;
        // SAFETY: flattened_len is checked to be in bounds for I.
        Ok(unsafe { TypedSlice::from_raw_parts(self.as_ptr().cast(), flattened_len) })
    }

    pub fn as_flattened_mut(&mut self) -> Result<&mut TypedSlice<I, T>, I::IndexTooBigError> {
        // SAFETY: N is representable by I::Scalar if N <= MAX_RAW_INDEX, which we check.
        let n = unsafe { <I::Scalar as IndexScalarType>::from_usize_unchecked(N) };
        let flattened_len = self.len().checked_mul_scalar(n)?;
        // SAFETY: flattened_len is checked to be in bounds for I.
        Ok(unsafe { TypedSlice::from_raw_parts_mut(self.as_mut_ptr().cast(), flattened_len) })
    }
}

impl<I: IndexType, T: PartialEq> PartialEq for TypedSlice<I, T> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        PartialEq::eq(&self.raw, &other.raw)
    }
}

impl<I: IndexType, T: Eq> Eq for TypedSlice<I, T> {}

impl<I: IndexType, T: core::hash::Hash> core::hash::Hash for TypedSlice<I, T> {
    #[inline]
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.raw.hash(state);
    }
}

impl<I: IndexType, T: core::fmt::Debug> core::fmt::Debug for TypedSlice<I, T> {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.raw.fmt(f)
    }
}

impl<I: IndexType, T, X: TypedSliceIndex<TypedSlice<I, T>>> Index<X> for TypedSlice<I, T> {
    type Output = X::Output;

    #[inline]
    fn index(&self, index: X) -> &Self::Output {
        index.index(self)
    }
}

impl<I: IndexType, T, X: TypedSliceIndex<TypedSlice<I, T>>> IndexMut<X> for TypedSlice<I, T> {
    #[inline]
    fn index_mut(&mut self, index: X) -> &mut Self::Output {
        index.index_mut(self)
    }
}

impl<'a, I: IndexType, T> TryFrom<&'a [T]> for &'a TypedSlice<I, T> {
    type Error = I::IndexTooBigError;

    #[inline]
    fn try_from(value: &'a [T]) -> Result<Self, Self::Error> {
        TypedSlice::try_from_slice(value)
    }
}

impl<'a, I: IndexType, T> TryFrom<&'a mut [T]> for &'a mut TypedSlice<I, T> {
    type Error = I::IndexTooBigError;

    #[inline]
    fn try_from(value: &'a mut [T]) -> Result<Self, Self::Error> {
        TypedSlice::try_from_slice_mut(value)
    }
}

#[inline]
unsafe fn typify_binary_search_res<I: IndexType>(res: Result<usize, usize>) -> Result<I, I> {
    match res {
        // SAFETY: The caller ensures the usize is in bounds for I.
        Ok(v) => Ok(unsafe { I::from_raw_index_unchecked(v) }),
        // SAFETY: The caller ensures the usize is in bounds for I.
        Err(v) => Err(unsafe { I::from_raw_index_unchecked(v) }),
    }
}

#[inline]
unsafe fn typify_select_nth_res<'a, I: IndexType, T>(
    res: (&'a mut [T], &'a mut T, &'a mut [T]),
) -> (
    &'a mut TypedSlice<I, T>,
    &'a mut T,
    &'a mut TypedSlice<I, T>,
) {
    // SAFETY: The caller ensures the slices are in bounds for I.
    unsafe {
        (
            TypedSlice::from_slice_unchecked_mut(res.0),
            res.1,
            TypedSlice::from_slice_unchecked_mut(res.2),
        )
    }
}

#[inline]
const unsafe fn typed_slice_from_chunks_unchecked<I: IndexType, T, const N: usize>(
    slice: &[[T; N]],
) -> &TypedSlice<I, TypedArray<I, T, N>> {
    // SAFETY: TypedSlice and TypedArray are repr(transparent) over [T] and [T; N].
    unsafe { core::mem::transmute(slice) }
}

#[inline]
const unsafe fn typed_slice_from_chunks_unchecked_mut<I: IndexType, T, const N: usize>(
    slice: &mut [[T; N]],
) -> &mut TypedSlice<I, TypedArray<I, T, N>> {
    // SAFETY: TypedSlice and TypedArray are repr(transparent) over [T] and [T; N].
    unsafe { core::mem::transmute(slice) }
}

mod private_get_disjoint_mut_typed_index {
    pub trait Sealed {}
}

pub unsafe trait GetDisjointMutTypedIndex:
    private_get_disjoint_mut_typed_index::Sealed
{
    fn is_in_bounds(&self, len: usize) -> bool;

    fn is_overlapping(&self, other: &Self) -> bool;
}

impl<I: IndexType> private_get_disjoint_mut_typed_index::Sealed for I {}
unsafe impl<I: IndexType> GetDisjointMutTypedIndex for I {
    #[inline]
    fn is_in_bounds(&self, len: usize) -> bool {
        self.to_raw_index() < len
    }

    #[inline]
    fn is_overlapping(&self, other: &Self) -> bool {
        *self == *other
    }
}

impl<I: IndexType> private_get_disjoint_mut_typed_index::Sealed for core::ops::Range<I> {}
unsafe impl<I: IndexType> GetDisjointMutTypedIndex for core::ops::Range<I> {
    #[inline]
    fn is_in_bounds(&self, len: usize) -> bool {
        (self.start <= self.end) & (self.end.to_raw_index() <= len)
    }

    #[inline]
    fn is_overlapping(&self, other: &Self) -> bool {
        (self.start < other.end) & (other.start < self.end)
    }
}

impl<I: IndexType> private_get_disjoint_mut_typed_index::Sealed for core::ops::RangeInclusive<I> {}
unsafe impl<I: IndexType> GetDisjointMutTypedIndex for core::ops::RangeInclusive<I> {
    #[inline]
    fn is_in_bounds(&self, len: usize) -> bool {
        (self.start() <= self.end()) & (self.end().to_raw_index() < len)
    }

    #[inline]
    fn is_overlapping(&self, other: &Self) -> bool {
        (self.start() <= other.end()) & (other.start() <= self.end())
    }
}

#[inline]
fn get_disjoint_check_valid<
    I: IndexType,
    T,
    X: GetDisjointMutTypedIndex + TypedSliceIndex<TypedSlice<I, T>>,
    const N: usize,
>(
    indices: &[X; N],
    len: usize,
) -> Result<(), GetDisjointMutError> {
    for (i, idx) in indices.iter().enumerate() {
        if !idx.is_in_bounds(len) {
            return Err(GetDisjointMutError::IndexOutOfBounds);
        }
        for idx2 in &indices[..i] {
            if idx.is_overlapping(idx2) {
                return Err(GetDisjointMutError::OverlappingIndices);
            }
        }
    }
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GetDisjointMutError {
    IndexOutOfBounds,
    OverlappingIndices,
}

impl core::fmt::Display for GetDisjointMutError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let msg = match self {
            GetDisjointMutError::IndexOutOfBounds => "an index is out of bounds",
            GetDisjointMutError::OverlappingIndices => "there were overlapping indices",
        };
        core::fmt::Display::fmt(msg, f)
    }
}
