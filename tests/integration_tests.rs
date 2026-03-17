use index_type::typed_slice::TypedSlice;
use index_type::typed_vec::TypedVec;
use index_type::IndexType;
use core::num::NonZeroU8;

#[test]
fn test_index_type_u8() {
    let index = u8::try_from_index(10).unwrap();
    assert_eq!(index.to_index(), 10);
    assert_eq!(u8::ZERO, 0);

    assert!(u8::try_from_index(255).is_ok());
    // u8 can represent up to 255 elements. length 256 is too big.
    assert!(u8::try_from_index(256).is_err());
}

#[test]
fn test_index_type_nonzero_u8() {
    // NonZeroU8 maps 0..254 to 1..255
    let index = NonZeroU8::try_from_index(0).unwrap();
    assert_eq!(index.get(), 1);
    assert_eq!(index.to_index(), 0);

    let index = NonZeroU8::try_from_index(254).unwrap();
    assert_eq!(index.get(), 255);
    assert_eq!(index.to_index(), 254);

    assert!(NonZeroU8::try_from_index(255).is_err());
}

#[test]
fn test_typed_slice() {
    let data = [1, 2, 3, 4, 5];
    let slice = TypedSlice::<u8, i32>::from_slice(&data).unwrap();

    assert_eq!(slice.len_usize(), 5);
    assert_eq!(slice.len(), 5u8);

    assert_eq!(slice[0u8], 1);
    assert_eq!(slice[4u8], 5);

    let subslice = &slice[1u8..3u8];
    assert_eq!(subslice.len(), 2u8);
    assert_eq!(subslice[0u8], 2);
    assert_eq!(subslice[1u8], 3);
}

#[test]
fn test_typed_vec() {
    let mut vec = TypedVec::<u8, i32>::new();
    assert!(vec.is_empty());

    vec.push(10).unwrap();
    vec.push(20).unwrap();
    vec.push(30).unwrap();

    assert_eq!(vec.len(), 3u8);
    assert_eq!(vec.as_slice()[1u8], 20);

    assert_eq!(vec.pop(), Some(30));
    assert_eq!(vec.len(), 2u8);

    vec.insert(1u8, 15);
    assert_eq!(vec.as_slice()[1u8], 15);
    assert_eq!(vec.as_slice()[2u8], 20);

    let removed = vec.remove(1u8);
    assert_eq!(removed, 15);
    assert_eq!(vec.len(), 2u8);
}

#[test]
fn test_typed_slice_binary_search() {
    let data = [10, 20, 30, 40, 50];
    let slice = TypedSlice::<u8, i32>::from_slice(&data).unwrap();

    assert_eq!(slice.binary_search(&30), Ok(2u8));
    assert_eq!(slice.binary_search(&25), Err(2u8));
}

#[test]
fn test_typed_slice_get_disjoint_mut() {
    let mut data = [1, 2, 3, 4, 5];
    let slice = TypedSlice::<u8, i32>::from_slice_mut(&mut data).unwrap();

    let [a, b] = slice.get_disjoint_mut([1u8, 3u8]).unwrap();
    *a = 20;
    *b = 40;

    assert_eq!(data, [1, 20, 3, 40, 5]);
}

#[test]
fn test_typed_vec_limit() {
    let mut vec = TypedVec::<u8, i32>::new();
    // u8 can represent indices up to 255.
    // So it can hold 255 elements (indices 0..254).
    // Wait, if it can hold 255 elements, the length is 255.
    // try_from_index(255) for u8 returns Ok(255).
    // So it can hold 255 elements.
    for i in 0..255 {
        if let Err(e) = vec.push(i as i32) {
            panic!("failed at i={}: {:?}", i, e);
        }
    }
    assert_eq!(vec.len(), 255u8);
    // Adding one more would make length 256, which doesn't fit in u8.
    assert!(vec.push(255).is_err());
}
