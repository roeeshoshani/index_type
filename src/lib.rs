#![no_std]

use thiserror_no_std::Error;

extern crate alloc;

mod base_index_types;
pub mod typed_slice;
pub mod typed_vec;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Error)]
#[error("index too big")]
pub struct IndexTooBigError;

pub unsafe trait IndexType:
    Sized + Clone + Copy + PartialEq + Eq + PartialOrd + Ord
{
    const ZERO: Self;

    fn try_from_index(index: usize) -> Result<Self, IndexTooBigError>;

    unsafe fn from_index_unchecked(index: usize) -> Self;

    fn to_index(self) -> usize;

    unsafe fn unchecked_add_usize(self, rhs: usize) -> Self;

    unsafe fn unchecked_sub(self, rhs: Self) -> usize;
}
