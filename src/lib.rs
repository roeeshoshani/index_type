#![no_std]
//! A Rust library providing **strongly typed indices** for collections, designed for both `std` and `no_std` environments.
//!
//! ## What are typed indices?
//!
//! In standard Rust, collections use `usize` for indexing. This works well but provides no compile-time
//! protection against using an index from one collection with another. Typed indices solve this by
//! creating custom index types that are statically associated with specific collections.
//!
//! In standard Rust, a raw `usize` can index any collection. This allows subtle bugs:
//! ```rust
//! # #[derive(Default, Clone, Copy)]
//! # struct Node;
//! # #[derive(Default, Clone, Copy)]
//! # struct Edge;
//! let nodes: Vec<Node> = vec![Node::default(); 10];  // 10 nodes
//! let edges: Vec<Edge> = vec![Edge::default(); 5];   // 5 edges
//! let node_index = 3;
//! nodes[node_index];
//! edges[node_index]; // compiles just fine!
//! ```
//!
//! With typed indices, cross-contamination becomes a compile error:
//! ```rust
//! # use index_type::{IndexType, typed_vec::TypedVec, typed_vec};
//! # #[derive(Default, Clone, Copy)]
//! # struct Node;
//! # #[derive(Default, Clone, Copy)]
//! # struct Edge;
//! #[derive(IndexType, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
//! struct NodeId(u32);
//!
//! #[derive(IndexType, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
//! struct EdgeId(u32);
//!
//! let nodes: TypedVec<NodeId, Node> = typed_vec![Node::default(); 10];
//! let edges: TypedVec<EdgeId, Edge> = typed_vec![Edge::default(); 10];
//! let node_id = NodeId(3);
//! nodes[node_id]; // OK
//! // edges[node_id]; // COMPILE ERROR: expected EdgeId, found NodeId
//! ```
//!
//! ## Features
//!
//! - **Type Safety**: Prevents accidental misuse of indices between different collections at compile time
//! - **`no_std` Support**: Works in embedded systems and other `no_std` environments
//! - **Memory Efficiency**: Use smaller integer types (`u8`, `u16`) for indices when collections are bounded
//! - **Niche Optimization**: Supports [`NonZero`](core::num::NonZero) types so `Option<Index>` has the same size as `Index`
//! - **Rich Collections**: Provides [`TypedSlice`](crate::typed_slice::TypedSlice), [`TypedVec`](crate::typed_vec::TypedVec), [`TypedArray`](crate::typed_array::TypedArray), and [`TypedArrayVec`](crate::typed_array_vec::TypedArrayVec)
//! - **Derive Macros**: Easy to define custom index types with `#[derive(IndexType)]`
//! - **Range Iterators**: Iterate over ranges using custom index types
//!
//! ## Quick Start
//!
//! ```rust
//! use index_type::IndexType;
//! use index_type::typed_vec::TypedVec;
//!
//! #[derive(IndexType, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
//! struct MyIndex(u32);
//!
//! let mut vec: TypedVec<MyIndex, i32> = TypedVec::new();
//! let idx = vec.push(42);
//!
//! assert_eq!(vec[idx], 42);
//! // vec[0usize]; // This won't compile - requires MyIndex type
//! ```
//!
//! ## Defining Index Types
//!
//! Use the `#[derive(IndexType)]` macro on a newtype struct:
//!
//! ```
//! use index_type::IndexType;
//!
//! #[derive(IndexType, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
//! struct MyIndex(u32);
//! ```
//!
//! The macro automatically implements the [`IndexType`] trait for your custom type. By default,
//! it generates an error type `MyIndexTooBigError`. You can specify a custom error type:
//!
//! ```
//! use index_type::IndexType;
//! use index_type::IndexTooBigError;
//!
//! #[derive(Debug, IndexTooBigError)]
//! #[index_too_big_error(msg = "item id too big")]
//! struct ItemIdTooBigError;
//!
//! #[derive(IndexType, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
//! #[index_type(error = ItemIdTooBigError)]
//! struct ItemId(u32);
//! ```
//!
//! ## Typed Collections
//!
//! ### TypedVec
//!
//! A growable vector with typed indexing. See [`TypedVec`](crate::typed_vec::TypedVec) for the full API.
//!
//! ```
//! use index_type::IndexType;
//! use index_type::typed_vec::TypedVec;
//!
//! #[derive(IndexType, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
//! struct NodeId(u32);
//!
//! let mut nodes: TypedVec<NodeId, String> = TypedVec::new();
//! let id0 = nodes.push("Alice".to_string());
//! let id1 = nodes.push("Bob".to_string());
//!
//! println!("Node 0: {}", nodes[id0]);
//! ```
//!
//! Operations that can fail due to index overflow have both panicking and fallible variants:
//!
//! ```
//! # use index_type::IndexType;
//! # use index_type::typed_vec::TypedVec;
//! # #[derive(IndexType, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
//! # struct MyIndex(u32);
//! let mut vec: TypedVec<MyIndex, i32> = TypedVec::new();
//! vec.push(1);                              // Panics if index too big
//! let result = vec.try_push(2);             // Returns Result<(), Error>
//! ```
//!
//! ### TypedSlice
//!
//! A slice wrapper with typed indexing. See [`TypedSlice`](crate::typed_slice::TypedSlice) for the full API.
//!
//! ```
//! use index_type::IndexType;
//! use index_type::typed_vec::TypedVec;
//! use index_type::typed_slice::TypedSlice;
//!
//! #[derive(IndexType, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
//! struct RowId(u16);
//!
//! let vec: TypedVec<RowId, f64> = TypedVec::from_vec(vec![1.0, 2.0, 3.0]);
//! let slice: &TypedSlice<RowId, f64> = vec.as_slice();
//!
//! // Safe indexing with custom type
//! let first = slice[RowId::ZERO];
//! ```
//!
//! ### TypedArray
//!
//! A fixed-size array with typed indexing. The array length `N` is checked at compile time
//! to ensure it fits within the index type's range. See [`TypedArray`](crate::typed_array::TypedArray) for the full API.
//!
//! ```
//! use index_type::IndexType;
//! use index_type::typed_array::TypedArray;
//!
//! #[derive(IndexType, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
//! struct PixelIdx(u8);
//!
//! let mut pixels: TypedArray<PixelIdx, [u8; 3], 4> = TypedArray::default();
//! pixels[PixelIdx::ZERO] = [255, 0, 0];  // Red
//! pixels[PixelIdx(1)] = [0, 255, 0];     // Green
//! ```
//!
//! ### TypedArrayVec
//!
//! A fixed-capacity vector ideal for embedded systems. It never allocates after creation.
//! See [`TypedArrayVec`](crate::typed_array_vec::TypedArrayVec) for the full API.
//!
//! ```
//! use index_type::IndexType;
//! use index_type::typed_array_vec::TypedArrayVec;
//!
//! #[derive(IndexType, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
//! struct BufferIndex(u8);
//!
//! let mut buffer: TypedArrayVec<BufferIndex, u8, 16> = TypedArrayVec::new();
//! buffer.push(42);
//! assert_eq!(buffer.len().to_raw_index(), 1);
//! ```
//!
//! A `TypedArrayVec<u8, u8, 3>` is only 4 bytes (3 bytes for data + 1 byte for length).
//!
//! ## Memory-Efficient Indices
//!
//! Using smaller integer types reduces memory when storing many indices:
//!
//! ```
//! # use index_type::IndexType;
//! #[derive(IndexType, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
//! struct SmallIndex(u8);  // Only 1 byte per index!
//!
//! println!("SmallIndex: {} bytes", std::mem::size_of::<SmallIndex>());
//! println!("u32 index: {} bytes", std::mem::size_of::<u32>());
//! ```
//!
//! For collections with at most 255 elements, `u8` saves 75% memory compared to `u32`.
//!
//! ## NonZero Indices and Niche Optimization
//!
//! Using [`NonZero`](core::num::NonZero) types enables niche optimization, where `Option<Index>`
//! has the same size as `Index`:
//!
//! ```
//! use index_type::IndexType;
//! use core::num::NonZeroU32;
//!
//! #[derive(IndexType, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
//! struct SafeId(NonZeroU32);
//!
//! // Option<SafeId> takes only 4 bytes, not 8!
//! assert_eq!(std::mem::size_of::<SafeId>(), 4);
//! assert_eq!(std::mem::size_of::<Option<SafeId>>(), 4);
//! assert_eq!(SafeId::BIAS, 1);
//! assert_eq!(SafeId::ZERO.to_raw_index(), 0);
//! assert_eq!(SafeId::ZERO.to_raw_index_biased(), 1);
//! ```
//!
//! ## Range Iterators
//!
//! Standard Rust ranges require the unstable [`Step`](core::iter::Step) trait. This crate provides
//! [`TypedRangeIterExt`](crate::typed_range::TypedRangeIterExt) for iterating over ranges with custom index types:
//!
//! ```
//! use index_type::IndexType;
//! use index_type::typed_range::TypedRangeIterExt;
//!
//! #[derive(IndexType, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
//! struct MyIdx(u32);
//!
//! let start = MyIdx(5);
//! let end = MyIdx(10);
//!
//! for idx in (start..end).iter() {
//!     println!("{:?}", idx);
//! }
//! ```
//!
//! ## Typed Enumerate
//!
//! Use [`TypedIteratorExt`](crate::typed_enumerate::TypedIteratorExt) to enumerate any iterator with typed indices:
//!
//! ```
//! use index_type::IndexType;
//! use index_type::typed_enumerate::TypedIteratorExt;
//!
//! #[derive(IndexType, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
//! struct RowIdx(u32);
//!
//! let pairs: Vec<_> = ["a", "b", "c"]
//!     .into_iter()
//!     .typed_enumerate::<RowIdx>()
//!     .collect();
//!
//! assert_eq!(pairs[1].0, RowIdx(1));
//! assert_eq!(pairs[1].1, "b");
//! ```
//!
//! ## Macros
//!
//! Convenience macros for creating typed collections:
//!
//! ```
//! use index_type::{typed_vec, typed_array, typed_array_vec, typed_slice, typed_slice_mut, IndexType};
//! use index_type::typed_vec::TypedVec;
//! use index_type::typed_array::TypedArray;
//! use index_type::typed_array_vec::TypedArrayVec;
//! use index_type::typed_slice::TypedSlice;
//!
//! #[derive(IndexType, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
//! struct MyIndex(u32);
//!
//! // Create a TypedVec
//! let v: TypedVec<MyIndex, i32> = typed_vec![1, 2, 3];
//!
//! // Create a TypedArray
//! let a: TypedArray<MyIndex, i32, 3> = typed_array![1, 2, 3];
//!
//! // Create a TypedArrayVec
//! let av: TypedArrayVec<MyIndex, u8, 4> = typed_array_vec![1, 2, 3, 4];
//!
//! // Create a TypedSlice reference
//! let s: &TypedSlice<MyIndex, i32> = typed_slice![1, 2, 3];
//! ```
//!
//! ## Error Handling
//!
//! Operations that can fail due to index overflow return `Result` types:
//!
//! ```
//! use index_type::IndexType;
//! use index_type::typed_vec::TypedVec;
//!
//! #[derive(IndexType, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
//! struct MyIndex(u8);  // MAX_RAW_INDEX = 255
//!
//! let mut vec: TypedVec<MyIndex, i32> = TypedVec::new();
//!
//! // Fill up to capacity
//! for i in 0..255 {
//!     vec.try_push(i).unwrap();
//! }
//!
//! // This fails gracefully
//! assert!(vec.try_push(255).is_err());
//! ```
//!
//! ## no_std Compatibility
//!
//! This crate is `no_std` compatible. The `alloc` feature (enabled by default) enables
//! heap-allocated collections ([`TypedVec`](crate::typed_vec::TypedVec) and related macros).
//!
//! For pure `no_std` environments without heap allocation, disable the `alloc` feature:
//!
//! ```toml
//! [dependencies]
//! index_type = { version = "...", default-features = false }
//! ```
//!

pub use crate::error::GenericIndexTooBigError;

#[cfg(feature = "alloc")]
#[doc(hidden)]
pub extern crate alloc;

mod base_index_types;
mod error;
mod index_scalar_types;
#[doc(hidden)]
pub mod macros;
pub mod typed_array;
pub mod typed_array_vec;
pub mod typed_enumerate;
pub mod typed_range;
pub mod typed_slice;
#[cfg(feature = "alloc")]
pub mod typed_vec;
mod utils;

pub use index_type_macros::{IndexTooBigError, IndexType};

/// A trait for types that can be used as indices into typed collections.
///
/// This trait is the foundation of the crate. It is implemented for primitive unsigned integer
/// types (`u8`, `u16`, `u32`, `u64`, `usize`) and their [`NonZero`](core::num::NonZero) variants. Custom index types
/// should be defined using the `#[derive(IndexType)]` macro, which implements this trait for a
/// newtype struct.
///
/// # Safety
///
/// Do not implement directly; use `#[derive(IndexType)]` instead.
///
/// # Index vs Raw Index
///
/// The distinction between "index" and "raw index" is important for [`NonZero`](core::num::NonZero) types.
/// For a regular type like `u8`:
/// - Raw index 0 maps to `u8::ZERO` (0)
/// - Raw index 255 maps to `u8::MAX` (255)
/// - `BIAS` is `0`, so `to_raw_index()` and `to_raw_index_biased()` return the same value
///
/// For a [`NonZero`](core::num::NonZero) type like `NonZeroU8`:
/// - Raw index 0 maps to `NonZeroU8::new_unchecked(1)` (the minimum valid value)
/// - Raw index 254 maps to `NonZeroU8::new_unchecked(255)` (the maximum valid value)
/// - Raw index 255 is **invalid** because it would overflow when adding 1 to get the inner value
/// - `BIAS` is `1`, so `to_raw_index_biased()` exposes the actual stored integer value
///
/// This design allows `Option<NonZeroU8>` to occupy a single byte (niche optimization).
///
/// # Example
///
/// ```
/// use index_type::IndexType;
///
/// #[derive(IndexType, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
/// struct MyIndex(u32);
///
/// let idx = MyIndex::ZERO;
/// let next = MyIndex::try_from_raw_index(5).unwrap();
/// assert_eq!(next.to_raw_index(), 5);
/// assert_eq!(MyIndex::BIAS, 0);
/// assert_eq!(next.to_raw_index_biased(), 5);
/// ```
pub unsafe trait IndexType:
    Sized + Clone + Copy + PartialEq + Eq + PartialOrd + Ord
{
    /// The error type returned when an index exceeds the maximum representable value.
    type IndexTooBigError: IndexTooBigError;

    /// The scalar type used for arithmetic operations with this index type.
    ///
    /// For `u32`, this is `u32`. For `NonZeroU32`, this is `u32`.
    type Scalar: IndexScalarType;

    /// The zero index value.
    ///
    /// This is typically `0` for regular integers, or `1` for [`NonZero`](core::num::NonZero) types.
    const ZERO: Self;

    /// The maximum index value representable by this type.
    ///
    /// For `u8`, this is `255`. For [`NonZeroU8`](core::num::NonZeroU8), this is `NonZeroU8::new(255)`.
    const MAX_INDEX: Self;

    /// The maximum raw index value representable by this type.
    ///
    /// For `u8`, this is `255`. For [`NonZeroU8`](core::num::NonZeroU8), this is `254`
    /// (since raw index 255 would map to a value outside the valid range).
    const MAX_RAW_INDEX: usize;

    /// The offset between the logical raw index and the underlying integer representation.
    ///
    /// This is `0` for regular unsigned integer types and `1` for
    /// [`NonZero`](core::num::NonZero) types.
    const BIAS: usize;

    /// Attempts to create an index from a raw `usize` value.
    ///
    /// Returns an error if the value exceeds `MAX_RAW_INDEX`.
    fn try_from_raw_index(index: usize) -> Result<Self, Self::IndexTooBigError>;

    /// Creates an index from a raw `usize` value without bounds checking.
    ///
    /// # Safety
    ///
    /// The index must be less than or equal to `MAX_RAW_INDEX`.
    unsafe fn from_raw_index_unchecked(index: usize) -> Self;

    /// Converts the index to a raw `usize` value.
    fn to_raw_index(self) -> usize;

    /// Converts the index to its raw `usize` value in the underlying integer representation.
    ///
    /// This is equal to `self.to_raw_index() + Self::BIAS`, but is computed in a much more efficient way.
    ///
    /// For regular integer index types, this is identical to [`Self::to_raw_index`].
    /// For [`NonZero`](core::num::NonZero) index types, this returns the actual stored integer
    /// value, which is one greater than the logical raw index.
    ///
    /// This operation is very cheap, it is basically a no-op, as it returns the underlying integer value in its existing
    /// representation and does not require any conversion.
    fn to_raw_index_biased(self) -> usize;

    /// Attempts to create an index from a scalar value.
    ///
    /// Returns an error if the value cannot be represented.
    fn try_from_scalar(scalar: Self::Scalar) -> Result<Self, Self::IndexTooBigError>;

    /// Creates an index from a scalar value without bounds checking.
    ///
    /// # Safety
    ///
    /// The scalar must be representable by this index type.
    unsafe fn from_scalar_unchecked(scalar: Self::Scalar) -> Self;

    /// Converts the index to its scalar representation.
    fn to_scalar(self) -> Self::Scalar;

    /// Performs checked addition with a scalar value.
    ///
    /// Returns an error if the result would exceed `MAX_RAW_INDEX`.
    fn checked_add_scalar(self, rhs: Self::Scalar) -> Result<Self, Self::IndexTooBigError>;

    /// Performs checked subtraction with a scalar value.
    ///
    /// Returns `None` if the result would underflow.
    fn checked_sub_scalar(self, rhs: Self::Scalar) -> Option<Self>;

    /// Performs checked multiplication with a scalar value.
    ///
    /// Returns an error if the result would exceed `MAX_RAW_INDEX`.
    fn checked_mul_scalar(self, rhs: Self::Scalar) -> Result<Self, Self::IndexTooBigError>;

    /// Performs checked subtraction of another index, returning a scalar.
    ///
    /// Returns `None` if the result would underflow.
    fn checked_sub_index(self, rhs: Self) -> Option<Self::Scalar>;

    /// Performs unchecked addition with a scalar value.
    ///
    /// # Safety
    ///
    /// The result must not exceed `MAX_RAW_INDEX`.
    unsafe fn unchecked_add_scalar(self, rhs: Self::Scalar) -> Self;

    /// Performs unchecked subtraction with a scalar value.
    ///
    /// # Safety
    ///
    /// The result must not underflow.
    unsafe fn unchecked_sub_scalar(self, rhs: Self::Scalar) -> Self;

    /// Performs unchecked subtraction of another index, returning a scalar.
    ///
    /// # Safety
    ///
    /// The result must be non-negative and representable by the scalar type.
    unsafe fn unchecked_sub_index(self, rhs: Self) -> Self::Scalar;
}

mod index_scalar_type_private {
    pub trait Sealed {}
}

/// A trait for scalar types used with [`IndexType`].
///
/// This trait is implemented for unsigned integer types (`u8`, `u16`, `u32`, `u64`, `usize`)
/// and provides the arithmetic operations needed for index manipulation.
///
/// # Safety
///
/// Implementations must be for unsigned integer types whose size is less than or equal to `usize`.
pub unsafe trait IndexScalarType:
    index_scalar_type_private::Sealed + Sized + Clone + Copy + PartialEq + PartialOrd + Ord
{
    /// The zero value of this scalar type.
    const ZERO: Self;

    /// The one value of this scalar type.
    const ONE: Self;

    /// Attempts to convert a `usize` to this scalar type.
    ///
    /// Returns `None` if the value exceeds the maximum representable value.
    fn try_from_usize(value: usize) -> Option<Self>;

    /// Converts a `usize` to this scalar type without bounds checking.
    ///
    /// # Safety
    ///
    /// The value must be representable by this scalar type.
    unsafe fn from_usize_unchecked(value: usize) -> Self;

    /// Converts this scalar type to a `usize`.
    fn to_usize(self) -> usize;

    /// Performs checked addition with another scalar.
    ///
    /// Returns `None` if overflow would occur.
    fn checked_add_scalar(self, rhs: Self) -> Option<Self>;

    /// Performs checked subtraction with another scalar.
    ///
    /// Returns `None` if underflow would occur.
    fn checked_sub_scalar(self, rhs: Self) -> Option<Self>;

    /// Performs unchecked addition with another scalar.
    ///
    /// # Safety
    ///
    /// The result must not overflow the scalar type.
    unsafe fn unchecked_add_scalar(self, rhs: Self) -> Self;

    /// Performs unchecked subtraction with another scalar.
    ///
    /// # Safety
    ///
    /// The result must not underflow the scalar type.
    unsafe fn unchecked_sub_scalar(self, rhs: Self) -> Self;
}

/// A trait for errors indicating that an index value is too large.
///
/// This trait is implemented by error types returned when index operations would
/// exceed the maximum representable value for an [`IndexType`].
///
/// # Example
///
/// ```
/// use index_type::IndexType;
/// use index_type::GenericIndexTooBigError;
///
/// #[derive(IndexType, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
/// #[index_type(error = GenericIndexTooBigError)]
/// struct MyIndex(u32);
///
/// let result = MyIndex::try_from_raw_index(u32::MAX as usize + 1);
/// assert!(result.is_err());
/// ```
pub trait IndexTooBigError: core::error::Error {
    /// Creates a new instance of the error.
    fn new() -> Self;
}
