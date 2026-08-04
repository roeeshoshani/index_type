#![no_std]
//! A Rust library providing **strongly typed indices** for collections and everything else needed for working with them in an
//! ergonomic manner.
//!
//! ## What are typed indices?
//!
//! In standard Rust, collections use `usize` for indexing. This works well but provides no compile-time
//! protection against using an index from one collection with another. Typed indices solve this by
//! creating custom index types that are statically associated with specific collections.
//!
//! In standard Rust, a raw `usize` can index any collection. This allows subtle bugs:
//! ```rust
//! # #[cfg(feature = "alloc")] {
//! # #[derive(Default, Clone, Copy)]
//! # struct Node;
//! # #[derive(Default, Clone, Copy)]
//! # struct Edge;
//! let nodes: Vec<Node> = vec![Node::default(); 10];  // 10 nodes
//! let edges: Vec<Edge> = vec![Edge::default(); 5];   // 5 edges
//! let node_index = 3;
//! nodes[node_index];
//! edges[node_index]; // compiles just fine!
//! # }
//! ```
//!
//! With typed indices, cross-contamination becomes a compile error:
//! ```rust
//! # #[cfg(feature = "alloc")] {
//! # use index_type::{IndexType, vec::TypedVec, typed_vec};
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
//! # }
//! ```
//!
//! ## Features
//!
//! - **Type Safety**: Prevents accidental misuse of indices between different collections at compile time
//! - **`no_std` Support**: Works in embedded systems and other `no_std` environments
//! - **Memory Efficiency**: Use smaller integer types (`u8`, `u16`) for indices when collections are bounded
//! - **Niche Optimization**: Supports [`NonZero`](core::num::NonZero) types so `Option<Index>` has the same size as `Index`
//! - **Rich Collections**: Provides [`TypedSlice`](crate::slice::TypedSlice), [`TypedVec`](crate::vec::TypedVec), [`TypedArray`](crate::array::TypedArray), and [`TypedArrayVec`](crate::array_vec::TypedArrayVec)
//! - **Derive Macros**: Easy to define custom index types with `#[derive(IndexType)]`
//! - **Range Iterators**: Iterate over ranges using custom index types
//!
//! ## Quick Start
//!
//! ```rust
//! # #[cfg(feature = "alloc")] {
//! use index_type::{IndexType, vec::TypedVec};
//!
//! #[derive(IndexType, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
//! struct MyIndex(u32);
//!
//! let mut vec: TypedVec<MyIndex, i32> = TypedVec::new();
//! let idx = vec.push(42);
//!
//! assert_eq!(vec[idx], 42);
//! // vec[0usize]; // This won't compile - requires MyIndex type
//! # }
//! ```
//!
//! ## Defining Index Types
//!
//! Use the `#[derive(IndexType)]` macro on a newtype struct:
//! ```
//! use index_type::IndexType;
//!
//! #[derive(IndexType, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
//! struct MyIndex(u32);
//! ```
//!
//! The macro automatically implements the [`IndexType`] trait for your custom type. By default,
//! it generates an error type `MyIndexTooBigError`. You can specify a custom error type:
//! ```
//! # use index_type::IndexType;
//! # use index_type::IndexTooBigError;
//! #[derive(Debug, IndexTooBigError)]
//! #[index_too_big_error(msg = "item id too big")]
//! struct ItemIdTooBigError;
//!
//! #[derive(IndexType, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
//! #[index_type(error = ItemIdTooBigError)]
//! struct ItemId(u32);
//! ```
//!
//! ## Complex Indexing
//!
//! This crate also supports complex forms of indexing when using custom index types, for example, slicing a range with a custom
//! index type:
//! ```
//! # #[cfg(feature = "alloc")] {
//! # use index_type::IndexType;
//! # use index_type::typed_vec;
//! # use index_type::vec::TypedVec;
//! # use index_type::slice::TypedSlice;
//! #[derive(IndexType, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
//! struct ItemId(usize);
//!
//! #[derive(Debug, PartialEq, Eq)]
//! struct Item(u32);
//!
//! let values: TypedVec<ItemId, Item> = typed_vec![
//!     Item(45), Item(54), Item(32), Item(19), Item(78)
//! ];
//!
//! let some_values: &TypedSlice<ItemId, Item> = &values[ItemId(1)..ItemId(4)];
//! assert_eq!(some_values.as_slice(), &[Item(54), Item(32), Item(19)]);
//!
//! // Can even perform more complex types of slicing
//! let other_values: &TypedSlice<ItemId, Item> = &values[..ItemId(3)];
//! assert_eq!(other_values.as_slice(), &[Item(45), Item(54), Item(32)]);
//!
//! let other_values_2: &TypedSlice<ItemId, Item> = &values[ItemId(3)..];
//! assert_eq!(other_values_2.as_slice(), &[Item(19), Item(78)]);
//! # }
//! ```
//!
//! ## Typed Collections
//!
//! ### TypedVec
//!
//! A growable vector with typed indexing. See [`TypedVec`](crate::vec::TypedVec) for the full API.
//! ```
//! # #[cfg(feature = "alloc")] {
//! # use index_type::IndexType;
//! # use index_type::vec::TypedVec;
//! #[derive(IndexType, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
//! struct NodeId(u32);
//!
//! let mut nodes: TypedVec<NodeId, String> = TypedVec::new();
//! let id0 = nodes.push("Alice".to_string());
//! let id1 = nodes.push("Bob".to_string());
//!
//! println!("Node 0: {}", nodes[id0]);
//! # }
//! ```
//!
//! Operations that can fail due to index overflow have both panicking and fallible variants:
//! ```
//! # #[cfg(feature = "alloc")] {
//! # use index_type::IndexType;
//! # use index_type::vec::TypedVec;
//! # #[derive(IndexType, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
//! # struct MyIndex(u32);
//! let mut vec: TypedVec<MyIndex, i32> = TypedVec::new();
//!
//! // This will panic on index overflow (e.g. if the vector already contains (2^32 - 1) elements before calling `push`)
//! let idx = vec.push(1);
//!
//! // This will gracefully return an error in case of index overflow
//! let result: Result<MyIndex, MyIndexTooBigError> = vec.try_push(2);
//! # }
//! ```
//!
//! ### TypedSlice
//!
//! A slice wrapper with typed indexing.
//! `TypedSlice<I, T>` is the same as `[T]` but with index type `I`.
//! So, to represent `&[u8]` for example, use `&TypedSlice<I, u8>`, where `I` is your custom index type.
//! See [`TypedSlice`](crate::slice::TypedSlice) for the full API.
//! ```
//! # #[cfg(feature = "alloc")] {
//! # use index_type::IndexType;
//! # use index_type::vec::TypedVec;
//! # use index_type::typed_vec;
//! # use index_type::slice::TypedSlice;
//! #[derive(IndexType, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
//! struct RowId(u16);
//!
//! let vec: TypedVec<RowId, f64> = typed_vec![1.0, 2.0, 3.0];
//! let slice: &TypedSlice<RowId, f64> = vec.as_slice();
//!
//! // Safe indexing with custom type
//! let first = slice[RowId::ZERO];
//! # }
//! ```
//!
//! ### TypedArray
//!
//! A fixed-size array with typed indexing. The array length `N` is checked at compile time
//! to ensure it fits within the index type's range. See [`TypedArray`](crate::array::TypedArray) for the full API.
//! ```
//! # use index_type::IndexType;
//! # use index_type::array::TypedArray;
//! #[derive(IndexType, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
//! struct ValueIdx(u8);
//!
//! #[derive(Debug, PartialEq, Eq)]
//! struct Value(u32);
//!
//! // An index-typed version of `[Value; 3]`, with index type `ValueIdx`
//! let mut values: TypedArray<ValueIdx, Value, 3> = TypedArray::from_array([Value(3), Value(7), Value(5)]);
//! values[ValueIdx::ZERO] = Value(7);
//! values[ValueIdx(1)] = Value(32);
//! assert_eq!(values[ValueIdx(2)], Value(5));
//! ```
//!
//! ### TypedArrayVec
//!
//! A fixed-capacity vector backed by an array, similar to the `ArrayVec` type provided by the `arrayvec` crate but with typed indexing.
//! See [`TypedArrayVec`](crate::array_vec::TypedArrayVec) for the full API.
//! ```
//! # use index_type::IndexType;
//! # use index_type::array_vec::TypedArrayVec;
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
//! Using smaller integer types reduces memory when storing many indices. Useful when you know that the size of the collection is bounded.
//!
//! For example, if you are implementing a graph using an adjacency list, and you know that the graph will be reasonably small, you can use
//! 32-bit integers as indices instead of `usize`, which on 64-bit machines is half the size:
//! ```
//! # #[cfg(feature = "alloc")] {
//! # use index_type::{IndexType, vec::TypedVec};
//! // We know that the graph will never have more than 2^32 elements, so we can use `u32` as the index type.
//! #[derive(IndexType, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
//! struct NodeId(u32);
//!
//! struct Node {
//!     // Each node id is only 32-bits compared to `usize` which is 64-bit (assuming we are running on a 64-bit machine), which may
//!     // save a lot of space for large graphs.
//!     children: Vec<NodeId>,
//! }
//!
//! struct Graph {
//!     nodes: TypedVec<NodeId, Node>,
//!     root: NodeId,
//! }
//! # }
//! ```
//!
//! ## NonZero Indices and Niche Optimization
//!
//! Using [`NonZero`](core::num::NonZero) types enables niche optimization, where `Option<Index>`
//! has the same size as `Index`:
//! ```
//! # use index_type::IndexType;
//! # use core::num::NonZeroU32;
//! #[derive(IndexType, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
//! struct SafeId(NonZeroU32);
//!
//! // Option<SafeId> takes only 4 bytes, not 8!
//! assert_eq!(std::mem::size_of::<SafeId>(), 4);
//! assert_eq!(std::mem::size_of::<Option<SafeId>>(), 4);
//! ```
//!
//! And indexing into a collection with non-zero indices is of course as seamless as using any other integer type as the index type:
//! ```
//! # use index_type::IndexType;
//! # use index_type::typed_array;
//! # use index_type::array::TypedArray;
//! # use core::num::NonZeroU32;
//! #[derive(IndexType, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
//! struct MyId(NonZeroU32);
//!
//! let arr: TypedArray<MyId, i32, 4> = typed_array![7, 12, 19, 22];
//! assert_eq!(arr[MyId::from_raw_index(2)], 19);
//! ```
//!
//!
//! ## Range Iterators
//!
//! Currently, in stable rust, you cannot iterate over a range of values of a custom type:
//! ```compile_fail
//! struct MyIdx(u32);
//!
//! // There is nothing you can do to make this code work in stable rust
//! for i in MyIdx(0)..MyIdx(20) {}
//! ```
//!
//! The reason for this is that the built in range types only implement the [`Iterator`] trait if the value type `T` implements the
//! unstable [`Step`](core::iter::Step) trait, which you cannot implement for your own types in stable rust.
//!
//! Being able to iterate over ranges of index type is important for making the experience of working with typed indices feel seamless
//! and as smooth as using regular index types.
//!
//! This crate provides [`TypedRangeIterExt`](crate::range::TypedRangeIterExt) for iterating over ranges with custom index types:
//! ```
//! # use index_type::IndexType;
//! use index_type::range::TypedRangeIterExt;
//!
//! #[derive(IndexType, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
//! struct MyIdx(u32);
//!
//! for idx in (MyIdx(5)..MyIdx(10)).iter() {
//!     println!("{:?}", idx);
//! }
//! ```
//!
//! ## Typed Enumerate
//!
//! Use [`TypedIteratorExt`](crate::enumerate::TypedIteratorExt) to enumerate any iterator with typed indices:
//! ```
//! # use index_type::IndexType;
//! use index_type::enumerate::TypedIteratorExt;
//!
//! #[derive(IndexType, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
//! struct MyIdx(u32);
//!
//! let pairs: Vec<(MyIdx, &str)> = ["a", "b", "c"]
//!     .into_iter()
//!     .typed_enumerate::<MyIdx>()
//!     .collect();
//!
//! assert_eq!(pairs[1].0, MyIdx(1));
//! assert_eq!(pairs[1].1, "b");
//! ```
//!
//! ## Macros
//!
//! Convenience macros for creating typed collections:
//! ```
//! # #[cfg(feature = "alloc")] {
//! # use index_type::{typed_vec, typed_array, typed_array_vec, typed_slice, typed_slice_mut, IndexType};
//! # use index_type::vec::TypedVec;
//! # use index_type::array::TypedArray;
//! # use index_type::array_vec::TypedArrayVec;
//! # use index_type::slice::TypedSlice;
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
//! let av: TypedArrayVec<MyIndex, u8, 4> = typed_array_vec![1, 2];
//!
//! // Create a TypedSlice reference, similar to a slice literal (`&[1, 2, 3]`)
//! let s: &TypedSlice<MyIndex, i32> = typed_slice![1, 2, 3];
//! # }
//! ```
//!
//! ## Error Handling
//!
//! Operations that can fail due to index overflow return `Result` types.
//! Each index type has its own custom error type which is returned when operating on a collection which uses that index type.
//! ```
//! # #[cfg(feature = "alloc")] {
//! # use index_type::IndexType;
//! # use index_type::vec::TypedVec;
//! // Note: the `#[derive(IndexType)]` automatically generates a type called `MyIndexTooBigError` which is the custom error type
//! // for this custom index type.
//! #[derive(IndexType, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
//! struct MyIndex(u8);
//!
//! let mut vec: TypedVec<MyIndex, i32> = TypedVec::new();
//!
//! // Fill up to capacity
//! for i in 0..255 {
//!     vec.try_push(i).unwrap();
//! }
//!
//! // At this point, the length of the vec exceeds the index type, so this fails gracefully.
//! let res: Result<MyIndex, MyIndexTooBigError> = vec.try_push(255);
//! assert!(res.is_err());
//! # }
//! ```
//!
//!
//! ## no_std Compatibility
//!
//! This crate is `no_std` compatible. The `alloc` feature (enabled by default) enables
//! heap-allocated collections ([`TypedVec`](crate::vec::TypedVec) and related macros).
//!
//! For pure `no_std` environments without heap allocation, disable the `alloc` feature:
//! ```toml
//! [dependencies]
//! index_type = { version = "...", default-features = false }
//! ```
//!
//! ## `serde` Support
//!
//! This crate has a `serde` feature flag which implements `Serialize` and `Deserialize` for all of the relevant types exported by this
//! crate. This includes for example the main collection types (e.g. [`TypedVec`](crate::vec::TypedVec)).

pub use crate::error::GenericIndexTooBigError;

#[cfg(any(feature = "alloc", doc))]
#[doc(hidden)]
pub extern crate alloc;

pub mod array;
pub mod array_vec;
mod base_index_types;
pub mod enumerate;
mod error;
mod index_scalar_types;
#[doc(hidden)]
pub mod macros;
pub mod range;
pub mod slice;
mod utils;
#[cfg(any(feature = "alloc", doc))]
pub mod vec;

/// Derives the `IndexTooBigError` trait for an empty struct.
///
/// # Usage
///
/// ```rust
/// # use index_type::IndexTooBigError;
/// #[derive(IndexTooBigError, Debug)]
/// #[index_too_big_error(msg = "my custom error message")]
/// struct MyError;
/// ```
///
/// The `msg` attribute is required and specifies the display message for the error.
pub use index_type_macros::IndexTooBigError;

/// Derives the `IndexType` trait for a newtype struct around an existing `IndexType` (typically a primitive integer).
///
/// # Basic Usage
///
/// ```rust
/// # use index_type::IndexType;
/// #[derive(IndexType, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
/// struct MyIndex(u32);
/// ```
///
/// By default, this will also generate a `MyIndexTooBigError` struct that implements `IndexTooBigError`.
///
/// # Advanced Usage
///
/// You can specify a custom error type using the `#[index_type(error = ...)]` attribute:
///
/// ```rust
/// # use index_type::{IndexType, GenericIndexTooBigError};
/// #[derive(IndexType, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
/// #[index_type(error = GenericIndexTooBigError)]
/// struct MyIndex(u32);
/// ```
pub use index_type_macros::IndexType;

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
/// # use index_type::IndexType;
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
    /// This is `0` for regular unsigned integer types and `1` for [`NonZero`](core::num::NonZero) types.
    ///
    /// This can also be thought of as the minimum raw index biased value representable by this type.
    const BIAS: usize;

    /// Attempts to create an index from a raw `usize` value.
    ///
    /// Returns an error if the value exceeds `MAX_RAW_INDEX`.
    fn try_from_raw_index(index: usize) -> Result<Self, Self::IndexTooBigError>;

    /// Creates an index from a raw `usize` value.
    ///
    /// # Panics
    ///
    /// Panics if the value exceeds `MAX_RAW_INDEX`.
    fn from_raw_index(index: usize) -> Self {
        Self::try_from_raw_index(index).unwrap()
    }

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

    /// Creates an index from a scalar value.
    ///
    /// # Panics
    ///
    /// Panics if the value cannot be represented.
    fn from_scalar(scalar: Self::Scalar) -> Self {
        Self::try_from_scalar(scalar).unwrap()
    }

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

    /// Converts a `usize` to this scalar type.
    ///
    /// # Panics
    ///
    /// Panics if the value exceeds the maximum representable value.
    fn from_usize(value: usize) -> Self {
        Self::try_from_usize(value).unwrap()
    }

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
/// # use index_type::IndexType;
/// # use index_type::GenericIndexTooBigError;
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
