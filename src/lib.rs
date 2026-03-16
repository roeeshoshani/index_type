#![no_std]

extern crate alloc;

pub mod typed_slice;
pub mod typed_vec;

pub struct IndexTooBigError;

pub unsafe trait IndexType:
    Sized + Clone + Copy + PartialEq + Eq + PartialOrd + Ord
{
    const ZERO: Self;

    fn try_from_index(index: usize) -> Result<Self, IndexTooBigError>;

    unsafe fn from_index_unchecked(index: usize) -> Self;

    fn to_index(self) -> usize;

    unsafe fn unchecked_sub(self, rhs: Self) -> Self;
}
unsafe impl IndexType for usize {
    const ZERO: Self = 0;

    #[inline(always)]
    fn try_from_index(index: usize) -> Result<Self, IndexTooBigError> {
        Ok(index)
    }

    #[inline(always)]
    unsafe fn from_index_unchecked(index: usize) -> Self {
        index
    }

    #[inline(always)]
    fn to_index(self) -> usize {
        self
    }

    #[inline(always)]
    unsafe fn unchecked_sub(self, rhs: Self) -> Self {
        unsafe { self.unchecked_sub(rhs) }
    }
}
