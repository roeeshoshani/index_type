# index_type

A Rust crate providing strongly typed indices for slices, vectors, and arrays, designed for `no_std` environments.

## Overview

This crate allows you to define custom index types for your collections, providing better type safety and preventing accidental use of indices from one collection with another. It also supports using smaller integer types for indices to save memory when you know your collection won't exceed a certain size.

## Features

- **Typed Indices**: Define custom types for indices using the [`IndexType`] derive macro.
- **`no_std` Support**: Designed to work in embedded or other `no_std` environments.
- **Memory Efficiency**: Use smaller integer types (e.g., `u8`, `u16`) as indices for memory-constrained applications.
- **Rich Collection Support**: Provides [`TypedSlice`](crate::typed_slice::TypedSlice), [`TypedVec`](crate::typed_vec::TypedVec), and [`TypedArray`](crate::typed_array::TypedArray) which are thin wrappers around the standard library's slice, `Vec`, and array types.

## Usage

Add `index_type` to your `Cargo.toml`:

```toml
[dependencies]
index_type = "0.1.0"
```

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

```rust
#[derive(IndexType, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct SmallIndex(u8);

// TypedVec<SmallIndex, T> can only hold up to 255 elements.
// This is useful for saving memory in large data structures containing many indices.
```

## Safety

This crate uses `unsafe` code for performance optimizations (e.g., `transmute` between `repr(transparent)` wrappers and raw slices/vectors). All `unsafe` blocks are documented with `SAFETY` comments explaining why they are safe.
