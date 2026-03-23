use crate::IndexTooBigError;

/// A generic error type used when an index is too big.
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
