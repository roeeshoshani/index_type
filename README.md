# index_type

A Rust crate providing **strongly typed indices** for collections, designed for both `std` and `no_std` environments.

### Overview

This crate allows you to define custom index types for your collections, providing compile-time
guarantees that indices from one collection cannot be accidentally used with another. It also
supports using smaller integer types for indices to save memory in large data structures.

### Features

- **Type Safety**: Prevents accidental misuse of indices between different collections at compile time
- **`no_std` Support**: Works in embedded systems and other `no_std` environments
- **Memory Efficiency**: Use smaller integer types (e.g., `u8`, `u16`) for indices when you know your collection won't exceed a certain size
- **Niche Optimization**: Supports `NonZero` types for `Option<T>` space optimization
- **Rich Collection Support**: Provides [`TypedSlice`](crate::typed_slice::TypedSlice), [`TypedVec`](crate::typed_vec::TypedVec), [`TypedArray`](crate::typed_array::TypedArray), and [`TypedArrayVec`](crate::typed_array_vec::TypedArrayVec) - thin wrappers around standard library types with typed indexing
- **Derive Macros**: Easy to define custom index types with `#[derive(IndexType)]`
- **Range Iterators**: Iterate over ranges using custom index types

### Quick Start

```rust
use index_type::IndexType;
use index_type::typed_vec::TypedVec;

#[derive(IndexType, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct MyIndex(u32);

let mut vec: TypedVec<MyIndex, i32> = TypedVec::new();
let idx = vec.push(42);

assert_eq!(vec[idx], 42);
// vec[0usize]; // This won't compile - requires MyIndex type
```

### Defining Custom Index Types

Use the `#[derive(IndexType)]` macro on a newtype struct:

```rust
use index_type::IndexType;

#[derive(IndexType, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct MyIndex(u32);
```

By default, this generates an error type `MyIndexTooBigError`. You can specify a custom error:

```rust
use index_type::IndexType;
use index_type::IndexTooBigError;

#[derive(Debug, IndexTooBigError)]
#[index_too_big_error(msg = "item id too big")]
struct ItemIdTooBigError;

#[derive(IndexType, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[index_type(error = ItemIdTooBigError)]
struct ItemId(u32);
```

### Typed Collections

#### TypedVec

A growable vector with typed indexing. See [`TypedVec`](crate::typed_vec::TypedVec) for the full API.

```rust
use index_type::IndexType;
use index_type::typed_vec::TypedVec;

#[derive(IndexType, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct NodeId(u32);

let mut nodes: TypedVec<NodeId, String> = TypedVec::new();
let id0 = nodes.push("Alice".to_string());
let id1 = nodes.push("Bob".to_string());

println!("Node 0: {}", nodes[id0]);
```

All operations that can fail due to index overflow have both panicking and fallible variants:

```rust
let mut vec: TypedVec<MyIndex, i32> = TypedVec::new();
vec.push(1);                              // Panics if index too big
let result = vec.try_push(2);             // Returns [`Result`](core::result::Result)<(), Error>
```

#### TypedSlice

A slice wrapper with typed indexing. See [`TypedSlice`](crate::typed_slice::TypedSlice) for the full API.

```rust
use index_type::IndexType;
use index_type::typed_vec::TypedVec;
use index_type::typed_slice::TypedSlice;

#[derive(IndexType, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct RowId(u16);

let vec: TypedVec<RowId, f64> = TypedVec::from_vec(vec![1.0, 2.0, 3.0]);
let slice: &TypedSlice<RowId, f64> = vec.as_slice();

// Safe indexing with custom type
let first = slice[RowId::ZERO];
```

#### TypedArray

A fixed-size array with typed indexing. See [`TypedArray`](crate::typed_array::TypedArray) for the full API.

```rust
use index_type::IndexType;
use index_type::typed_array::TypedArray;

#[derive(IndexType, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct PixelIdx(u8);

let mut pixels: TypedArray<PixelIdx, [u8; 3], 4> = TypedArray::default();
pixels[PixelIdx::ZERO] = [255, 0, 0];  // Red
```

#### TypedArrayVec

A fixed-capacity vector with typed indexing - ideal for embedded systems. See [`TypedArrayVec`](crate::typed_array_vec::TypedArrayVec) for the full API.

```rust
use index_type::IndexType;
use index_type::typed_array_vec::TypedArrayVec;

#[derive(IndexType, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct BufferIndex(u8);

let mut buffer: TypedArrayVec<BufferIndex, u8, 16> = TypedArrayVec::new();
buffer.push(42);
assert_eq!(buffer.len().to_raw_index(), 1);
```

It is also very compact - [`TypedArrayVec<u8, u8, 3>`](crate::typed_array_vec::TypedArrayVec) is only 4 bytes (3 bytes for the array + 1 byte for the length).

### Memory-Efficient Indices

Using smaller integer types reduces memory usage when storing many indices:

```rust
#[derive(IndexType, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct SmallIndex(u8);  // Only 1 byte per index!

// Compare memory usage:
println!("SmallIndex: {} bytes", std::mem::size_of::<SmallIndex>());
println!("u32 index: {} bytes", std::mem::size_of::<u32>());
```

For collections that will never exceed 255 elements, `u8` saves 75% memory compared to `u32`.

### NonZero Indices and Niche Optimization

Using [`NonZero`](core::num::NonZero) types enables niche optimization, where `Option<Index>` has the same size as `Index`:

```rust
use index_type::IndexType;
use core::num::NonZeroU32;

#[derive(IndexType, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct SafeId(NonZeroU32);

// Option<SafeId> takes only 4 bytes, not 8!
assert_eq!(std::mem::size_of::<SafeId>(), 4);
assert_eq!(std::mem::size_of::<Option<SafeId>>(), 4);
```

### Range Iterators

Standard Rust range types ([`Range`](core::ops::Range), [`RangeFrom`](core::ops::RangeFrom), [`RangeInclusive`](core::ops::RangeInclusive)) require the [`Step`](core::iter::Step)
trait to iterate, which is currently unstable. To iterate over ranges with custom index
types, use the [`TypedRangeIterExt`](crate::typed_range_iter::TypedRangeIterExt) extension trait:

```rust
use index_type::IndexType;
use index_type::typed_range_iter::TypedRangeIterExt;

#[derive(IndexType, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct MyIdx(u32);

let start = MyIdx(5);
let end = MyIdx(10);

for idx in (start..end).iter() {
    println!("{:?}", idx);
}
```

### Macros

Convenience macros for creating typed collections:

```rust
use index_type::{typed_vec, typed_array, typed_slice, typed_slice_mut, IndexType};
use index_type::typed_vec::TypedVec;
use index_type::typed_array::TypedArray;
use index_type::typed_slice::TypedSlice;

#[derive(IndexType, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct MyIndex(u32);

// Create a TypedVec
let v: TypedVec<MyIndex, i32> = typed_vec![1, 2, 3];

// Create a TypedArray
let a: TypedArray<MyIndex, i32, 3> = typed_array![1, 2, 3];

// Create a TypedSlice reference
let s: &TypedSlice<MyIndex, i32> = typed_slice![1, 2, 3];
```

### Trait Implementations

The typed collections implement many standard library traits:

- [`Deref`](core::ops::Deref) / [`DerefMut`](core::ops::DerefMut) → [`TypedSlice`](crate::typed_slice::TypedSlice)
- [`IntoIterator`] for values, references, and mutable references
- [`FromIterator`] and [`Extend`]
- [`AsRef`] / [`AsMut`] for [`TypedSlice`](crate::typed_slice::TypedSlice) and [`TypedVec`](crate::typed_vec::TypedVec)
- [`PartialEq`], [`Eq`], [`PartialOrd`], [`Ord`], [`Hash`]
- [`Debug`], [`Clone`], [`Default`]
- [`Index`](core::ops::Index) / [`IndexMut`](core::ops::IndexMut) with typed indices

### Error Handling

Operations that can fail due to index overflow return `Result` types with a specific error:

```rust
use index_type::IndexType;
use index_type::typed_vec::TypedVec;

#[derive(IndexType, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct MyIndex(u8);  // MAX_RAW_INDEX = 255

let mut vec: TypedVec<MyIndex, i32> = TypedVec::new();

// Fill up to capacity
for i in 0..255 {
    vec.try_push(i).unwrap();
}

// This fails gracefully
assert!(vec.try_push(255).is_err());
```

### no_std Compatibility

This crate is `no_std` compatible. Enable the `alloc` feature for heap-allocated collections (`TypedVec`):

```toml
[dependencies]
index_type = { version = "0.1", features = ["alloc"] }
```
