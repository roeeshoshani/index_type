#![no_std]

pub use crate::error::GenericIndexTooBigError;

extern crate alloc;

mod base_index_types;
mod error;
mod index_scalar_types;
pub mod typed_array;
pub mod typed_slice;
pub mod typed_vec;
mod utils;

pub use index_type_macros::{IndexTooBigError, IndexType};

/// A trait for types that can be used as indices for `TypedSlice`, `TypedVec`, and `TypedArray`.
///
/// # Safety
///
/// The implementation must ensure that `to_raw_index` returns a value that is less than or equal to `MAX_RAW_INDEX`.
/// `from_raw_index_unchecked` must be safe to call with any value less than or equal to `MAX_RAW_INDEX`.
pub unsafe trait IndexType: Sized + Clone + Copy + PartialEq + Eq + PartialOrd + Ord {
    /// The error type returned when an index is too big.
    type IndexTooBigError: IndexTooBigError;

    /// The scalar type associated with this index type.
    type Scalar: IndexScalarType;

    /// The zero index.
    const ZERO: Self;

    /// The maximum raw index representable by this type.
    const MAX_RAW_INDEX: usize;

    /// Tries to create an index from a raw `usize` index.
    fn try_from_raw_index(index: usize) -> Result<Self, Self::IndexTooBigError>;

    /// Creates an index from a raw `usize` index without checking if it's in bounds.
    ///
    /// # Safety
    ///
    /// The index must be less than or equal to `MAX_RAW_INDEX`.
    unsafe fn from_raw_index_unchecked(index: usize) -> Self;

    /// Converts the index to a raw `usize` index.
    fn to_raw_index(self) -> usize;

    /// Tries to create an index from a scalar value.
    fn try_from_scalar(scalar: Self::Scalar) -> Result<Self, Self::IndexTooBigError>;

    /// Creates an index from a scalar value without checking if it's in bounds.
    ///
    /// # Safety
    ///
    /// The scalar must be representable by this index type.
    unsafe fn from_scalar_unchecked(scalar: Self::Scalar) -> Self;

    /// Converts the index to a scalar value.
    fn to_scalar(self) -> Self::Scalar;

    /// Checked addition with a scalar.
    fn checked_add_scalar(self, rhs: Self::Scalar) -> Result<Self, Self::IndexTooBigError>;

    /// Checked multiplication with a scalar.
    fn checked_mul_scalar(self, rhs: Self::Scalar) -> Result<Self, Self::IndexTooBigError>;

    /// Unchecked addition with a scalar.
    ///
    /// # Safety
    ///
    /// The result must be representable by this index type.
    unsafe fn unchecked_add_scalar(self, rhs: Self::Scalar) -> Self;

    /// Unchecked subtraction of an index, returning a scalar.
    ///
    /// # Safety
    ///
    /// The result must be non-negative and representable by the scalar type.
    unsafe fn unchecked_sub_index(self, rhs: Self) -> Self::Scalar;
}

mod index_scalar_type_private {
    pub trait Sealed {}
}
/// A trait for scalar types that can be used with `IndexType`.
///
/// # Safety
///
/// The implementation must ensure that `to_usize` returns a value that is consistent with `from_usize_unchecked`.
pub unsafe trait IndexScalarType:
    index_scalar_type_private::Sealed + Sized + Clone + Copy + PartialEq + PartialOrd + Ord
{
    /// The zero value.
    const ZERO: Self;
    /// The one value.
    const ONE: Self;

    /// Tries to create a scalar from a `usize`.
    fn try_from_usize(value: usize) -> Option<Self>;

    /// Creates a scalar from a `usize` without checking if it's in bounds.
    ///
    /// # Safety
    ///
    /// The value must be representable by this scalar type.
    unsafe fn from_usize_unchecked(value: usize) -> Self;

    /// Converts the scalar to a `usize`.
    fn to_usize(self) -> usize;

    /// Checked addition.
    fn checked_add_scalar(self, rhs: Self) -> Option<Self>;

    /// Unchecked addition.
    ///
    /// # Safety
    ///
    /// The result must be representable by this scalar type.
    unsafe fn unchecked_add_scalar(self, rhs: Self) -> Self;
}

/// A trait for errors indicating that an index is too big.
pub trait IndexTooBigError: core::error::Error {
    /// Creates a new instance of the error.
    fn new() -> Self;
}
