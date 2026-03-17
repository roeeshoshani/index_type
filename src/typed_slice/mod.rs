use core::marker::PhantomData;

use crate::{IndexTooBigError, IndexType};

mod index;

pub use index::TypedSliceIndex;

#[repr(transparent)]
pub struct TypedSlice<I: IndexType, T> {
    phantom: PhantomData<fn(&I)>,
    raw: [T],
}
impl<I: IndexType, T> TypedSlice<I, T> {
    pub fn from_slice(slice: &[T]) -> Result<&Self, IndexTooBigError> {
        let _ = I::try_from_index(slice.len())?;
        Ok(unsafe { core::mem::transmute(slice) })
    }

    pub fn from_slice_mut(slice: &mut [T]) -> Result<&mut Self, IndexTooBigError> {
        let _ = I::try_from_index(slice.len())?;
        Ok(unsafe { core::mem::transmute(slice) })
    }

    pub unsafe fn from_slice_unchecked(slice: &[T]) -> &Self {
        unsafe { core::mem::transmute(slice) }
    }

    pub unsafe fn from_slice_unchecked_mut(slice: &mut [T]) -> &mut Self {
        unsafe { core::mem::transmute(slice) }
    }
}
impl<I: IndexType, T> TypedSlice<I, T> {
    #[inline]
    #[must_use]
    pub const fn len(&self) -> usize {
        self.raw.len()
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
    #[track_caller]
    pub unsafe fn get_unchecked<X>(&self, index: X) -> &X::Output
    where
        X: TypedSliceIndex<Self>,
    {
        unsafe { &*index.get_unchecked(self) }
    }

    #[inline]
    #[must_use]
    #[track_caller]
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
    #[track_caller]
    pub fn swap(&mut self, a: I, b: I) {
        self.raw.swap(a.to_index(), b.to_index());
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
    #[track_caller]
    pub fn windows(&self, size: usize) -> core::slice::Windows<'_, T> {
        self.raw.windows(size)
    }

    #[inline]
    #[track_caller]
    pub fn chunks(&self, chunk_size: usize) -> core::slice::Chunks<'_, T> {
        self.raw.chunks(chunk_size)
    }

    #[inline]
    #[track_caller]
    pub fn chunks_mut(&mut self, chunk_size: usize) -> core::slice::ChunksMut<'_, T> {
        self.raw.chunks_mut(chunk_size)
    }

    #[inline]
    #[track_caller]
    pub fn chunks_exact(&self, chunk_size: usize) -> core::slice::ChunksExact<'_, T> {
        self.raw.chunks_exact(chunk_size)
    }

    #[inline]
    #[track_caller]
    pub fn chunks_exact_mut(&mut self, chunk_size: usize) -> core::slice::ChunksExactMut<'_, T> {
        self.raw.chunks_exact_mut(chunk_size)
    }

    #[inline]
    #[must_use]
    #[track_caller]
    pub const unsafe fn as_chunks_unchecked<const N: usize>(&self) -> &[[T; N]] {
        unsafe { self.raw.as_chunks_unchecked() }
    }

    #[inline]
    #[track_caller]
    #[must_use]
    pub const fn as_chunks<const N: usize>(&self) -> (&[[T; N]], &[T]) {
        self.raw.as_chunks()
    }

    #[inline]
    #[track_caller]
    #[must_use]
    pub const fn as_rchunks<const N: usize>(&self) -> (&[T], &[[T; N]]) {
        self.raw.as_rchunks()
    }

    #[inline]
    #[must_use]
    #[track_caller]
    pub const unsafe fn as_chunks_unchecked_mut<const N: usize>(&mut self) -> &mut [[T; N]] {
        unsafe { self.raw.as_chunks_unchecked_mut() }
    }

    #[inline]
    #[track_caller]
    #[must_use]
    pub const fn as_chunks_mut<const N: usize>(&mut self) -> (&mut [[T; N]], &mut [T]) {
        self.raw.as_chunks_mut()
    }

    #[inline]
    #[track_caller]
    #[must_use]
    pub const fn as_rchunks_mut<const N: usize>(&mut self) -> (&mut [T], &mut [[T; N]]) {
        self.raw.as_rchunks_mut()
    }

    #[inline]
    #[track_caller]
    pub fn rchunks(&self, chunk_size: usize) -> core::slice::RChunks<'_, T> {
        self.raw.rchunks(chunk_size)
    }

    #[inline]
    #[track_caller]
    pub fn rchunks_mut(&mut self, chunk_size: usize) -> core::slice::RChunksMut<'_, T> {
        self.raw.rchunks_mut(chunk_size)
    }

    #[inline]
    #[track_caller]
    pub fn rchunks_exact(&self, chunk_size: usize) -> core::slice::RChunksExact<'_, T> {
        self.raw.rchunks_exact(chunk_size)
    }

    #[inline]
    #[track_caller]
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
    #[track_caller]
    #[must_use]
    pub fn split_at(&self, mid: I) -> (&[T], &[T]) {
        self.raw.split_at(mid.to_index())
    }

    #[inline]
    #[track_caller]
    #[must_use]
    pub fn split_at_mut(&mut self, mid: I) -> (&mut [T], &mut [T]) {
        self.raw.split_at_mut(mid.to_index())
    }

    #[inline]
    #[must_use]
    #[track_caller]
    pub unsafe fn split_at_unchecked(&self, mid: I) -> (&[T], &[T]) {
        unsafe { self.raw.split_at_unchecked(mid.to_index()) }
    }

    #[inline]
    #[must_use]
    #[track_caller]
    pub unsafe fn split_at_mut_unchecked(&mut self, mid: I) -> (&mut [T], &mut [T]) {
        unsafe { self.raw.split_at_mut_unchecked(mid.to_index()) }
    }

    #[inline]
    #[must_use]
    pub fn split_at_checked(&self, mid: I) -> Option<(&[T], &[T])> {
        self.raw.split_at_checked(mid.to_index())
    }

    #[inline]
    #[must_use]
    pub fn split_at_mut_checked(&mut self, mid: I) -> Option<(&mut [T], &mut [T])> {
        self.raw.split_at_mut_checked(mid.to_index())
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
        self.raw.select_nth_unstable(index.to_index())
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
        self.raw.select_nth_unstable_by(index.to_index(), compare)
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
        self.raw.select_nth_unstable_by_key(index.to_index(), f)
    }

    #[inline]
    pub fn rotate_left(&mut self, mid: I) {
        self.raw.rotate_left(mid.to_index())
    }

    #[inline]
    pub fn rotate_right(&mut self, k: I) {
        self.raw.rotate_right(k.to_index())
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
    #[track_caller]
    pub fn clone_from_slice(&mut self, src: &[T])
    where
        T: Clone,
    {
        self.raw.clone_from_slice(src)
    }

    #[inline]
    #[doc(alias = "memcpy")]
    #[track_caller]
    pub const fn copy_from_slice(&mut self, src: &[T])
    where
        T: Copy,
    {
        self.raw.copy_from_slice(src)
    }

    #[inline]
    #[track_caller]
    pub fn copy_within<R: core::ops::RangeBounds<I>>(&mut self, src: R, dest: usize)
    where
        T: Copy,
    {
        let raw_bounds = (
            src.start_bound().map(|x| x.to_index()),
            src.end_bound().map(|x| x.to_index()),
        );
        self.raw.copy_within(raw_bounds, dest)
    }

    #[inline]
    #[track_caller]
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
        unsafe { I::from_index_unchecked(self.raw.partition_point(pred)) }
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

unsafe fn typify_binary_search_res<I: IndexType>(res: Result<usize, usize>) -> Result<I, I> {
    match res {
        Ok(v) => Ok(unsafe { I::from_index_unchecked(v) }),
        Err(v) => Err(unsafe { I::from_index_unchecked(v) }),
    }
}
