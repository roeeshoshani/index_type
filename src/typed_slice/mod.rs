use core::{
    marker::PhantomData,
    mem::MaybeUninit,
    ops::{Index, IndexMut},
};

use crate::{IndexTooBigError, IndexType};

mod index;

pub use index::TypedSliceIndex;

/// A slice with a strongly typed index.
///
/// This type is a wrapper around a standard slice `[T]`, but it uses a custom
/// index type `I` instead of `usize` for many of its operations.
#[repr(transparent)]
pub struct TypedSlice<I: IndexType, T> {
    phantom: PhantomData<fn(&I)>,
    raw: [T],
}

impl<I: IndexType, T> TypedSlice<I, T> {
    /// Creates a `&TypedSlice` from a standard slice `&[T]`.
    ///
    /// Returns [`IndexTooBigError`] if the slice's length is too large for `I`.
    #[inline]
    pub fn from_slice(slice: &[T]) -> Result<&Self, IndexTooBigError> {
        let _ = I::try_from_index(slice.len())?;
        // SAFETY: TypedSlice is repr(transparent) over [T].
        Ok(unsafe { core::mem::transmute(slice) })
    }

    /// Creates a `&mut TypedSlice` from a standard mutable slice `&mut [T]`.
    ///
    /// Returns [`IndexTooBigError`] if the slice's length is too large for `I`.
    #[inline]
    pub fn from_slice_mut(slice: &mut [T]) -> Result<&mut Self, IndexTooBigError> {
        let _ = I::try_from_index(slice.len())?;
        // SAFETY: TypedSlice is repr(transparent) over [T].
        Ok(unsafe { core::mem::transmute(slice) })
    }

    /// Creates a `&TypedSlice` from raw parts.
    ///
    /// # Safety
    ///
    /// `data` must be valid for `len` elements.
    #[inline]
    pub unsafe fn from_raw_parts<'a>(data: *const T, len: I) -> &'a TypedSlice<I, T> {
        // SAFETY: The caller guarantees that data is valid for len elements.
        let slice = unsafe { core::slice::from_raw_parts(data, len.to_index()) };
        // SAFETY: TypedSlice is repr(transparent) over [T].
        unsafe { Self::from_slice_unchecked(slice) }
    }

    /// Creates a `&mut TypedSlice` from raw parts.
    ///
    /// # Safety
    ///
    /// `data` must be valid for `len` elements.
    #[inline]
    pub unsafe fn from_raw_parts_mut<'a>(data: *mut T, len: I) -> &'a mut TypedSlice<I, T> {
        // SAFETY: The caller guarantees that data is valid for len elements.
        let slice = unsafe { core::slice::from_raw_parts_mut(data, len.to_index()) };
        // SAFETY: TypedSlice is repr(transparent) over [T].
        unsafe { Self::from_slice_unchecked_mut(slice) }
    }

    /// Creates a `&TypedSlice` from a standard slice without checking its length.
    ///
    /// # Safety
    ///
    /// The slice's length must be representable by `I`.
    #[inline]
    pub const unsafe fn from_slice_unchecked(slice: &[T]) -> &Self {
        // SAFETY: TypedSlice is repr(transparent) over [T]. The caller guarantees the length is valid for I.
        unsafe { core::mem::transmute(slice) }
    }

    /// Creates a `&mut TypedSlice` from a standard mutable slice without checking its length.
    ///
    /// # Safety
    ///
    /// The slice's length must be representable by `I`.
    #[inline]
    pub const unsafe fn from_slice_unchecked_mut(slice: &mut [T]) -> &mut Self {
        // SAFETY: TypedSlice is repr(transparent) over [T]. The caller guarantees the length is valid for I.
        unsafe { core::mem::transmute(slice) }
    }
}

// methods copied from stdlib's slice implementation
impl<I: IndexType, T> TypedSlice<I, T> {
    /// Returns the number of elements in the slice as a `usize`.
    #[inline]
    #[must_use]
    pub const fn len_usize(&self) -> usize {
        self.raw.len()
    }

    /// Returns the number of elements in the slice as an `I`.
    #[inline]
    #[must_use]
    pub fn len(&self) -> I {
        // SAFETY: The length of the slice was checked when it was created.
        unsafe { I::from_index_unchecked(self.raw.len()) }
    }

    /// Returns `true` if the slice has a length of 0.
    #[inline]
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.raw.is_empty()
    }

    /// Returns the first element of the slice, or `None` if it is empty.
    #[inline]
    #[must_use]
    pub const fn first(&self) -> Option<&T> {
        self.raw.first()
    }

    /// Returns a mutable reference to the first element of the slice, or `None` if it is empty.
    #[inline]
    #[must_use]
    pub const fn first_mut(&mut self) -> Option<&mut T> {
        self.raw.first_mut()
    }

    /// Returns the first and all the rest of the elements of the slice, or `None` if it is empty.
    #[inline]
    #[must_use]
    pub const fn split_first(&self) -> Option<(&T, &[T])> {
        self.raw.split_first()
    }

    /// Returns the first and all the rest of the elements of the slice, or `None` if it is empty.
    #[inline]
    #[must_use]
    pub const fn split_first_mut(&mut self) -> Option<(&mut T, &mut [T])> {
        self.raw.split_first_mut()
    }

    /// Returns the last and all the rest of the elements of the slice, or `None` if it is empty.
    #[inline]
    #[must_use]
    pub const fn split_last(&self) -> Option<(&T, &[T])> {
        self.raw.split_last()
    }

    /// Returns the last and all the rest of the elements of the slice, or `None` if it is empty.
    #[inline]
    #[must_use]
    pub const fn split_last_mut(&mut self) -> Option<(&mut T, &mut [T])> {
        self.raw.split_last_mut()
    }

    /// Returns the last element of the slice, or `None` if it is empty.
    #[inline]
    #[must_use]
    pub const fn last(&self) -> Option<&T> {
        self.raw.last()
    }

    /// Returns a mutable reference to the last element of the slice, or `None` if it is empty.
    #[inline]
    #[must_use]
    pub const fn last_mut(&mut self) -> Option<&mut T> {
        self.raw.last_mut()
    }

    /// Returns the first `N` elements of the slice, or `None` if it has fewer than `N` elements.
    #[inline]
    pub const fn first_chunk<const N: usize>(&self) -> Option<&[T; N]> {
        self.raw.first_chunk()
    }

    /// Returns a mutable reference to the first `N` elements of the slice, or `None` if it has fewer than `N` elements.
    #[inline]
    pub const fn first_chunk_mut<const N: usize>(&mut self) -> Option<&mut [T; N]> {
        self.raw.first_chunk_mut()
    }

    /// Returns the first `N` elements and the rest of the slice, or `None` if it has fewer than `N` elements.
    #[inline]
    pub const fn split_first_chunk<const N: usize>(&self) -> Option<(&[T; N], &[T])> {
        self.raw.split_first_chunk()
    }

    /// Returns the first `N` elements and the rest of the slice, or `None` if it has fewer than `N` elements.
    #[inline]
    pub const fn split_first_chunk_mut<const N: usize>(
        &mut self,
    ) -> Option<(&mut [T; N], &mut [T])> {
        self.raw.split_first_chunk_mut()
    }

    /// Returns the last `N` elements and the rest of the slice, or `None` if it has fewer than `N` elements.
    #[inline]
    pub const fn split_last_chunk<const N: usize>(&self) -> Option<(&[T], &[T; N])> {
        self.raw.split_last_chunk()
    }

    /// Returns the last `N` elements and the rest of the slice, or `None` if it has fewer than `N` elements.
    #[inline]
    pub const fn split_last_chunk_mut<const N: usize>(
        &mut self,
    ) -> Option<(&mut [T], &mut [T; N])> {
        self.raw.split_last_chunk_mut()
    }

    /// Returns the last `N` elements of the slice, or `None` if it has fewer than `N` elements.
    #[inline]
    pub const fn last_chunk<const N: usize>(&self) -> Option<&[T; N]> {
        self.raw.last_chunk()
    }

    /// Returns a mutable reference to the last `N` elements of the slice, or `None` if it has fewer than `N` elements.
    #[inline]
    pub const fn last_chunk_mut<const N: usize>(&mut self) -> Option<&mut [T; N]> {
        self.raw.last_chunk_mut()
    }

    /// Returns a reference to an element or subslice depending on the type of index.
    #[inline]
    #[must_use]
    pub fn get<X>(&self, index: X) -> Option<&X::Output>
    where
        X: TypedSliceIndex<Self>,
    {
        index.get(self)
    }

    /// Returns a mutable reference to an element or subslice depending on the type of index.
    #[inline]
    #[must_use]
    pub fn get_mut<X>(&mut self, index: X) -> Option<&mut X::Output>
    where
        X: TypedSliceIndex<Self>,
    {
        index.get_mut(self)
    }

    /// Returns a reference to an element or subslice without checking bounds.
    ///
    /// # Safety
    ///
    /// The index must be within bounds.
    #[inline]
    #[must_use]
    #[track_caller]
    pub unsafe fn get_unchecked<X>(&self, index: X) -> &X::Output
    where
        X: TypedSliceIndex<Self>,
    {
        // SAFETY: The caller guarantees that the index is within bounds.
        unsafe { &*index.get_unchecked(self) }
    }

    /// Returns a mutable reference to an element or subslice without checking bounds.
    ///
    /// # Safety
    ///
    /// The index must be within bounds.
    #[inline]
    #[must_use]
    #[track_caller]
    pub unsafe fn get_unchecked_mut<X>(&mut self, index: X) -> &mut X::Output
    where
        X: TypedSliceIndex<Self>,
    {
        // SAFETY: The caller guarantees that the index is within bounds.
        unsafe { &mut *index.get_unchecked_mut(self) }
    }

    /// Returns a raw pointer to the slice's buffer.
    #[inline(always)]
    #[must_use]
    pub const fn as_ptr(&self) -> *const T {
        self.raw.as_ptr()
    }

    /// Returns an unsafe mutable pointer to the slice's buffer.
    #[inline(always)]
    #[must_use]
    pub const fn as_mut_ptr(&mut self) -> *mut T {
        self.raw.as_mut_ptr()
    }

    /// Returns the two raw pointers spanning the slice.
    #[inline]
    #[must_use]
    pub const fn as_ptr_range(&self) -> core::ops::Range<*const T> {
        self.raw.as_ptr_range()
    }

    /// Returns the two unsafe mutable pointers spanning the slice.
    #[inline]
    #[must_use]
    pub const fn as_mut_ptr_range(&mut self) -> core::ops::Range<*mut T> {
        self.raw.as_mut_ptr_range()
    }

    /// Returns a reference to the slice as an array of `N` elements, or `None` if it has a different length.
    #[inline]
    #[must_use]
    pub const fn as_array<const N: usize>(&self) -> Option<&[T; N]> {
        self.raw.as_array()
    }

    /// Returns a mutable reference to the slice as an array of `N` elements, or `None` if it has a different length.
    #[inline]
    #[must_use]
    pub const fn as_mut_array<const N: usize>(&mut self) -> Option<&mut [T; N]> {
        self.raw.as_mut_array()
    }

    /// Swaps two elements in the slice.
    #[inline]
    #[track_caller]
    pub fn swap(&mut self, a: I, b: I) {
        self.raw.swap(a.to_index(), b.to_index());
    }

    /// Reverses the order of elements in the slice, in place.
    #[inline]
    pub const fn reverse(&mut self) {
        self.raw.reverse();
    }

    /// Returns an iterator over the slice.
    #[inline]
    pub fn iter(&self) -> core::slice::Iter<'_, T> {
        self.raw.iter()
    }

    /// Returns an iterator that allows modifying each value.
    #[inline]
    pub fn iter_mut(&mut self) -> core::slice::IterMut<'_, T> {
        self.raw.iter_mut()
    }

    /// Returns an iterator over all contiguous windows of length `size`.
    #[inline]
    #[track_caller]
    pub fn windows(&self, size: usize) -> core::slice::Windows<'_, T> {
        self.raw.windows(size)
    }

    /// Returns an iterator over `chunk_size` elements of the slice at a time.
    #[inline]
    #[track_caller]
    pub fn chunks(&self, chunk_size: usize) -> core::slice::Chunks<'_, T> {
        self.raw.chunks(chunk_size)
    }

    /// Returns an iterator over `chunk_size` elements of the slice at a time.
    #[inline]
    #[track_caller]
    pub fn chunks_mut(&mut self, chunk_size: usize) -> core::slice::ChunksMut<'_, T> {
        self.raw.chunks_mut(chunk_size)
    }

    /// Returns an iterator over `chunk_size` elements of the slice at a time.
    #[inline]
    #[track_caller]
    pub fn chunks_exact(&self, chunk_size: usize) -> core::slice::ChunksExact<'_, T> {
        self.raw.chunks_exact(chunk_size)
    }

    /// Returns an iterator over `chunk_size` elements of the slice at a time.
    #[inline]
    #[track_caller]
    pub fn chunks_exact_mut(&mut self, chunk_size: usize) -> core::slice::ChunksExactMut<'_, T> {
        self.raw.chunks_exact_mut(chunk_size)
    }

    /// Returns the slice as a slice of `N`-element arrays, without checking that the length is a multiple of `N`.
    ///
    /// # Safety
    ///
    /// The length of the slice must be a multiple of `N`.
    #[inline]
    #[must_use]
    #[track_caller]
    pub unsafe fn as_chunks_unchecked<const N: usize>(&self) -> &[[T; N]] {
        // SAFETY: The caller guarantees the length is a multiple of N.
        unsafe { self.raw.as_chunks_unchecked() }
    }

    /// Returns the slice as a slice of `N`-element arrays and a remainder slice.
    #[inline]
    #[track_caller]
    #[must_use]
    pub const fn as_chunks<const N: usize>(&self) -> (&[[T; N]], &[T]) {
        self.raw.as_chunks()
    }

    /// Returns the slice as a slice of `N`-element arrays and a remainder slice, starting from the end.
    #[inline]
    #[track_caller]
    #[must_use]
    pub const fn as_rchunks<const N: usize>(&self) -> (&[T], &[[T; N]]) {
        self.raw.as_rchunks()
    }

    /// Returns the slice as a mutable slice of `N`-element arrays, without checking that the length is a multiple of `N`.
    ///
    /// # Safety
    ///
    /// The length of the slice must be a multiple of `N`.
    #[inline]
    #[must_use]
    #[track_caller]
    pub unsafe fn as_chunks_unchecked_mut<const N: usize>(&mut self) -> &mut [[T; N]] {
        // SAFETY: The caller guarantees the length is a multiple of N.
        unsafe { self.raw.as_chunks_unchecked_mut() }
    }

    /// Returns the slice as a mutable slice of `N`-element arrays and a remainder slice.
    #[inline]
    #[track_caller]
    #[must_use]
    pub const fn as_chunks_mut<const N: usize>(&mut self) -> (&mut [[T; N]], &mut [T]) {
        self.raw.as_chunks_mut()
    }

    /// Returns the slice as a mutable slice of `N`-element arrays and a remainder slice, starting from the end.
    #[inline]
    #[track_caller]
    #[must_use]
    pub const fn as_rchunks_mut<const N: usize>(&mut self) -> (&mut [T], &mut [[T; N]]) {
        self.raw.as_rchunks_mut()
    }

    /// Returns an iterator over `chunk_size` elements of the slice at a time, starting from the end of the slice.
    #[inline]
    #[track_caller]
    pub fn rchunks(&self, chunk_size: usize) -> core::slice::RChunks<'_, T> {
        self.raw.rchunks(chunk_size)
    }

    /// Returns an iterator over `chunk_size` elements of the slice at a time, starting from the end of the slice.
    #[inline]
    #[track_caller]
    pub fn rchunks_mut(&mut self, chunk_size: usize) -> core::slice::RChunksMut<'_, T> {
        self.raw.rchunks_mut(chunk_size)
    }

    /// Returns an iterator over `chunk_size` elements of the slice at a time, starting from the end of the slice.
    #[inline]
    #[track_caller]
    pub fn rchunks_exact(&self, chunk_size: usize) -> core::slice::RChunksExact<'_, T> {
        self.raw.rchunks_exact(chunk_size)
    }

    /// Returns an iterator over `chunk_size` elements of the slice at a time, starting from the end of the slice.
    #[inline]
    #[track_caller]
    pub fn rchunks_exact_mut(&mut self, chunk_size: usize) -> core::slice::RChunksExactMut<'_, T> {
        self.raw.rchunks_exact_mut(chunk_size)
    }

    /// Returns an iterator over the slice, producing non-overlapping runs of elements using the predicate to separate them.
    #[inline]
    pub fn chunk_by<F>(&self, pred: F) -> core::slice::ChunkBy<'_, T, F>
    where
        F: FnMut(&T, &T) -> bool,
    {
        self.raw.chunk_by(pred)
    }

    /// Returns a mutable iterator over the slice, producing non-overlapping runs of elements using the predicate to separate them.
    #[inline]
    pub fn chunk_by_mut<F>(&mut self, pred: F) -> core::slice::ChunkByMut<'_, T, F>
    where
        F: FnMut(&T, &T) -> bool,
    {
        self.raw.chunk_by_mut(pred)
    }

    /// Divides one slice into two at an index.
    #[inline]
    #[track_caller]
    #[must_use]
    pub fn split_at(&self, mid: I) -> (&[T], &[T]) {
        self.raw.split_at(mid.to_index())
    }

    /// Divides one mutable slice into two at an index.
    #[inline]
    #[track_caller]
    #[must_use]
    pub fn split_at_mut(&mut self, mid: I) -> (&mut [T], &mut [T]) {
        self.raw.split_at_mut(mid.to_index())
    }

    /// Divides one slice into two at an index, without checking bounds.
    ///
    /// # Safety
    ///
    /// `mid` must be less than or equal to `self.len()`.
    #[inline]
    #[must_use]
    #[track_caller]
    pub unsafe fn split_at_unchecked(&self, mid: I) -> (&[T], &[T]) {
        // SAFETY: The caller guarantees that mid is within bounds.
        unsafe { self.raw.split_at_unchecked(mid.to_index()) }
    }

    /// Divides one mutable slice into two at an index, without checking bounds.
    ///
    /// # Safety
    ///
    /// `mid` must be less than or equal to `self.len()`.
    #[inline]
    #[must_use]
    #[track_caller]
    pub unsafe fn split_at_mut_unchecked(&mut self, mid: I) -> (&mut [T], &mut [T]) {
        // SAFETY: The caller guarantees that mid is within bounds.
        unsafe { self.raw.split_at_mut_unchecked(mid.to_index()) }
    }

    /// Divides one slice into two at an index, returning `None` if out of bounds.
    #[inline]
    #[must_use]
    pub fn split_at_checked(&self, mid: I) -> Option<(&[T], &[T])> {
        self.raw.split_at_checked(mid.to_index())
    }

    /// Divides one mutable slice into two at an index, returning `None` if out of bounds.
    #[inline]
    #[must_use]
    pub fn split_at_mut_checked(&mut self, mid: I) -> Option<(&mut [T], &mut [T])> {
        self.raw.split_at_mut_checked(mid.to_index())
    }

    /// Returns an iterator over subslices separated by elements that match `pred`.
    #[inline]
    pub fn split<F>(&self, pred: F) -> core::slice::Split<'_, T, F>
    where
        F: FnMut(&T) -> bool,
    {
        self.raw.split(pred)
    }

    /// Returns an iterator over mutable subslices separated by elements that match `pred`.
    #[inline]
    pub fn split_mut<F>(&mut self, pred: F) -> core::slice::SplitMut<'_, T, F>
    where
        F: FnMut(&T) -> bool,
    {
        self.raw.split_mut(pred)
    }

    /// Returns an iterator over subslices separated by elements that match `pred`, including the matched element.
    #[inline]
    pub fn split_inclusive<F>(&self, pred: F) -> core::slice::SplitInclusive<'_, T, F>
    where
        F: FnMut(&T) -> bool,
    {
        self.raw.split_inclusive(pred)
    }

    /// Returns an iterator over mutable subslices separated by elements that match `pred`, including the matched element.
    #[inline]
    pub fn split_inclusive_mut<F>(&mut self, pred: F) -> core::slice::SplitInclusiveMut<'_, T, F>
    where
        F: FnMut(&T) -> bool,
    {
        self.raw.split_inclusive_mut(pred)
    }

    /// Returns an iterator over subslices separated by elements that match `pred`, starting from the end.
    #[inline]
    pub fn rsplit<F>(&self, pred: F) -> core::slice::RSplit<'_, T, F>
    where
        F: FnMut(&T) -> bool,
    {
        self.raw.rsplit(pred)
    }

    /// Returns an iterator over mutable subslices separated by elements that match `pred`, starting from the end.
    #[inline]
    pub fn rsplit_mut<F>(&mut self, pred: F) -> core::slice::RSplitMut<'_, T, F>
    where
        F: FnMut(&T) -> bool,
    {
        self.raw.rsplit_mut(pred)
    }

    /// Returns an iterator over subslices separated by elements that match `pred`, limited to `n` splits.
    #[inline]
    pub fn splitn<F>(&self, n: usize, pred: F) -> core::slice::SplitN<'_, T, F>
    where
        F: FnMut(&T) -> bool,
    {
        self.raw.splitn(n, pred)
    }

    /// Returns an iterator over mutable subslices separated by elements that match `pred`, limited to `n` splits.
    #[inline]
    pub fn splitn_mut<F>(&mut self, n: usize, pred: F) -> core::slice::SplitNMut<'_, T, F>
    where
        F: FnMut(&T) -> bool,
    {
        self.raw.splitn_mut(n, pred)
    }

    /// Returns an iterator over subslices separated by elements that match `pred`, limited to `n` splits, starting from the end.
    #[inline]
    pub fn rsplitn<F>(&self, n: usize, pred: F) -> core::slice::RSplitN<'_, T, F>
    where
        F: FnMut(&T) -> bool,
    {
        self.raw.rsplitn(n, pred)
    }

    /// Returns an iterator over mutable subslices separated by elements that match `pred`, limited to `n` splits, starting from the end.
    #[inline]
    pub fn rsplitn_mut<F>(&mut self, n: usize, pred: F) -> core::slice::RSplitNMut<'_, T, F>
    where
        F: FnMut(&T) -> bool,
    {
        self.raw.rsplitn_mut(n, pred)
    }

    /// Returns `true` if the slice contains an element with the given value.
    #[inline]
    #[must_use]
    pub fn contains(&self, x: &T) -> bool
    where
        T: PartialEq,
    {
        self.raw.contains(x)
    }

    /// Returns `true` if `needle` is a prefix of the slice.
    #[inline]
    #[must_use]
    pub fn starts_with(&self, needle: &[T]) -> bool
    where
        T: PartialEq,
    {
        self.raw.starts_with(needle)
    }

    /// Returns `true` if `needle` is a suffix of the slice.
    #[inline]
    #[must_use]
    pub fn ends_with(&self, needle: &[T]) -> bool
    where
        T: PartialEq,
    {
        self.raw.ends_with(needle)
    }

    /// Binary searches this slice for a given element.
    #[inline]
    pub fn binary_search(&self, x: &T) -> Result<I, I>
    where
        T: Ord,
    {
        // SAFETY: The underlying result is a valid index into this slice, which fits in I.
        unsafe { typify_binary_search_res(self.raw.binary_search(x)) }
    }

    /// Binary searches this slice with a comparator function.
    #[inline]
    pub fn binary_search_by<'a, F>(&'a self, f: F) -> Result<I, I>
    where
        F: FnMut(&'a T) -> core::cmp::Ordering,
    {
        // SAFETY: The underlying result is a valid index into this slice, which fits in I.
        unsafe { typify_binary_search_res(self.raw.binary_search_by(f)) }
    }

    /// Binary searches this slice with a key extraction function.
    #[inline]
    pub fn binary_search_by_key<'a, B, F>(&'a self, b: &B, f: F) -> Result<I, I>
    where
        F: FnMut(&'a T) -> B,
        B: Ord,
    {
        // SAFETY: The underlying result is a valid index into this slice, which fits in I.
        unsafe { typify_binary_search_res(self.raw.binary_search_by_key(b, f)) }
    }

    /// Sorts the slice, but might not preserve the order of equal elements.
    #[inline]
    pub fn sort_unstable(&mut self)
    where
        T: Ord,
    {
        self.raw.sort_unstable()
    }

    /// Sorts the slice with a comparator function, but might not preserve the order of equal elements.
    #[inline]
    pub fn sort_unstable_by<F>(&mut self, compare: F)
    where
        F: FnMut(&T, &T) -> core::cmp::Ordering,
    {
        self.raw.sort_unstable_by(compare)
    }

    /// Sorts the slice with a key extraction function, but might not preserve the order of equal elements.
    #[inline]
    pub fn sort_unstable_by_key<K, F>(&mut self, f: F)
    where
        F: FnMut(&T) -> K,
        K: Ord,
    {
        self.raw.sort_unstable_by_key(f)
    }

    /// Reorder the slice such that the element at `index` is at its final sorted position.
    #[inline]
    pub fn select_nth_unstable(&mut self, index: I) -> (&mut [T], &mut T, &mut [T])
    where
        T: Ord,
    {
        self.raw.select_nth_unstable(index.to_index())
    }

    /// Reorder the slice with a comparator function such that the element at `index` is at its final sorted position.
    #[inline]
    pub fn select_nth_unstable_by<F>(
        &mut self,
        index: I,
        compare: F,
    ) -> (&mut [T], &mut T, &mut [T])
    where
        F: FnMut(&T, &T) -> core::cmp::Ordering,
    {
        self.raw.select_nth_unstable_by(index.to_index(), compare)
    }

    /// Reorder the slice with a key extraction function such that the element at `index` is at its final sorted position.
    #[inline]
    pub fn select_nth_unstable_by_key<K, F>(
        &mut self,
        index: I,
        f: F,
    ) -> (&mut [T], &mut T, &mut [T])
    where
        F: FnMut(&T) -> K,
        K: Ord,
    {
        self.raw.select_nth_unstable_by_key(index.to_index(), f)
    }

    /// Rotates the slice in-place such that the first `mid` elements of the slice move to the end while the rest move to the front.
    #[inline]
    pub fn rotate_left(&mut self, mid: I) {
        self.raw.rotate_left(mid.to_index())
    }

    /// Rotates the slice in-place such that the last `k` elements move to the front while the rest move to the end.
    #[inline]
    pub fn rotate_right(&mut self, k: I) {
        self.raw.rotate_right(k.to_index())
    }

    /// Fills the slice with `value`.
    #[inline]
    #[doc(alias = "memset")]
    pub fn fill(&mut self, value: T)
    where
        T: Clone,
    {
        self.raw.fill(value)
    }

    /// Fills the slice with elements returned by calling a closure repeatedly.
    #[inline]
    pub fn fill_with<F>(&mut self, f: F)
    where
        F: FnMut() -> T,
    {
        self.raw.fill_with(f)
    }

    /// Copies the elements from `src` into `self`.
    #[inline]
    #[track_caller]
    pub fn clone_from_slice(&mut self, src: &[T])
    where
        T: Clone,
    {
        self.raw.clone_from_slice(src)
    }

    /// Copies all elements from `src` into `self`, using a memmove.
    #[inline]
    #[doc(alias = "memcpy")]
    #[track_caller]
    pub const fn copy_from_slice(&mut self, src: &[T])
    where
        T: Copy,
    {
        self.raw.copy_from_slice(src)
    }

    /// Copies elements from one part of the slice to another part of itself, using a memmove.
    #[inline]
    #[track_caller]
    pub fn copy_within<R: core::ops::RangeBounds<I>>(&mut self, src: R, dest: I)
    where
        T: Copy,
    {
        let raw_bounds = (
            src.start_bound().map(|x| x.to_index()),
            src.end_bound().map(|x| x.to_index()),
        );
        self.raw.copy_within(raw_bounds, dest.to_index())
    }

    /// Swaps all elements in `self` with those in `other`.
    #[inline]
    #[track_caller]
    pub fn swap_with_slice(&mut self, other: &mut [T]) {
        self.raw.swap_with_slice(other)
    }

    /// Transmutes the slice to a slice of another type, ensuring alignment and size.
    ///
    /// # Safety
    ///
    /// This method is essentially a `mem::transmute` for slices.
    #[inline]
    #[must_use]
    pub unsafe fn align_to<U>(&self) -> (&[T], &[U], &[T]) {
        // SAFETY: The caller must ensure that transmuting T to U is safe.
        unsafe { self.raw.align_to::<U>() }
    }

    /// Transmutes the slice to a mutable slice of another type, ensuring alignment and size.
    ///
    /// # Safety
    ///
    /// This method is essentially a `mem::transmute` for slices.
    #[inline]
    #[must_use]
    pub unsafe fn align_to_mut<U>(&mut self) -> (&mut [T], &mut [U], &mut [T]) {
        // SAFETY: The caller must ensure that transmuting T to U is safe.
        unsafe { self.raw.align_to_mut::<U>() }
    }

    /// Returns `true` if the slice is sorted.
    #[inline]
    #[must_use]
    pub fn is_sorted(&self) -> bool
    where
        T: PartialOrd,
    {
        self.raw.is_sorted()
    }

    /// Returns `true` if the slice is sorted using the given comparator.
    #[inline]
    #[must_use]
    pub fn is_sorted_by<'a, F>(&'a self, compare: F) -> bool
    where
        F: FnMut(&'a T, &'a T) -> bool,
    {
        self.raw.is_sorted_by(compare)
    }

    /// Returns `true` if the slice is sorted by key.
    #[inline]
    #[must_use]
    pub fn is_sorted_by_key<'a, F, K>(&'a self, f: F) -> bool
    where
        F: FnMut(&'a T) -> K,
        K: PartialOrd,
    {
        self.raw.is_sorted_by_key(f)
    }

    /// Returns the index of the first element of the slice for which `pred` returns `false`.
    #[inline]
    #[must_use]
    pub fn partition_point<P>(&self, pred: P) -> I
    where
        P: FnMut(&T) -> bool,
    {
        // SAFETY: The partition point is a valid index into this slice, which fits in I.
        unsafe { I::from_index_unchecked(self.raw.partition_point(pred)) }
    }

    /// Removes the first element from the slice and returns it, or `None` if it is empty.
    #[inline]
    pub fn split_off_first<'a>(self: &mut &'a Self) -> Option<&'a T> {
        // SAFETY: TypedSlice is repr(transparent) over [T].
        let raw_self: &mut &'a [T] = unsafe { core::mem::transmute(self) };
        raw_self.split_off_first()
    }

    /// Removes the first element from the mutable slice and returns it, or `None` if it is empty.
    #[inline]
    pub fn split_off_first_mut<'a>(self: &mut &'a mut Self) -> Option<&'a mut T> {
        // SAFETY: TypedSlice is repr(transparent) over [T].
        let raw_self: &mut &'a mut [T] = unsafe { core::mem::transmute(self) };
        raw_self.split_off_first_mut()
    }

    /// Removes the last element from the slice and returns it, or `None` if it is empty.
    #[inline]
    pub fn split_off_last<'a>(self: &mut &'a Self) -> Option<&'a T> {
        // SAFETY: TypedSlice is repr(transparent) over [T].
        let raw_self: &mut &'a [T] = unsafe { core::mem::transmute(self) };
        raw_self.split_off_last()
    }

    /// Removes the last element from the mutable slice and returns it, or `None` if it is empty.
    #[inline]
    pub fn split_off_last_mut<'a>(self: &mut &'a mut Self) -> Option<&'a mut T> {
        // SAFETY: TypedSlice is repr(transparent) over [T].
        let raw_self: &mut &'a mut [T] = unsafe { core::mem::transmute(self) };
        raw_self.split_off_last_mut()
    }

    /// Gets multiple mutable references to elements or subslices.
    ///
    /// Returns an error if any indices are out of bounds or overlapping.
    #[inline]
    pub fn get_disjoint_mut<X, const N: usize>(
        &mut self,
        indices: [X; N],
    ) -> Result<[&mut X::Output; N], GetDisjointMutError>
    where
        X: GetDisjointMutTypedIndex + TypedSliceIndex<Self>,
    {
        get_disjoint_check_valid(&indices, self.len_usize())?;
        // SAFETY: get_disjoint_check_valid ensures indices are in bounds and not overlapping.
        unsafe { Ok(self.get_disjoint_unchecked_mut(indices)) }
    }

    /// Gets multiple mutable references to elements or subslices without checking bounds or overlap.
    ///
    /// # Safety
    ///
    /// Indices must be within bounds and non-overlapping.
    #[inline]
    #[track_caller]
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
            // SAFETY: The caller guarantees that indices are in bounds and non-overlapping.
            // Multiple mutable references are safe as they don't overlap.
            unsafe { arr_ptr.add(i).write(&mut *idx.get_unchecked_mut(slice)) }
        }
        // SAFETY: The array was fully initialized in the loop.
        unsafe { arr.assume_init() }
    }
}

impl<I: IndexType, T: PartialEq> PartialEq for TypedSlice<I, T> {
    fn eq(&self, other: &Self) -> bool {
        PartialEq::eq(&self.raw, &other.raw)
    }
}

impl<I: IndexType, T: Eq> Eq for TypedSlice<I, T> {}

impl<I: IndexType, T: core::hash::Hash> core::hash::Hash for TypedSlice<I, T> {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.raw.hash(state);
    }
}

impl<I: IndexType, T: core::fmt::Debug> core::fmt::Debug for TypedSlice<I, T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.raw.fmt(f)
    }
}

impl<I: IndexType, T, X: TypedSliceIndex<TypedSlice<I, T>>> Index<X> for TypedSlice<I, T> {
    type Output = X::Output;

    fn index(&self, index: X) -> &Self::Output {
        index.index(self)
    }
}

impl<I: IndexType, T, X: TypedSliceIndex<TypedSlice<I, T>>> IndexMut<X> for TypedSlice<I, T> {
    fn index_mut(&mut self, index: X) -> &mut Self::Output {
        index.index_mut(self)
    }
}

/// Converts a `Result<usize, usize>` from a binary search into a `Result<I, I>`.
///
/// # Safety
///
/// The values in the `Result` must be valid indices or insertion points for the slice,
/// which must fit in `I`.
unsafe fn typify_binary_search_res<I: IndexType>(res: Result<usize, usize>) -> Result<I, I> {
    match res {
        // SAFETY: The caller guarantees the index fits in I.
        Ok(v) => Ok(unsafe { I::from_index_unchecked(v) }),
        // SAFETY: The caller guarantees the index fits in I.
        Err(v) => Err(unsafe { I::from_index_unchecked(v) }),
    }
}

mod private_get_disjoint_mut_typed_index {
    pub trait Sealed {}
}

/// A trait for indices that can be used with [`TypedSlice::get_disjoint_mut`].
///
/// # Safety
///
/// Implementations must correctly report if they are in bounds and if they overlap with another index.
pub unsafe trait GetDisjointMutTypedIndex:
    private_get_disjoint_mut_typed_index::Sealed
{
    /// Returns `true` if this index is within bounds of a slice of length `len`.
    fn is_in_bounds(&self, len: usize) -> bool;

    /// Returns `true` if this index overlaps with another.
    fn is_overlapping(&self, other: &Self) -> bool;
}

impl<I: IndexType> private_get_disjoint_mut_typed_index::Sealed for I {}
// SAFETY: Correctly implements bounds check and equality check.
unsafe impl<I: IndexType> GetDisjointMutTypedIndex for I {
    #[inline]
    fn is_in_bounds(&self, len: usize) -> bool {
        self.to_index() < len
    }

    #[inline]
    fn is_overlapping(&self, other: &Self) -> bool {
        *self == *other
    }
}

impl<I: IndexType> private_get_disjoint_mut_typed_index::Sealed for core::ops::Range<I> {}
// SAFETY: Correctly implements bounds check and intersection check for ranges.
unsafe impl<I: IndexType> GetDisjointMutTypedIndex for core::ops::Range<I> {
    #[inline]
    fn is_in_bounds(&self, len: usize) -> bool {
        (self.start <= self.end) & (self.end.to_index() <= len)
    }

    #[inline]
    fn is_overlapping(&self, other: &Self) -> bool {
        (self.start < other.end) & (other.start < self.end)
    }
}

impl<I: IndexType> private_get_disjoint_mut_typed_index::Sealed for core::ops::RangeInclusive<I> {}
// SAFETY: Correctly implements bounds check and intersection check for inclusive ranges.
unsafe impl<I: IndexType> GetDisjointMutTypedIndex for core::ops::RangeInclusive<I> {
    #[inline]
    fn is_in_bounds(&self, len: usize) -> bool {
        (self.start() <= self.end()) & (self.end().to_index() < len)
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

/// Error returned by [`TypedSlice::get_disjoint_mut`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GetDisjointMutError {
    /// An index provided was out-of-bounds for the slice.
    IndexOutOfBounds,
    /// Two indices provided were overlapping.
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
