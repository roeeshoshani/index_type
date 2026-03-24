#![no_std]
//! A Rust crate providing strongly typed indices for collections (e.g, slice, vec), designed for `no_std` environments.
//!
//! # Overview
//!
//! This crate allows you to define custom index types for your collections, providing better type safety and preventing accidental use of indices from one collection with another. It also supports using smaller integer types for indices to save memory when you know your collection won't exceed a certain size.
//!
//! # Features
//!
//! - **Typed Indices**: Define custom types for indices using the [`IndexType`] derive macro.
//! - **`no_std` Support**: Designed to work in embedded or other `no_std` environments.
//! - **Memory Efficiency**: Use smaller integer types (e.g., `u8`, `u16`) as indices for memory-constrained applications.
//! - **Rich Collection Support**: Provides [`TypedSlice`](crate::typed_slice::TypedSlice), [`TypedVec`](crate::typed_vec::TypedVec), and [`TypedArray`](crate::typed_array::TypedArray) which are thin wrappers around the standard library's slice, `Vec`, and array types.
//!
//! ## Basic Example
//!
//! ```rust
//! use index_type::IndexType;
//! use index_type::typed_vec::TypedVec;
//!
//! #[derive(IndexType, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
//! struct MyIndex(u32);
//!
//! # fn main() {
//! let mut vec: TypedVec<MyIndex, i32> = TypedVec::new();
//! let idx = vec.push(42);
//!
//! assert_eq!(vec[idx], 42);
//! // vec[0usize]; // This will not compile as it requires MyIndex
//! # }
//! ```
//!
//! ## Memory-Efficient Indices
//!
//! ```rust
//! # use index_type::IndexType;
//! #[derive(IndexType, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
//! struct SmallIndex(u8);
//!
//! // TypedVec<SmallIndex, T> can only hold up to 255 elements.
//! // This is useful for saving memory in large data structures containing many indices.
//! ```

pub use crate::error::GenericIndexTooBigError;

#[doc(hidden)]
pub extern crate alloc;

mod base_index_types;
mod error;
mod index_scalar_types;
#[doc(hidden)]
pub mod macros;
pub mod typed_array;
pub mod typed_array_vec;
pub mod typed_range_iter;
pub mod typed_slice;
pub mod typed_vec;
mod utils;

pub use index_type_macros::{IndexTooBigError, IndexType};

/// A trait for types that can be used as indices to some collection of items.
///
/// # Safety
///
/// Do not implement directly, use `#[derive(IndexType)]` instead (see [`IndexType`]).
///
/// Incorrect implementations may lead to undefined behaviour when using the index type.
///
/// [`IndexType`]: index_type_macros::IndexType
pub unsafe trait IndexType:
    Sized + Clone + Copy + PartialEq + Eq + PartialOrd + Ord
{
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

    /// Checked subtraction with a scalar.
    fn checked_sub_scalar(self, rhs: Self::Scalar) -> Option<Self>;

    /// Checked multiplication with a scalar.
    fn checked_mul_scalar(self, rhs: Self::Scalar) -> Result<Self, Self::IndexTooBigError>;

    /// Unchecked addition with a scalar.
    ///
    /// # Safety
    ///
    /// The result must be representable by this index type.
    unsafe fn unchecked_add_scalar(self, rhs: Self::Scalar) -> Self;

    /// Unchecked subtraction with a scalar.
    ///
    /// # Safety
    ///
    /// The result must be representable by this index type.
    unsafe fn unchecked_sub_scalar(self, rhs: Self::Scalar) -> Self;

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
/// Must only be implemented for standard integer types whose size is less than or equal to the size of `usize`.
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

    /// Checked subtraction.
    fn checked_sub_scalar(self, rhs: Self) -> Option<Self>;

    /// Unchecked addition.
    ///
    /// # Safety
    ///
    /// The result must be representable by this scalar type.
    unsafe fn unchecked_add_scalar(self, rhs: Self) -> Self;

    /// Unchecked subtraction.
    ///
    /// # Safety
    ///
    /// The result must be representable by this scalar type.
    unsafe fn unchecked_sub_scalar(self, rhs: Self) -> Self;
}

/// A trait for errors indicating that an index is too big.
pub trait IndexTooBigError: core::error::Error {
    /// Creates a new instance of the error.
    fn new() -> Self;
}
