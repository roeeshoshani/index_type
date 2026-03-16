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

    fn get(self, slice: &TypedSlice<I, T>) -> Option<&Self::Output> {
        slice.raw.get(self.to_index())
    }

    fn get_mut(self, slice: &mut TypedSlice<I, T>) -> Option<&mut Self::Output> {
        slice.raw.get_mut(self.to_index())
    }

    unsafe fn get_unchecked(self, slice: *const TypedSlice<I, T>) -> *const Self::Output {
        let ptr = slice as *const T;
        unsafe { ptr.add(self.to_index()) }
    }

    unsafe fn get_unchecked_mut(self, slice: *mut TypedSlice<I, T>) -> *mut Self::Output {
        let ptr = slice as *mut T;
        unsafe { ptr.add(self.to_index()) }
    }

    fn index(self, slice: &TypedSlice<I, T>) -> &Self::Output {
        &slice.raw[self.to_index()]
    }

    fn index_mut(self, slice: &mut TypedSlice<I, T>) -> &mut Self::Output {
        &mut slice.raw[self.to_index()]
    }
}

impl<I: IndexType> private_typed_slice_index::Sealed for core::ops::Range<I> {}
unsafe impl<I: IndexType, T> TypedSliceIndex<TypedSlice<I, T>> for core::ops::Range<I> {
    type Output = TypedSlice<I, T>;

    fn get(self, slice: &TypedSlice<I, T>) -> Option<&Self::Output> {
        let raw_range = self.start.to_index()..self.end.to_index();
        slice.raw.get(raw_range).map(TypedSlice::from_slice)
    }

    fn get_mut(self, slice: &mut TypedSlice<I, T>) -> Option<&mut Self::Output> {
        let raw_range = self.start.to_index()..self.end.to_index();
        slice.raw.get_mut(raw_range).map(TypedSlice::from_slice_mut)
    }

    unsafe fn get_unchecked(self, slice: *const TypedSlice<I, T>) -> *const Self::Output {
        let raw_range = self.start.to_index()..self.end.to_index();
        let ptr = slice as *const T;
        unsafe {
            let new_len = raw_range.end.unchecked_sub(raw_range.start);
            core::ptr::slice_from_raw_parts(ptr.add(raw_range.start), new_len) as _
        }
    }

    unsafe fn get_unchecked_mut(self, slice: *mut TypedSlice<I, T>) -> *mut Self::Output {
        let raw_range = self.start.to_index()..self.end.to_index();
        let ptr = slice as *mut T;
        unsafe {
            let new_len = raw_range.end.unchecked_sub(raw_range.start);
            core::ptr::slice_from_raw_parts_mut(ptr.add(raw_range.start), new_len) as _
        }
    }

    fn index(self, slice: &TypedSlice<I, T>) -> &Self::Output {
        let raw_range = self.start.to_index()..self.end.to_index();
        TypedSlice::from_slice(&slice.raw[raw_range])
    }

    fn index_mut(self, slice: &mut TypedSlice<I, T>) -> &mut Self::Output {
        let raw_range = self.start.to_index()..self.end.to_index();
        TypedSlice::from_slice_mut(&mut slice.raw[raw_range])
    }
}
