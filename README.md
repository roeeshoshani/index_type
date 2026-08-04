<!-- cargo-reedme: start -->

<!-- cargo-reedme: info-start

    Do not edit this region by hand
    ===============================

    This region was generated from Rust documentation comments by `cargo-reedme` using this command:

        cargo +nightly reedme

    for more info: https://github.com/nik-rev/cargo-reedme

cargo-reedme: info-end -->

A Rust library providing **strongly typed indices** for collections and everything else needed for working with them in an
ergonomic manner.

## What are typed indices?

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

## Features

- **Type Safety**: Prevents accidental misuse of indices between different collections at compile time
- **`no_std` Support**: Works in embedded systems and other `no_std` environments
- **Memory Efficiency**: Use smaller integer types (`u8`, `u16`) for indices when collections are bounded
- **Niche Optimization**: Supports [`NonZero`](https://doc.rust-lang.org/stable/core/num/nonzero/struct.NonZero.html) types so `Option<Index>` has the same size as `Index`
- **Rich Collections**: Provides [`TypedSlice`](https://docs.rs/index_type/latest/index_type/slice/struct.TypedSlice.html), [`TypedVec`](https://docs.rs/index_type/latest/index_type/vec/struct.TypedVec.html), [`TypedArray`](https://docs.rs/index_type/latest/index_type/array/struct.TypedArray.html), and [`TypedArrayVec`](https://docs.rs/index_type/latest/index_type/array_vec/struct.TypedArrayVec.html)
- **Derive Macros**: Easy to define custom index types with `#[derive(IndexType)]`
- **Range Iterators**: Iterate over ranges using custom index types

## Quick Start

```rust
use index_type::{IndexType, vec::TypedVec};

#[derive(IndexType, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct MyIndex(u32);

let mut vec: TypedVec<MyIndex, i32> = TypedVec::new();
let idx = vec.push(42);

assert_eq!(vec[idx], 42);
// vec[0usize]; // This won't compile - requires MyIndex type
```

## Defining Index Types

Use the `#[derive(IndexType)]` macro on a newtype struct:

```rust
use index_type::IndexType;

#[derive(IndexType, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct MyIndex(u32);
```

The macro automatically implements the [`IndexType`](https://docs.rs/index_type/latest/index_type/trait.IndexType.html) trait for your custom type. By default,
it generates an error type `MyIndexTooBigError`. You can specify a custom error type:

```rust
#[derive(Debug, IndexTooBigError)]
#[index_too_big_error(msg = "item id too big")]
struct ItemIdTooBigError;

#[derive(IndexType, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[index_type(error = ItemIdTooBigError)]
struct ItemId(u32);
```

## Typed Collections

### TypedVec

A growable vector with typed indexing. See [`TypedVec`](https://docs.rs/index_type/latest/index_type/vec/struct.TypedVec.html) for the full API.

```rust
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

// This will panic on index overflow (e.g. if the vector already contains (2^32 - 1) elements before calling `push`)
let idx = vec.push(1);

// This will gracefully return an error in case of index overflow
let result: Result<MyIndex, MyIndexTooBigError> = vec.try_push(2);
```

### TypedSlice

A slice wrapper with typed indexing.
`TypedSlice<I, T>` is the same as `[T]` but with index type `I`.
So, to represent `&[u8]` for example, use `&TypedSlice<I, u8>`, where `I` is your custom index type.
See [`TypedSlice`](https://docs.rs/index_type/latest/index_type/slice/struct.TypedSlice.html) for the full API.

```rust
#[derive(IndexType, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct RowId(u16);

let vec: TypedVec<RowId, f64> = typed_vec![1.0, 2.0, 3.0];
let slice: &TypedSlice<RowId, f64> = vec.as_slice();

// Safe indexing with custom type
let first = slice[RowId::ZERO];
```

### TypedArray

A fixed-size array with typed indexing. The array length `N` is checked at compile time
to ensure it fits within the index type’s range. See [`TypedArray`](https://docs.rs/index_type/latest/index_type/array/struct.TypedArray.html) for the full API.

```rust
#[derive(IndexType, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct ValueIdx(u8);

#[derive(Debug, PartialEq, Eq)]
struct Value(u32);

// An index-typed version of `[Value; 3]`, with index type `ValueIdx`
let mut values: TypedArray<ValueIdx, Value, 3> = TypedArray::from_array([Value(3), Value(7), Value(5)]);
values[ValueIdx::ZERO] = Value(20);
values[ValueIdx(1)] = Value(32);
assert_eq!(values[ValueIdx(0)], Value(20));
assert_eq!(values[ValueIdx(1)], Value(32));
assert_eq!(values[ValueIdx(2)], Value(5));
```

### TypedArrayVec

A fixed-capacity vector backed by an array, similar to the `ArrayVec` type provided by the `arrayvec` crate but with typed indexing.
See [`TypedArrayVec`](https://docs.rs/index_type/latest/index_type/array_vec/struct.TypedArrayVec.html) for the full API.

```rust
#[derive(IndexType, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct BufferIndex(u8);

let mut buffer: TypedArrayVec<BufferIndex, u8, 16> = TypedArrayVec::new();
buffer.push(42);
assert_eq!(buffer.len().to_raw_index(), 1);
```

A `TypedArrayVec<u8, u8, 3>` is only 4 bytes (3 bytes for data + 1 byte for length).

## Complex Indexing

This crate also supports complex forms of indexing when using custom index types, for example, slicing a collection with a range
of a custom index type:

```rust
#[derive(IndexType, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct ItemId(usize);

#[derive(Debug, PartialEq, Eq)]
struct Item(u32);

let values: TypedVec<ItemId, Item> = typed_vec![
    Item(45), Item(54), Item(32), Item(19), Item(78)
];

let some_values: &TypedSlice<ItemId, Item> = &values[ItemId(1)..ItemId(4)];
assert_eq!(some_values.as_slice(), &[Item(54), Item(32), Item(19)]);

// Can even perform more complex types of slicing
let other_values: &TypedSlice<ItemId, Item> = &values[..ItemId(3)];
assert_eq!(other_values.as_slice(), &[Item(45), Item(54), Item(32)]);

let other_values_2: &TypedSlice<ItemId, Item> = &values[ItemId(3)..];
assert_eq!(other_values_2.as_slice(), &[Item(19), Item(78)]);
```

## Memory-Efficient Indices

Using smaller integer types reduces memory when storing many indices.
This is useful when you know that the size of the collection is bounded.

For example, if you are implementing a graph using an adjacency list, and you know that the graph will be reasonably small, you can use
32-bit integers as indices instead of `usize`, which on 64-bit machines is half the size:

```rust
// We know that the graph will never have more than `2^32 - 1` nodes, so we can use `u32` as the index type.
#[derive(IndexType, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct NodeId(u32);

struct Node {
    // Each node id is only 32 bits, compared to `usize` which is 64 bits (assuming we are running on a 64-bit machine), which
    // may save a lot of space for large graphs.
    children: Vec<NodeId>,
}

struct Graph {
    nodes: TypedVec<NodeId, Node>,
    root: NodeId,
}
```

## NonZero Indices and Niche Optimization

Using [`NonZero`](https://doc.rust-lang.org/stable/core/num/nonzero/struct.NonZero.html) types enables niche optimization, where `Option<Index>`
has the same size as `Index`:

```rust
#[derive(IndexType, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct SafeId(NonZeroU32);

// Option<SafeId> takes only 4 bytes, not 8!
assert_eq!(size_of::<SafeId>(), 4);
assert_eq!(size_of::<Option<SafeId>>(), 4);
```

And indexing into a collection with non-zero indices is of course as seamless as using any other integer type as the index type:

```rust
#[derive(IndexType, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct MyId(NonZeroU32);

let arr: TypedArray<MyId, i32, 4> = typed_array![7, 12, 19, 22];
assert_eq!(arr[MyId::from_raw_index(2)], 19);
```

## Range Iterators

Currently, in stable Rust, you cannot iterate over a range of values of a custom type:

```compile_fail,E0277
struct MyIdx(u32);

// There is nothing you can do to make this code work in stable Rust
for i in MyIdx(0)..MyIdx(20) {}
```

The reason for this is that the built-in range types only implement the [`Iterator`](https://doc.rust-lang.org/stable/core/iter/traits/iterator/trait.Iterator.html) trait if the value type `T` implements the
unstable [`Step`](https://doc.rust-lang.org/stable/core/iter/range/trait.Step.html) trait, which you cannot implement for your own types in stable Rust.

Being able to iterate over ranges of custom index types is important for making the experience of working with typed indices
feel seamless and as smooth as using regular index types.

This crate provides [`TypedRangeIterExt`](https://docs.rs/index_type/latest/index_type/range/trait.TypedRangeIterExt.html) for iterating over ranges with custom index types:

```rust
use index_type::range::TypedRangeIterExt;

#[derive(IndexType, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct MyIdx(u32);

for idx in (MyIdx(5)..MyIdx(10)).iter() {
    println!("{:?}", idx);
}
```

## Typed Enumerate

Use [`TypedIteratorExt`](https://docs.rs/index_type/latest/index_type/enumerate/trait.TypedIteratorExt.html) to enumerate any iterator with typed indices:

```rust
use index_type::enumerate::TypedIteratorExt;

#[derive(IndexType, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct MyIdx(u32);

let pairs: Vec<(MyIdx, &str)> = ["a", "b", "c"]
    .into_iter()
    .typed_enumerate::<MyIdx>()
    .collect();

assert_eq!(pairs[1].0, MyIdx(1));
assert_eq!(pairs[1].1, "b");
```

## Macros

Convenience macros for creating typed collections:

```rust
#[derive(IndexType, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct MyIndex(u32);

// Create a TypedVec
let v: TypedVec<MyIndex, i32> = typed_vec![1, 2, 3];

// Create a TypedArray
let a: TypedArray<MyIndex, i32, 3> = typed_array![1, 2, 3];

// Create a TypedArrayVec
let av: TypedArrayVec<MyIndex, u8, 4> = typed_array_vec![1, 2];

// Create a TypedSlice reference, similar to a slice literal (`&[1, 2, 3]`)
let s: &TypedSlice<MyIndex, i32> = typed_slice![1, 2, 3];
```

## Error Handling

Operations that can fail due to index overflow return `Result` types.
Each index type has its own custom error type which is returned when operating on a collection which uses that index type.

```rust
// Note: the `#[derive(IndexType)]` macro automatically generates a type called `MyIndexTooBigError` which is the custom error
// type for this custom index type.
#[derive(IndexType, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct MyIndex(u8);

let mut vec: TypedVec<MyIndex, i32> = TypedVec::new();

// Fill up to capacity
for i in 0..255 {
    vec.try_push(i).unwrap();
}

// At this point, pushing will cause the length of the vec to exceed the index type, so this fails gracefully.
let res: Result<MyIndex, MyIndexTooBigError> = vec.try_push(255);
assert!(res.is_err());
```

## no_std Compatibility

This crate is `no_std` compatible. The `alloc` feature (enabled by default) enables
heap-allocated collections ([`TypedVec`](https://docs.rs/index_type/latest/index_type/vec/struct.TypedVec.html) and related macros).

For pure `no_std` environments without heap allocation, disable the `alloc` feature:

```toml
[dependencies]
index_type = { version = "...", default-features = false }
```

## `serde` Support

This crate has a `serde` feature flag which implements `Serialize` and `Deserialize` for all of the relevant types exported by this
crate. This includes for example the main collection types (e.g. [`TypedVec`](https://docs.rs/index_type/latest/index_type/vec/struct.TypedVec.html)).

<!-- cargo-reedme: end -->
