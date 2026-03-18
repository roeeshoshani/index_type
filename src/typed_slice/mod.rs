use core::{
    marker::PhantomData,
    mem::MaybeUninit,
    ops::{Index, IndexMut},
};

use crate::{IndexTooBigError, IndexType, utils::range_bounds_to_raw};

mod index;

pub use index::TypedSliceIndex;

#[repr(transparent)]
pub struct TypedSlice<I: IndexType, T> {
    phantom: PhantomData<fn(&I)>,
    raw: [T],
}

impl<I: IndexType, T> TypedSlice<I, T> {
    #[inline]
    pub fn try_from_slice(slice: &[T]) -> Result<&Self, IndexTooBigError> {
        let _ = I::try_from_raw_index(slice.len())?;
        Ok(unsafe { core::mem::transmute(slice) })
    }

    #[inline]
    pub fn try_from_slice_mut(slice: &mut [T]) -> Result<&mut Self, IndexTooBigError> {
        let _ = I::try_from_raw_index(slice.len())?;
        Ok(unsafe { core::mem::transmute(slice) })
    }

    #[inline]
    pub unsafe fn from_raw_parts<'a>(data: *const T, len: I) -> &'a TypedSlice<I, T> {
        let slice = unsafe { core::slice::from_raw_parts(data, len.to_raw_index()) };
        unsafe { Self::from_slice_unchecked(slice) }
    }

    #[inline]
    pub unsafe fn from_raw_parts_mut<'a>(data: *mut T, len: I) -> &'a mut TypedSlice<I, T> {
        let slice = unsafe { core::slice::from_raw_parts_mut(data, len.to_raw_index()) };
        unsafe { Self::from_slice_unchecked_mut(slice) }
    }

    #[inline]
    pub const unsafe fn from_slice_unchecked(slice: &[T]) -> &Self {
        unsafe { core::mem::transmute(slice) }
    }

    #[inline]
    pub const unsafe fn from_slice_unchecked_mut(slice: &mut [T]) -> &mut Self {
        unsafe { core::mem::transmute(slice) }
    }

    #[inline]
    pub const fn to_slice(&self) -> &[T] {
        unsafe { core::mem::transmute(self) }
    }

    #[inline]
    pub const fn to_slice_mut(&mut self) -> &mut [T] {
        unsafe { core::mem::transmute(self) }
    }
}

// methods copied from stdlib's slice implementation
impl<I: IndexType, T> TypedSlice<I, T> {
    #[inline]
    #[must_use]
    pub const fn len_usize(&self) -> usize {
        self.raw.len()
    }

    #[inline]
    #[must_use]
    pub fn len(&self) -> I {
        unsafe { I::from_raw_index_unchecked(self.raw.len()) }
    }

    #[inline]
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.raw.is_empty()
    }

    #[inline]
    #[must_use]
    pub const fn first(&self) -> Option<&T> {
        self.raw.first()
    }

    #[inline]
    #[must_use]
    pub const fn first_mut(&mut self) -> Option<&mut T> {
        self.raw.first_mut()
    }

    #[inline]
    #[must_use]
    pub const fn split_first(&self) -> Option<(&T, &[T])> {
        self.raw.split_first()
    }

    #[inline]
    #[must_use]
    pub const fn split_first_mut(&mut self) -> Option<(&mut T, &mut [T])> {
        self.raw.split_first_mut()
    }

    #[inline]
    #[must_use]
    pub const fn split_last(&self) -> Option<(&T, &[T])> {
        self.raw.split_last()
    }

    #[inline]
    #[must_use]
    pub const fn split_last_mut(&mut self) -> Option<(&mut T, &mut [T])> {
        self.raw.split_last_mut()
    }

    #[inline]
    #[must_use]
    pub const fn last(&self) -> Option<&T> {
        self.raw.last()
    }

    #[inline]
    #[must_use]
    pub const fn last_mut(&mut self) -> Option<&mut T> {
        self.raw.last_mut()
    }

    #[inline]
    pub const fn first_chunk<const N: usize>(&self) -> Option<&[T; N]> {
        self.raw.first_chunk()
    }

    #[inline]
    pub const fn first_chunk_mut<const N: usize>(&mut self) -> Option<&mut [T; N]> {
        self.raw.first_chunk_mut()
    }

    #[inline]
    pub const fn split_first_chunk<const N: usize>(&self) -> Option<(&[T; N], &[T])> {
        self.raw.split_first_chunk()
    }

    #[inline]
    pub const fn split_first_chunk_mut<const N: usize>(
        &mut self,
    ) -> Option<(&mut [T; N], &mut [T])> {
        self.raw.split_first_chunk_mut()
    }
    #[inline]
    pub const fn split_last_chunk<const N: usize>(&self) -> Option<(&[T], &[T; N])> {
        self.raw.split_last_chunk()
    }

    #[inline]
    pub const fn split_last_chunk_mut<const N: usize>(
        &mut self,
    ) -> Option<(&mut [T], &mut [T; N])> {
        self.raw.split_last_chunk_mut()
    }

    #[inline]
    pub const fn last_chunk<const N: usize>(&self) -> Option<&[T; N]> {
        self.raw.last_chunk()
    }

    #[inline]
    pub const fn last_chunk_mut<const N: usize>(&mut self) -> Option<&mut [T; N]> {
        self.raw.last_chunk_mut()
    }

    #[inline]
    #[must_use]
    pub fn get<X>(&self, index: X) -> Option<&X::Output>
    where
        X: TypedSliceIndex<Self>,
    {
        index.get(self)
    }

    #[inline]
    #[must_use]
    pub fn get_mut<X>(&mut self, index: X) -> Option<&mut X::Output>
    where
        X: TypedSliceIndex<Self>,
    {
        index.get_mut(self)
    }

    #[inline]
    #[must_use]
    pub unsafe fn get_unchecked<X>(&self, index: X) -> &X::Output
    where
        X: TypedSliceIndex<Self>,
    {
        unsafe { &*index.get_unchecked(self) }
    }

    #[inline]
    #[must_use]
    pub unsafe fn get_unchecked_mut<X>(&mut self, index: X) -> &mut X::Output
    where
        X: TypedSliceIndex<Self>,
    {
        unsafe { &mut *index.get_unchecked_mut(self) }
    }

    #[inline(always)]
    #[must_use]
    pub const fn as_ptr(&self) -> *const T {
        self.raw.as_ptr()
    }

    #[inline(always)]
    #[must_use]
    pub const fn as_mut_ptr(&mut self) -> *mut T {
        self.raw.as_mut_ptr()
    }

    #[inline]
    #[must_use]
    pub const fn as_ptr_range(&self) -> core::ops::Range<*const T> {
        self.raw.as_ptr_range()
    }

    #[inline]
    #[must_use]
    pub const fn as_mut_ptr_range(&mut self) -> core::ops::Range<*mut T> {
        self.raw.as_mut_ptr_range()
    }

    #[inline]
    #[must_use]
    pub const fn as_array<const N: usize>(&self) -> Option<&[T; N]> {
        self.raw.as_array()
    }

    #[inline]
    #[must_use]
    pub const fn as_mut_array<const N: usize>(&mut self) -> Option<&mut [T; N]> {
        self.raw.as_mut_array()
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

    #[inline]
    pub fn windows(&self, size: usize) -> core::slice::Windows<'_, T> {
        self.raw.windows(size)
    }

    #[inline]
    pub fn chunks(&self, chunk_size: usize) -> core::slice::Chunks<'_, T> {
        self.raw.chunks(chunk_size)
    }

    #[inline]
    pub fn chunks_mut(&mut self, chunk_size: usize) -> core::slice::ChunksMut<'_, T> {
        self.raw.chunks_mut(chunk_size)
    }

    #[inline]
    pub fn chunks_exact(&self, chunk_size: usize) -> core::slice::ChunksExact<'_, T> {
        self.raw.chunks_exact(chunk_size)
    }

    #[inline]
    pub fn chunks_exact_mut(&mut self, chunk_size: usize) -> core::slice::ChunksExactMut<'_, T> {
        self.raw.chunks_exact_mut(chunk_size)
    }

    #[inline]
    #[must_use]
    pub const unsafe fn as_chunks_unchecked<const N: usize>(&self) -> &[[T; N]] {
        unsafe { self.raw.as_chunks_unchecked() }
    }

    #[inline]
    #[must_use]
    pub const fn as_chunks<const N: usize>(&self) -> (&[[T; N]], &[T]) {
        self.raw.as_chunks()
    }

    #[inline]
    #[must_use]
    pub const fn as_rchunks<const N: usize>(&self) -> (&[T], &[[T; N]]) {
        self.raw.as_rchunks()
    }

    #[inline]
    #[must_use]
    pub const unsafe fn as_chunks_unchecked_mut<const N: usize>(&mut self) -> &mut [[T; N]] {
        unsafe { self.raw.as_chunks_unchecked_mut() }
    }

    #[inline]
    #[must_use]
    pub const fn as_chunks_mut<const N: usize>(&mut self) -> (&mut [[T; N]], &mut [T]) {
        self.raw.as_chunks_mut()
    }

    #[inline]
    #[must_use]
    pub const fn as_rchunks_mut<const N: usize>(&mut self) -> (&mut [T], &mut [[T; N]]) {
        self.raw.as_rchunks_mut()
    }

    #[inline]
    pub fn rchunks(&self, chunk_size: usize) -> core::slice::RChunks<'_, T> {
        self.raw.rchunks(chunk_size)
    }

    #[inline]
    pub fn rchunks_mut(&mut self, chunk_size: usize) -> core::slice::RChunksMut<'_, T> {
        self.raw.rchunks_mut(chunk_size)
    }

    #[inline]
    pub fn rchunks_exact(&self, chunk_size: usize) -> core::slice::RChunksExact<'_, T> {
        self.raw.rchunks_exact(chunk_size)
    }

    #[inline]
    pub fn rchunks_exact_mut(&mut self, chunk_size: usize) -> core::slice::RChunksExactMut<'_, T> {
        self.raw.rchunks_exact_mut(chunk_size)
    }

    #[inline]
    pub fn chunk_by<F>(&self, pred: F) -> core::slice::ChunkBy<'_, T, F>
    where
        F: FnMut(&T, &T) -> bool,
    {
        self.raw.chunk_by(pred)
    }

    #[inline]
    pub fn chunk_by_mut<F>(&mut self, pred: F) -> core::slice::ChunkByMut<'_, T, F>
    where
        F: FnMut(&T, &T) -> bool,
    {
        self.raw.chunk_by_mut(pred)
    }

    #[inline]
    #[must_use]
    pub fn split_at(&self, mid: I) -> (&[T], &[T]) {
        self.raw.split_at(mid.to_raw_index())
    }

    #[inline]
    #[must_use]
    pub fn split_at_mut(&mut self, mid: I) -> (&mut [T], &mut [T]) {
        self.raw.split_at_mut(mid.to_raw_index())
    }

    #[inline]
    #[must_use]
    pub unsafe fn split_at_unchecked(&self, mid: I) -> (&[T], &[T]) {
        unsafe { self.raw.split_at_unchecked(mid.to_raw_index()) }
    }

    #[inline]
    #[must_use]
    pub unsafe fn split_at_mut_unchecked(&mut self, mid: I) -> (&mut [T], &mut [T]) {
        unsafe { self.raw.split_at_mut_unchecked(mid.to_raw_index()) }
    }

    #[inline]
    #[must_use]
    pub fn split_at_checked(&self, mid: I) -> Option<(&[T], &[T])> {
        self.raw.split_at_checked(mid.to_raw_index())
    }

    #[inline]
    #[must_use]
    pub fn split_at_mut_checked(&mut self, mid: I) -> Option<(&mut [T], &mut [T])> {
        self.raw.split_at_mut_checked(mid.to_raw_index())
    }

    #[inline]
    pub fn split<F>(&self, pred: F) -> core::slice::Split<'_, T, F>
    where
        F: FnMut(&T) -> bool,
    {
        self.raw.split(pred)
    }

    #[inline]
    pub fn split_mut<F>(&mut self, pred: F) -> core::slice::SplitMut<'_, T, F>
    where
        F: FnMut(&T) -> bool,
    {
        self.raw.split_mut(pred)
    }

    #[inline]
    pub fn split_inclusive<F>(&self, pred: F) -> core::slice::SplitInclusive<'_, T, F>
    where
        F: FnMut(&T) -> bool,
    {
        self.raw.split_inclusive(pred)
    }

    #[inline]
    pub fn split_inclusive_mut<F>(&mut self, pred: F) -> core::slice::SplitInclusiveMut<'_, T, F>
    where
        F: FnMut(&T) -> bool,
    {
        self.raw.split_inclusive_mut(pred)
    }

    #[inline]
    pub fn rsplit<F>(&self, pred: F) -> core::slice::RSplit<'_, T, F>
    where
        F: FnMut(&T) -> bool,
    {
        self.raw.rsplit(pred)
    }

    #[inline]
    pub fn rsplit_mut<F>(&mut self, pred: F) -> core::slice::RSplitMut<'_, T, F>
    where
        F: FnMut(&T) -> bool,
    {
        self.raw.rsplit_mut(pred)
    }

    #[inline]
    pub fn splitn<F>(&self, n: usize, pred: F) -> core::slice::SplitN<'_, T, F>
    where
        F: FnMut(&T) -> bool,
    {
        self.raw.splitn(n, pred)
    }

    #[inline]
    pub fn splitn_mut<F>(&mut self, n: usize, pred: F) -> core::slice::SplitNMut<'_, T, F>
    where
        F: FnMut(&T) -> bool,
    {
        self.raw.splitn_mut(n, pred)
    }

    #[inline]
    pub fn rsplitn<F>(&self, n: usize, pred: F) -> core::slice::RSplitN<'_, T, F>
    where
        F: FnMut(&T) -> bool,
    {
        self.raw.rsplitn(n, pred)
    }

    #[inline]
    pub fn rsplitn_mut<F>(&mut self, n: usize, pred: F) -> core::slice::RSplitNMut<'_, T, F>
    where
        F: FnMut(&T) -> bool,
    {
        self.raw.rsplitn_mut(n, pred)
    }

    #[inline]
    #[must_use]
    pub fn contains(&self, x: &T) -> bool
    where
        T: PartialEq,
    {
        self.raw.contains(x)
    }

    #[inline]
    #[must_use]
    pub fn starts_with(&self, needle: &[T]) -> bool
    where
        T: PartialEq,
    {
        self.raw.starts_with(needle)
    }

    #[inline]
    #[must_use]
    pub fn ends_with(&self, needle: &[T]) -> bool
    where
        T: PartialEq,
    {
        self.raw.ends_with(needle)
    }

    #[inline]
    pub fn binary_search(&self, x: &T) -> Result<I, I>
    where
        T: Ord,
    {
        unsafe { typify_binary_search_res(self.raw.binary_search(x)) }
    }

    #[inline]
    pub fn binary_search_by<'a, F>(&'a self, f: F) -> Result<I, I>
    where
        F: FnMut(&'a T) -> core::cmp::Ordering,
    {
        unsafe { typify_binary_search_res(self.raw.binary_search_by(f)) }
    }

    #[inline]
    pub fn binary_search_by_key<'a, B, F>(&'a self, b: &B, f: F) -> Result<I, I>
    where
        F: FnMut(&'a T) -> B,
        B: Ord,
    {
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
    pub fn select_nth_unstable(&mut self, index: I) -> (&mut [T], &mut T, &mut [T])
    where
        T: Ord,
    {
        self.raw.select_nth_unstable(index.to_raw_index())
    }

    #[inline]
    pub fn select_nth_unstable_by<F>(
        &mut self,
        index: I,
        compare: F,
    ) -> (&mut [T], &mut T, &mut [T])
    where
        F: FnMut(&T, &T) -> core::cmp::Ordering,
    {
        self.raw
            .select_nth_unstable_by(index.to_raw_index(), compare)
    }

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
        self.raw.select_nth_unstable_by_key(index.to_raw_index(), f)
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
    pub fn clone_from_slice(&mut self, src: &[T])
    where
        T: Clone,
    {
        self.raw.clone_from_slice(src)
    }

    #[inline]
    #[doc(alias = "memcpy")]
    pub const fn copy_from_slice(&mut self, src: &[T])
    where
        T: Copy,
    {
        self.raw.copy_from_slice(src)
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
    pub fn swap_with_slice(&mut self, other: &mut [T]) {
        self.raw.swap_with_slice(other)
    }

    #[inline]
    #[must_use]
    pub unsafe fn align_to<U>(&self) -> (&[T], &[U], &[T]) {
        unsafe { self.raw.align_to::<U>() }
    }

    #[inline]
    #[must_use]
    pub unsafe fn align_to_mut<U>(&mut self) -> (&mut [T], &mut [U], &mut [T]) {
        unsafe { self.raw.align_to_mut::<U>() }
    }

    #[inline]
    #[must_use]
    pub fn is_sorted(&self) -> bool
    where
        T: PartialOrd,
    {
        self.raw.is_sorted()
    }

    #[inline]
    #[must_use]
    pub fn is_sorted_by<'a, F>(&'a self, compare: F) -> bool
    where
        F: FnMut(&'a T, &'a T) -> bool,
    {
        self.raw.is_sorted_by(compare)
    }

    #[inline]
    #[must_use]
    pub fn is_sorted_by_key<'a, F, K>(&'a self, f: F) -> bool
    where
        F: FnMut(&'a T) -> K,
        K: PartialOrd,
    {
        self.raw.is_sorted_by_key(f)
    }

    #[inline]
    #[must_use]
    pub fn partition_point<P>(&self, pred: P) -> I
    where
        P: FnMut(&T) -> bool,
    {
        unsafe { I::from_raw_index_unchecked(self.raw.partition_point(pred)) }
    }

    #[inline]
    pub fn split_off_first<'a>(self: &mut &'a Self) -> Option<&'a T> {
        let raw_self: &mut &'a [T] = unsafe { core::mem::transmute(self) };
        raw_self.split_off_first()
    }

    #[inline]
    pub fn split_off_first_mut<'a>(self: &mut &'a mut Self) -> Option<&'a mut T> {
        let raw_self: &mut &'a mut [T] = unsafe { core::mem::transmute(self) };
        raw_self.split_off_first_mut()
    }

    #[inline]
    pub fn split_off_last<'a>(self: &mut &'a Self) -> Option<&'a T> {
        let raw_self: &mut &'a [T] = unsafe { core::mem::transmute(self) };
        raw_self.split_off_last()
    }

    #[inline]
    pub fn split_off_last_mut<'a>(self: &mut &'a mut Self) -> Option<&'a mut T> {
        let raw_self: &mut &'a mut [T] = unsafe { core::mem::transmute(self) };
        raw_self.split_off_last_mut()
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
            unsafe { arr_ptr.add(i).write(&mut *idx.get_unchecked_mut(slice)) }
        }
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

impl<'a, I: IndexType, T> TryFrom<&'a [T]> for &'a TypedSlice<I, T> {
    type Error = IndexTooBigError;

    fn try_from(value: &'a [T]) -> Result<Self, Self::Error> {
        TypedSlice::try_from_slice(value)
    }
}

impl<'a, I: IndexType, T> TryFrom<&'a mut [T]> for &'a mut TypedSlice<I, T> {
    type Error = IndexTooBigError;

    fn try_from(value: &'a mut [T]) -> Result<Self, Self::Error> {
        TypedSlice::try_from_slice_mut(value)
    }
}

unsafe fn typify_binary_search_res<I: IndexType>(res: Result<usize, usize>) -> Result<I, I> {
    match res {
        Ok(v) => Ok(unsafe { I::from_raw_index_unchecked(v) }),
        Err(v) => Err(unsafe { I::from_raw_index_unchecked(v) }),
    }
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
