# index_type

A Rust crate providing strongly typed indices for collections (e.g, slice, vec), designed for `no_std` environments.

## Overview

This crate allows you to define custom index types for your collections, providing better type safety and preventing accidental use of indices from one collection with another. It also supports using smaller integer types for indices to save memory when you know your collection won't exceed a certain size.

## Features

- **Typed Indices**: Define custom types for indices using the [`IndexType`] derive macro.
- **`no_std` Support**: Designed to work in embedded or other `no_std` environments.
- **Memory Efficiency**: Use smaller integer types (e.g., `u8`, `u16`) as indices for memory-constrained applications.
- **Rich Collection Support**: Provides [`TypedSlice`](crate::typed_slice::TypedSlice), [`TypedVec`](crate::typed_vec::TypedVec), and [`TypedArray`](crate::typed_array::TypedArray) which are thin wrappers around the standard library's slice, `Vec`, and array types.

### Basic Example

```rust
use index_type::IndexType;
use index_type::typed_vec::TypedVec;

#[derive(IndexType, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct MyIndex(u32);

let mut vec: TypedVec<MyIndex, i32> = TypedVec::new();
let idx = vec.push(42).unwrap();

assert_eq!(vec[idx], 42);
// vec[0usize]; // This will not compile as it requires MyIndex
```

### Memory-Efficient Indices

Using smaller integer types for indices can significantly reduce memory usage, especially when storing many indices in a collection.

For example, if you have a `Vec<MyIndex>` where `MyIndex` is a newtype over `u32`, each index takes 4 bytes. If you know your collection will never have more than 256 elements, you can use `u8` instead, reducing the size of each index to 1 byte.

```rust
#[derive(IndexType, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct SmallIndex(u8);

// TypedVec<SmallIndex, T> can only hold up to 255 elements.
// This is useful for saving memory in large data structures containing many indices.
let mut indices: Vec<SmallIndex> = Vec::new(); // Each element is only 1 byte
```

### NonZero Indices and Niche Optimization

This crate also supports `NonZero` integer types (e.g., `NonZeroU32`) as the underlying type for indices. This allows the Rust compiler to perform "niche optimization," where `Option<MyIndex>` takes up the same amount of space as `MyIndex` itself.

```rust
#[derive(IndexType, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct SafeIndex(NonZeroU32);

assert_eq!(core::mem::size_of::<SafeIndex>(), core::mem::size_of::<Option<SafeIndex>>());
```
