#![no_std]

pub use crate::error::IndexTooBigError;

extern crate alloc;

mod base_index_types;
mod error;
mod index_scalar_types;
pub mod typed_slice;
pub mod typed_vec;

pub unsafe trait IndexType:
    Sized + Clone + Copy + PartialEq + Eq + PartialOrd + Ord
{
    type Scalar: IndexScalarType;

    const ZERO: Self;

    fn try_from_raw_index(index: usize) -> Result<Self, IndexTooBigError>;
    unsafe fn from_raw_index_unchecked(index: usize) -> Self;
    fn to_raw_index(self) -> usize;

    fn try_from_scalar(scalar: Self::Scalar) -> Result<Self, IndexTooBigError>;
    unsafe fn from_scalar_unchecked(scalar: Self::Scalar) -> Self;
    fn to_scalar(self) -> Self::Scalar;

    fn checked_add_scalar(self, rhs: Self::Scalar) -> Result<Self, IndexTooBigError>;
    unsafe fn unchecked_add_scalar(self, rhs: Self::Scalar) -> Self;
    unsafe fn unchecked_sub_index(self, rhs: Self) -> Self::Scalar;
}

mod index_scalar_type_private {
    pub trait Sealed {}
}
pub unsafe trait IndexScalarType:
    index_scalar_type_private::Sealed + Sized + Clone + Copy + PartialEq + PartialOrd + Ord
{
    const ZERO: Self;
    const ONE: Self;

    fn try_from_usize(index: usize) -> Result<Self, IndexTooBigError>;
    unsafe fn from_usize_unchecked(index: usize) -> Self;
    fn to_usize(self) -> usize;

    fn checked_add_scalar(self, rhs: Self) -> Result<Self, IndexTooBigError>;
    unsafe fn unchecked_add_scalar(self, rhs: Self) -> Self;
}
