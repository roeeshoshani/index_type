use index_type::{IndexType, typed_array::TypedArray};

#[derive(IndexType, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct MyIndex(u32);

#[test]
fn test_typed_array_basic() {
    let arr: TypedArray<MyIndex, i32, 3> = TypedArray::try_from_array([1, 2, 3]).unwrap();
    assert_eq!(arr.len_usize(), 3);
    assert_eq!(arr[MyIndex::ZERO], 1);
    assert_eq!(arr[unsafe { MyIndex::from_raw_index_unchecked(1) }], 2);
    assert_eq!(arr[unsafe { MyIndex::from_raw_index_unchecked(2) }], 3);
}

#[test]
fn test_cast_index_type_upcast() {
    #[derive(IndexType, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    struct SmallIndex(u8);

    let arr: TypedArray<SmallIndex, i32, 5> = TypedArray::try_from_array([1, 2, 3, 4, 5]).unwrap();

    let cast: TypedArray<MyIndex, i32, 5> = arr.cast_index_type::<MyIndex>().unwrap();
    assert_eq!(cast.len_usize(), 5);
    assert_eq!(cast[MyIndex::ZERO], 1);
}

#[test]
fn test_cast_index_type_downcast() {
    #[derive(IndexType, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    struct SmallIndex(u8);

    let arr: TypedArray<MyIndex, i32, 5> = TypedArray::try_from_array([1, 2, 3, 4, 5]).unwrap();

    let result = arr.cast_index_type::<SmallIndex>();
    assert!(result.is_ok());
}

#[test]
fn test_cast_index_type_downcast_fails() {
    #[derive(IndexType, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    struct SmallIndex(u8);

    let arr: TypedArray<MyIndex, i32, 300> = TypedArray::try_from_array([0; 300]).unwrap();

    let result = arr.cast_index_type::<SmallIndex>();
    assert!(result.is_err());
}

#[test]
fn test_cast_index_type_same() {
    let arr: TypedArray<MyIndex, i32, 5> = TypedArray::try_from_array([1, 2, 3, 4, 5]).unwrap();

    let cast: TypedArray<MyIndex, i32, 5> = arr.cast_index_type::<MyIndex>().unwrap();
    assert_eq!(cast.len_usize(), 5);
}

#[test]
fn test_cast_index_type_mut_downcast_fails() {
    #[derive(IndexType, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    struct SmallIndex(u8);

    let mut arr: TypedArray<MyIndex, i32, 300> = TypedArray::try_from_array([0; 300]).unwrap();

    let result = arr.cast_index_type_mut::<SmallIndex>();
    assert!(result.is_err());
}
