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

/// An error type used when a collection is at full capacity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CapacityError<T> {
    element: T,
}

impl<T> CapacityError<T> {
    /// Creates a new `CapacityError` with the element that could not be added.
    pub const fn new(element: T) -> Self {
        Self { element }
    }

    /// Returns the element that could not be added.
    pub fn into_inner(self) -> T {
        self.element
    }
}

impl<T> core::fmt::Display for CapacityError<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "insufficient capacity")
    }
}

impl<T: core::fmt::Debug> core::error::Error for CapacityError<T> {}
