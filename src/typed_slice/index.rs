use crate::{IndexType, typed_slice::TypedSlice};

mod private_typed_slice_index {
    pub trait Sealed {}
}

pub unsafe trait TypedSliceIndex<T: ?Sized>: private_typed_slice_index::Sealed {
    type Output: ?Sized;

    fn get(self, slice: &T) -> Option<&Self::Output>;

    fn get_mut(self, slice: &mut T) -> Option<&mut Self::Output>;

    unsafe fn get_unchecked(self, slice: *const T) -> *const Self::Output;

    unsafe fn get_unchecked_mut(self, slice: *mut T) -> *mut Self::Output;

    #[track_caller]
    fn index(self, slice: &T) -> &Self::Output;

    #[track_caller]
    fn index_mut(self, slice: &mut T) -> &mut Self::Output;
}

impl<I: IndexType> private_typed_slice_index::Sealed for I {}
unsafe impl<I: IndexType, T> TypedSliceIndex<TypedSlice<I, T>> for I {
    type Output = T;

    #[inline]
    fn get(self, slice: &TypedSlice<I, T>) -> Option<&Self::Output> {
        slice.raw.get(self.to_index())
    }

    #[inline]
    fn get_mut(self, slice: &mut TypedSlice<I, T>) -> Option<&mut Self::Output> {
        slice.raw.get_mut(self.to_index())
    }

    #[inline]
    unsafe fn get_unchecked(self, slice: *const TypedSlice<I, T>) -> *const Self::Output {
        let ptr = slice as *const T;
        unsafe { ptr.add(self.to_index()) }
    }

    #[inline]
    unsafe fn get_unchecked_mut(self, slice: *mut TypedSlice<I, T>) -> *mut Self::Output {
        let ptr = slice as *mut T;
        unsafe { ptr.add(self.to_index()) }
    }

    #[inline]
    fn index(self, slice: &TypedSlice<I, T>) -> &Self::Output {
        &slice.raw[self.to_index()]
    }

    #[inline]
    fn index_mut(self, slice: &mut TypedSlice<I, T>) -> &mut Self::Output {
        &mut slice.raw[self.to_index()]
    }
}

impl<I: IndexType> private_typed_slice_index::Sealed for core::ops::Range<I> {}
unsafe impl<I: IndexType, T> TypedSliceIndex<TypedSlice<I, T>> for core::ops::Range<I> {
    type Output = TypedSlice<I, T>;

    #[inline]
    fn get(self, slice: &TypedSlice<I, T>) -> Option<&Self::Output> {
        let raw_range = self.start.to_index()..self.end.to_index();
        slice
            .raw
            .get(raw_range)
            .map(|new_slice| unsafe { TypedSlice::from_slice_unchecked(new_slice) })
    }

    #[inline]
    fn get_mut(self, slice: &mut TypedSlice<I, T>) -> Option<&mut Self::Output> {
        let raw_range = self.start.to_index()..self.end.to_index();
        slice
            .raw
            .get_mut(raw_range)
            .map(|new_slice| unsafe { TypedSlice::from_slice_unchecked_mut(new_slice) })
    }

    #[inline]
    unsafe fn get_unchecked(self, slice: *const TypedSlice<I, T>) -> *const Self::Output {
        let raw_range = self.start.to_index()..self.end.to_index();
        let ptr = slice as *const T;
        unsafe {
            let new_len = raw_range.end.unchecked_sub(raw_range.start);
            core::ptr::slice_from_raw_parts(ptr.add(raw_range.start), new_len) as _
        }
    }

    #[inline]
    unsafe fn get_unchecked_mut(self, slice: *mut TypedSlice<I, T>) -> *mut Self::Output {
        let raw_range = self.start.to_index()..self.end.to_index();
        let ptr = slice as *mut T;
        unsafe {
            let new_len = raw_range.end.unchecked_sub(raw_range.start);
            core::ptr::slice_from_raw_parts_mut(ptr.add(raw_range.start), new_len) as _
        }
    }

    #[inline]
    fn index(self, slice: &TypedSlice<I, T>) -> &Self::Output {
        let raw_range = self.start.to_index()..self.end.to_index();
        unsafe { TypedSlice::from_slice_unchecked(&slice.raw[raw_range]) }
    }

    #[inline]
    fn index_mut(self, slice: &mut TypedSlice<I, T>) -> &mut Self::Output {
        let raw_range = self.start.to_index()..self.end.to_index();
        unsafe { TypedSlice::from_slice_unchecked_mut(&mut slice.raw[raw_range]) }
    }
}

impl<I: IndexType> private_typed_slice_index::Sealed for core::ops::RangeTo<I> {}
unsafe impl<I: IndexType, T> TypedSliceIndex<TypedSlice<I, T>> for core::ops::RangeTo<I> {
    type Output = TypedSlice<I, T>;

    #[inline]
    fn get(self, slice: &TypedSlice<I, T>) -> Option<&TypedSlice<I, T>> {
        (I::ZERO..self.end).get(slice)
    }

    #[inline]
    fn get_mut(self, slice: &mut TypedSlice<I, T>) -> Option<&mut TypedSlice<I, T>> {
        (I::ZERO..self.end).get_mut(slice)
    }

    #[inline]
    unsafe fn get_unchecked(self, slice: *const TypedSlice<I, T>) -> *const TypedSlice<I, T> {
        unsafe { (I::ZERO..self.end).get_unchecked(slice) }
    }

    #[inline]
    unsafe fn get_unchecked_mut(self, slice: *mut TypedSlice<I, T>) -> *mut TypedSlice<I, T> {
        unsafe { (I::ZERO..self.end).get_unchecked_mut(slice) }
    }

    #[inline(always)]
    fn index(self, slice: &TypedSlice<I, T>) -> &TypedSlice<I, T> {
        (I::ZERO..self.end).index(slice)
    }

    #[inline]
    fn index_mut(self, slice: &mut TypedSlice<I, T>) -> &mut TypedSlice<I, T> {
        (I::ZERO..self.end).index_mut(slice)
    }
}

impl<I: IndexType> private_typed_slice_index::Sealed for core::ops::RangeFrom<I> {}
unsafe impl<I: IndexType, T> TypedSliceIndex<TypedSlice<I, T>> for core::ops::RangeFrom<I> {
    type Output = TypedSlice<I, T>;

    #[inline]
    fn get(self, slice: &TypedSlice<I, T>) -> Option<&TypedSlice<I, T>> {
        let len = unsafe { I::from_index_unchecked(slice.len()) };
        (self.start..len).get(slice)
    }

    #[inline]
    fn get_mut(self, slice: &mut TypedSlice<I, T>) -> Option<&mut TypedSlice<I, T>> {
        let len = unsafe { I::from_index_unchecked(slice.len()) };
        (self.start..len).get_mut(slice)
    }

    #[inline]
    unsafe fn get_unchecked(self, slice: *const TypedSlice<I, T>) -> *const TypedSlice<I, T> {
        let raw_slice = slice as *const [T];
        let len = unsafe { I::from_index_unchecked(raw_slice.len()) };
        unsafe { (self.start..len).get_unchecked(slice) }
    }

    #[inline]
    unsafe fn get_unchecked_mut(self, slice: *mut TypedSlice<I, T>) -> *mut TypedSlice<I, T> {
        let raw_slice = slice as *mut [T];
        let len = unsafe { I::from_index_unchecked(raw_slice.len()) };
        unsafe { (self.start..len).get_unchecked_mut(slice) }
    }

    #[inline(always)]
    fn index(self, slice: &TypedSlice<I, T>) -> &TypedSlice<I, T> {
        let len = unsafe { I::from_index_unchecked(slice.len()) };
        (self.start..len).index(slice)
    }

    #[inline]
    fn index_mut(self, slice: &mut TypedSlice<I, T>) -> &mut TypedSlice<I, T> {
        let len = unsafe { I::from_index_unchecked(slice.len()) };
        (self.start..len).index_mut(slice)
    }
}
