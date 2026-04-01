# index_type

A Rust library providing **strongly typed indices** for collections, designed for both `std` and `no_std` environments.

### What are typed indices?

In standard Rust, collections use `usize` for indexing. This works well but provides no compile-time
protection against using an index from one collection with another. Typed indices solve this by
creating custom index types that are statically associated with specific collections.

In standard Rust, a raw `usize` can index any collection. This allows subtle bugs:
```rust
let nodes: Vec<Node> = vec![Node::default(); 10];  // 10 nodes
let edges: Vec<Edge> = vec![Edge::default(); 5];   // 5 edges
let node_index = 3;
nodes[node_index];
edges[node_index]; // compiles just fine!
```

With typed indices, cross-contamination becomes a compile error:
```rust
#[derive(IndexType, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct NodeId(u32);

#[derive(IndexType, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct EdgeId(u32);

let nodes: TypedVec<NodeId, Node> = typed_vec![Node::default(); 10];
let edges: TypedVec<EdgeId, Edge> = typed_vec![Edge::default(); 10];
let node_id = NodeId(3);
nodes[node_id]; // OK
// edges[node_id]; // COMPILE ERROR: expected EdgeId, found NodeId
```

### Features

- **Type Safety**: Prevents accidental misuse of indices between different collections at compile time
- **`no_std` Support**: Works in embedded systems and other `no_std` environments
- **Memory Efficiency**: Use smaller integer types (`u8`, `u16`) for indices when collections are bounded
- **Niche Optimization**: Supports [`NonZero`](core::num::NonZero) types so `Option<Index>` has the same size as `Index`
- **Rich Collections**: Provides [`TypedSlice`](crate::typed_slice::TypedSlice), [`TypedVec`](crate::typed_vec::TypedVec), [`TypedArray`](crate::typed_array::TypedArray), and [`TypedArrayVec`](crate::typed_array_vec::TypedArrayVec)
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

### Defining Index Types

Use the `#[derive(IndexType)]` macro on a newtype struct:

```rust
use index_type::IndexType;

#[derive(IndexType, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct MyIndex(u32);
```

The macro automatically implements the [`IndexType`] trait for your custom type. By default,
it generates an error type `MyIndexTooBigError`. You can specify a custom error type:

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

Operations that can fail due to index overflow have both panicking and fallible variants:

```rust
let mut vec: TypedVec<MyIndex, i32> = TypedVec::new();
vec.push(1);                              // Panics if index too big
let result = vec.try_push(2);             // Returns Result<(), Error>
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

A fixed-size array with typed indexing. The array length `N` is checked at compile time
to ensure it fits within the index type's range. See [`TypedArray`](crate::typed_array::TypedArray) for the full API.

```rust
use index_type::IndexType;
use index_type::typed_array::TypedArray;

#[derive(IndexType, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct PixelIdx(u8);

let mut pixels: TypedArray<PixelIdx, [u8; 3], 4> = TypedArray::default();
pixels[PixelIdx::ZERO] = [255, 0, 0];  // Red
pixels[PixelIdx(1)] = [0, 255, 0];     // Green
```

#### TypedArrayVec

A fixed-capacity vector ideal for embedded systems. It never allocates after creation.
See [`TypedArrayVec`](crate::typed_array_vec::TypedArrayVec) for the full API.

```rust
use index_type::IndexType;
use index_type::typed_array_vec::TypedArrayVec;

#[derive(IndexType, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct BufferIndex(u8);

let mut buffer: TypedArrayVec<BufferIndex, u8, 16> = TypedArrayVec::new();
buffer.push(42);
assert_eq!(buffer.len().to_raw_index(), 1);
```

A `TypedArrayVec<u8, u8, 3>` is only 4 bytes (3 bytes for data + 1 byte for length).

### Memory-Efficient Indices

Using smaller integer types reduces memory when storing many indices:

```rust
#[derive(IndexType, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct SmallIndex(u8);  // Only 1 byte per index!

println!("SmallIndex: {} bytes", std::mem::size_of::<SmallIndex>());
println!("u32 index: {} bytes", std::mem::size_of::<u32>());
```

For collections with at most 255 elements, `u8` saves 75% memory compared to `u32`.

### NonZero Indices and Niche Optimization

Using [`NonZero`](core::num::NonZero) types enables niche optimization, where `Option<Index>`
has the same size as `Index`:

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

Standard Rust ranges require the unstable [`Step`](core::iter::Step) trait. This crate provides
[`TypedRangeIterExt`](crate::typed_range::TypedRangeIterExt) for iterating over ranges with custom index types:

```rust
use index_type::IndexType;
use index_type::typed_range::TypedRangeIterExt;

#[derive(IndexType, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct MyIdx(u32);

let start = MyIdx(5);
let end = MyIdx(10);

for idx in (start..end).iter() {
    println!("{:?}", idx);
}
```

### Typed Enumerate

Use [`TypedIteratorExt`](crate::typed_enumerate::TypedIteratorExt) to enumerate any iterator with typed indices:

```rust
use index_type::IndexType;
use index_type::typed_enumerate::TypedIteratorExt;

#[derive(IndexType, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct RowIdx(u32);

let pairs: Vec<_> = ["a", "b", "c"]
    .into_iter()
    .typed_enumerate::<RowIdx>()
    .collect();

assert_eq!(pairs[1].0, RowIdx(1));
assert_eq!(pairs[1].1, "b");
```

### Macros

Convenience macros for creating typed collections:

```rust
use index_type::{typed_vec, typed_array, typed_array_vec, typed_slice, typed_slice_mut, IndexType};
use index_type::typed_vec::TypedVec;
use index_type::typed_array::TypedArray;
use index_type::typed_array_vec::TypedArrayVec;
use index_type::typed_slice::TypedSlice;

#[derive(IndexType, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct MyIndex(u32);

// Create a TypedVec
let v: TypedVec<MyIndex, i32> = typed_vec![1, 2, 3];

// Create a TypedArray
let a: TypedArray<MyIndex, i32, 3> = typed_array![1, 2, 3];

// Create a TypedArrayVec
let av: TypedArrayVec<MyIndex, u8, 4> = typed_array_vec![1, 2, 3, 4];

// Create a TypedSlice reference
let s: &TypedSlice<MyIndex, i32> = typed_slice![1, 2, 3];
```

### Error Handling

Operations that can fail due to index overflow return `Result` types:

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

This crate is `no_std` compatible. The `alloc` feature (enabled by default) enables
heap-allocated collections ([`TypedVec`](crate::typed_vec::TypedVec) and related macros).

For pure `no_std` environments without heap allocation, disable the `alloc` feature:

```toml
[dependencies]
index_type = { version = "...", default-features = false }
```


License: MIT OR Apache-2.0
