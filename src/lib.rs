#![no_std]

//! A crate for strongly typed indices.
//!
//! This crate provides tools to use types other than `usize` as indices for slices and vectors,
//! providing more type safety and potentially smaller memory footprints.

use thiserror_no_std::Error;

extern crate alloc;

mod base_index_types;
pub mod typed_slice;
pub mod typed_vec;

/// Error returned when a `usize` index is too large to be represented by an [`IndexType`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Error)]
#[error("index too big")]
pub struct IndexTooBigError;

/// A trait for types that can be used as indices.
///
/// # Safety
///
/// Implementations must ensure that:
/// - `to_index` returns a value that, when passed back to `from_index_unchecked`, produces an equivalent index.
/// - `ZERO` is a valid representation of the index 0.
/// - `unchecked_add_usize` and `unchecked_sub` are correct and don't cause UB if their preconditions are met.
pub unsafe trait IndexType:
    Sized + Clone + Copy + PartialEq + Eq + PartialOrd + Ord
{
    /// The zero value for this index type.
    const ZERO: Self;

    /// Tries to create an index from a `usize`.
    ///
    /// Returns [`IndexTooBigError`] if the index is out of range for this type.
    fn try_from_index(index: usize) -> Result<Self, IndexTooBigError>;

    /// Creates an index from a `usize` without checking if it fits.
    ///
    /// # Safety
    ///
    /// `index` must be representable by this type.
    unsafe fn from_index_unchecked(index: usize) -> Self;

    /// Converts the index to a `usize`.
    fn to_index(self) -> usize;

    /// Adds a `usize` to this index without checking for overflow.
    ///
    /// # Safety
    ///
    /// The result must be representable by this type.
    unsafe fn unchecked_add_usize(self, rhs: usize) -> Self;

    /// Subtracts another index from this one without checking for underflow.
    ///
    /// # Safety
    ///
    /// `rhs` must be less than or equal to `self`.
    unsafe fn unchecked_sub(self, rhs: Self) -> usize;
}
