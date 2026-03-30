//! Error types for the index_type crate.
//!
//! This module provides the [`GenericIndexTooBigError`] type, which can be used
//! as the error type for custom index types derived with `#[derive(IndexType)]`.

use crate::IndexTooBigError;

/// A generic error returned when an index exceeds the maximum representable value.
///
/// This error type can be used with `#[index_type(error = GenericIndexTooBigError)]`
/// when deriving `IndexType`:
///
/// ```
/// use index_type::{IndexType, GenericIndexTooBigError};
///
/// #[derive(IndexType, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
/// #[index_type(error = GenericIndexTooBigError)]
/// struct MyIndex(u32);
/// ```
///
/// The error displays as "index too big".
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GenericIndexTooBigError;

impl IndexTooBigError for GenericIndexTooBigError {
    fn new() -> Self {
        Self
    }
}

impl core::fmt::Display for GenericIndexTooBigError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "index too big")
    }
}

impl core::error::Error for GenericIndexTooBigError {}
