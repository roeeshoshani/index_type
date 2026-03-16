#![no_std]

extern crate alloc;

use crate::index_type_sealed::IndexTypeSealed;

pub mod typed_slice;
pub mod typed_vec;

pub struct IndexTooBigError;

// we seal it since improper implementations may lead to undefined behaviour.
mod index_type_sealed {
    pub trait IndexTypeSealed {}
}

pub trait IndexType:
    IndexTypeSealed + Sized + Clone + Copy + PartialEq + Eq + PartialOrd + Ord
{
    const ZERO: Self;

    fn try_from_index(index: usize) -> Result<Self, IndexTooBigError>;

    unsafe fn from_index_unchecked(index: usize) -> Self;

    fn to_index(self) -> usize;
}
impl IndexTypeSealed for usize {}
impl IndexType for usize {
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
}
