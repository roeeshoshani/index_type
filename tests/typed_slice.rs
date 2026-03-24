use index_type::{
    typed_slice::{GetDisjointMutError, TypedSlice},
    IndexType,
};

#[derive(IndexType, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct MyIndex(u32);

#[test]
fn test_typed_slice_basic() {
    let mut data = [10, 20, 30, 40, 50];
    let slice = TypedSlice::<MyIndex, i32>::try_from_slice_mut(&mut data).unwrap();

    assert_eq!(slice.len_usize(), 5);
    assert_eq!(slice[MyIndex::ZERO], 10);

    let sub_slice = &slice[unsafe { MyIndex::from_raw_index_unchecked(1) }..unsafe {
        MyIndex::from_raw_index_unchecked(4)
    }];
    assert_eq!(sub_slice.len_usize(), 3);
    assert_eq!(sub_slice[MyIndex::ZERO], 20);
}

#[test]
fn test_cast_index_type_upcast() {
    #[derive(IndexType, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    struct SmallIndex(u8);

    let mut data = [1, 2, 3, 4, 5];
    let slice: &TypedSlice<SmallIndex, i32> = TypedSlice::try_from_slice_mut(&mut data).unwrap();

    let cast: &TypedSlice<MyIndex, i32> = slice.cast_index_type::<MyIndex>().unwrap();
    assert_eq!(cast.len_usize(), 5);
    assert_eq!(cast[MyIndex::ZERO], 1);
}

#[test]
fn test_cast_index_type_downcast() {
    #[derive(IndexType, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    struct SmallIndex(u8);

    let mut data = [1, 2, 3, 4, 5];
    let slice: &TypedSlice<MyIndex, i32> = TypedSlice::try_from_slice_mut(&mut data).unwrap();

    let result = slice.cast_index_type::<SmallIndex>();
    assert!(result.is_ok());
}

#[test]
fn test_cast_index_type_downcast_fails() {
    #[derive(IndexType, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    struct SmallIndex(u8);

    let mut data = [1; 300];
    let slice: &TypedSlice<MyIndex, i32> = TypedSlice::try_from_slice_mut(&mut data).unwrap();

    let result = slice.cast_index_type::<SmallIndex>();
    assert!(result.is_err());
}

#[test]
fn test_cast_index_type_same() {
    let mut data = [1, 2, 3, 4, 5];
    let slice: &TypedSlice<MyIndex, i32> = TypedSlice::try_from_slice_mut(&mut data).unwrap();

    let cast: &TypedSlice<MyIndex, i32> = slice.cast_index_type::<MyIndex>().unwrap();
    assert_eq!(cast.len_usize(), 5);
}

#[test]
fn test_cast_index_type_mut_upcast() {
    #[derive(IndexType, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    struct SmallIndex(u8);

    let mut data = [1, 2, 3, 4, 5];
    let slice: &mut TypedSlice<SmallIndex, i32> =
        TypedSlice::try_from_slice_mut(&mut data).unwrap();

    let cast: &mut TypedSlice<MyIndex, i32> = slice.cast_index_type_mut::<MyIndex>().unwrap();
    assert_eq!(cast.len_usize(), 5);
    cast[MyIndex::ZERO] = 100;
    assert_eq!(data[0], 100);
}

#[test]
fn test_cast_index_type_mut_downcast_fails() {
    #[derive(IndexType, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    struct SmallIndex(u8);

    let mut data = [1; 300];
    let slice: &mut TypedSlice<MyIndex, i32> = TypedSlice::try_from_slice_mut(&mut data).unwrap();

    let result = slice.cast_index_type_mut::<SmallIndex>();
    assert!(result.is_err());
}

#[test]
fn test_repeat() {
    let mut data = [1, 2];
    let slice: &TypedSlice<MyIndex, i32> = TypedSlice::try_from_slice(&data).unwrap();

    let repeated = slice.repeat(3).unwrap();
    assert_eq!(repeated.len_usize(), 6);
    assert_eq!(repeated[MyIndex::ZERO], 1);
    assert_eq!(repeated[unsafe { MyIndex::from_raw_index_unchecked(1) }], 2);
    assert_eq!(repeated[unsafe { MyIndex::from_raw_index_unchecked(2) }], 1);
    assert_eq!(repeated[unsafe { MyIndex::from_raw_index_unchecked(5) }], 2);
}

#[test]
fn test_repeat_overflow() {
    #[derive(IndexType, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    struct SmallIndex(u8);

    let mut data = [1, 2, 3];
    let slice: &TypedSlice<SmallIndex, i32> = TypedSlice::try_from_slice(&mut data).unwrap();

    let result = slice.repeat(100);
    assert!(result.is_err());
}

#[test]
fn test_repeat_zero() {
    let mut data = [1, 2];
    let slice: &TypedSlice<MyIndex, i32> = TypedSlice::try_from_slice(&mut data).unwrap();

    let repeated = slice.repeat(0).unwrap();
    assert!(repeated.is_empty());
}

#[test]
fn test_repeat_one() {
    let mut data = [1, 2, 3];
    let slice: &TypedSlice<MyIndex, i32> = TypedSlice::try_from_slice(&mut data).unwrap();

    let repeated = slice.repeat(1).unwrap();
    assert_eq!(repeated.len_usize(), 3);
    assert_eq!(repeated[MyIndex::ZERO], 1);
    assert_eq!(repeated[unsafe { MyIndex::from_raw_index_unchecked(2) }], 3);
}

#[test]
fn test_binary_search_empty() {
    let data: [i32; 0] = [];
    let slice: &TypedSlice<MyIndex, i32> = TypedSlice::try_from_slice(&data).unwrap();

    assert_eq!(slice.binary_search(&1), Err(MyIndex::ZERO));
}

#[test]
fn test_binary_search_not_found() {
    let mut data = [1, 3, 5, 7];
    let slice: &TypedSlice<MyIndex, i32> = TypedSlice::try_from_slice_mut(&mut data).unwrap();

    assert_eq!(
        slice.binary_search(&4),
        Err(unsafe { MyIndex::from_raw_index_unchecked(2) })
    );
}

#[test]
fn test_binary_search_by() {
    let mut data = [(1, "a"), (3, "b"), (5, "c")];
    let slice: &TypedSlice<MyIndex, (i32, &str)> =
        TypedSlice::try_from_slice_mut(&mut data).unwrap();

    let result = slice.binary_search_by(|x| x.0.cmp(&3));
    assert_eq!(result, Ok(unsafe { MyIndex::from_raw_index_unchecked(1) }));
}

#[test]
fn test_binary_search_by_key() {
    let mut data = [(1, "a"), (3, "b"), (5, "c")];
    let slice: &TypedSlice<MyIndex, (i32, &str)> =
        TypedSlice::try_from_slice_mut(&mut data).unwrap();

    let result = slice.binary_search_by_key(&3, |x| x.0);
    assert_eq!(result, Ok(unsafe { MyIndex::from_raw_index_unchecked(1) }));
}

#[test]
fn test_select_nth_unstable() {
    let mut data = [5, 3, 1, 4, 2];
    let slice: &mut TypedSlice<MyIndex, i32> = TypedSlice::try_from_slice_mut(&mut data).unwrap();

    let (left, pivot, right) =
        slice.select_nth_unstable(unsafe { MyIndex::from_raw_index_unchecked(2) });
    assert_eq!(pivot, &3);
    assert!(left.iter().all(|&x| x <= 3));
    assert!(right.iter().all(|&x| x >= 3));
}

#[test]
fn test_split_at_checked() {
    let mut data = [1, 2, 3, 4, 5];
    let slice: &mut TypedSlice<MyIndex, i32> = TypedSlice::try_from_slice_mut(&mut data).unwrap();

    let result = slice.split_at_checked(unsafe { MyIndex::from_raw_index_unchecked(2) });
    assert!(result.is_some());
    let (left, right) = result.unwrap();
    assert_eq!(left.len_usize(), 2);
    assert_eq!(right.len_usize(), 3);
}

#[test]
fn test_split_at_checked_out_of_bounds() {
    let mut data = [1, 2, 3, 4, 5];
    let slice: &mut TypedSlice<MyIndex, i32> = TypedSlice::try_from_slice_mut(&mut data).unwrap();

    let result = slice.split_at_checked(unsafe { MyIndex::from_raw_index_unchecked(10) });
    assert!(result.is_none());
}

#[test]
fn test_split_at_mut_checked() {
    let mut data = [1, 2, 3, 4, 5];
    let slice: &mut TypedSlice<MyIndex, i32> = TypedSlice::try_from_slice_mut(&mut data).unwrap();

    let result = slice.split_at_mut_checked(unsafe { MyIndex::from_raw_index_unchecked(2) });
    assert!(result.is_some());
}

#[test]
fn test_get_disjoint_mut_index_out_of_bounds() {
    let mut data = [1, 2, 3];
    let slice: &mut TypedSlice<MyIndex, i32> = TypedSlice::try_from_slice_mut(&mut data).unwrap();

    let result = slice.get_disjoint_mut([MyIndex::ZERO, unsafe {
        MyIndex::from_raw_index_unchecked(10)
    }]);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), GetDisjointMutError::IndexOutOfBounds);
}

#[test]
fn test_get_disjoint_mut_overlapping() {
    let mut data = [1, 2, 3];
    let slice: &mut TypedSlice<MyIndex, i32> = TypedSlice::try_from_slice_mut(&mut data).unwrap();

    let result = slice.get_disjoint_mut([MyIndex::ZERO, MyIndex::ZERO]);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), GetDisjointMutError::OverlappingIndices);
}

#[test]
fn test_get_disjoint_mut_range_overlapping() {
    let mut data = [1, 2, 3, 4];
    let slice: &mut TypedSlice<MyIndex, i32> = TypedSlice::try_from_slice_mut(&mut data).unwrap();

    let range1: std::ops::Range<MyIndex> =
        MyIndex::ZERO..unsafe { MyIndex::from_raw_index_unchecked(2) };
    let range2: std::ops::Range<MyIndex> = unsafe { MyIndex::from_raw_index_unchecked(1) }
        ..unsafe { MyIndex::from_raw_index_unchecked(3) };
    let result = slice.get_disjoint_mut([range1, range2]);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), GetDisjointMutError::OverlappingIndices);
}

#[test]
fn test_get_disjoint_mut_range_out_of_bounds() {
    let mut data = [1, 2, 3, 4];
    let slice: &mut TypedSlice<MyIndex, i32> = TypedSlice::try_from_slice_mut(&mut data).unwrap();

    let range: std::ops::Range<MyIndex> = unsafe { MyIndex::from_raw_index_unchecked(5) }..unsafe {
        MyIndex::from_raw_index_unchecked(10)
    };
    let result = slice.get_disjoint_mut([range]);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), GetDisjointMutError::IndexOutOfBounds);
}

#[test]
fn test_as_flattened() {
    use index_type::typed_array::TypedArray;
    let mut data: [TypedArray<MyIndex, i32, 2>; 3] = [
        TypedArray::from_array([1, 2]),
        TypedArray::from_array([3, 4]),
        TypedArray::from_array([5, 6]),
    ];
    let slice: &TypedSlice<MyIndex, TypedArray<MyIndex, i32, 2>> =
        TypedSlice::try_from_slice(&mut data).unwrap();

    let flattened: &TypedSlice<MyIndex, i32> = slice.as_flattened().unwrap();
    assert_eq!(flattened.len_usize(), 6);
    assert_eq!(flattened[MyIndex::ZERO], 1);
    assert_eq!(
        flattened[unsafe { MyIndex::from_raw_index_unchecked(5) }],
        6
    );
}

#[test]
fn test_as_flattened_overflow() {
    use index_type::typed_array::TypedArray;
    #[derive(IndexType, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    struct SmallIndex(u8);

    let data: [TypedArray<SmallIndex, i32, 2>; 128] =
        [const { TypedArray::from_array([0, 0]) }; 128];
    let slice: &TypedSlice<SmallIndex, TypedArray<SmallIndex, i32, 2>> =
        TypedSlice::try_from_slice(&data).unwrap();

    let result: Result<&TypedSlice<SmallIndex, i32>, _> = slice.as_flattened();
    assert!(result.is_err());
}

#[test]
fn test_as_flattened_mut() {
    use index_type::typed_array::TypedArray;
    let mut data: [TypedArray<MyIndex, i32, 2>; 3] = [
        TypedArray::from_array([1, 2]),
        TypedArray::from_array([3, 4]),
        TypedArray::from_array([5, 6]),
    ];
    let slice: &mut TypedSlice<MyIndex, TypedArray<MyIndex, i32, 2>> =
        TypedSlice::try_from_slice_mut(&mut data).unwrap();

    let flattened: &mut TypedSlice<MyIndex, i32> = slice.as_flattened_mut().unwrap();
    flattened[MyIndex::ZERO] = 100;
    assert_eq!(data[0].as_array()[0], 100);
}

#[test]
fn test_starts_with() {
    let mut data = [1, 2, 3, 4, 5];
    let slice: &TypedSlice<MyIndex, i32> = TypedSlice::try_from_slice_mut(&mut data).unwrap();

    let mut prefix = [1, 2];
    let prefix_slice: &TypedSlice<MyIndex, i32> =
        TypedSlice::try_from_slice_mut(&mut prefix).unwrap();
    assert!(slice.starts_with(prefix_slice));

    let mut prefix2 = [2, 3];
    let prefix_slice2: &TypedSlice<MyIndex, i32> =
        TypedSlice::try_from_slice_mut(&mut prefix2).unwrap();
    assert!(!slice.starts_with(prefix_slice2));
}

#[test]
fn test_ends_with() {
    let mut data = [1, 2, 3, 4, 5];
    let slice: &TypedSlice<MyIndex, i32> = TypedSlice::try_from_slice_mut(&mut data).unwrap();

    let mut suffix = [4, 5];
    let suffix_slice: &TypedSlice<MyIndex, i32> =
        TypedSlice::try_from_slice_mut(&mut suffix).unwrap();
    assert!(slice.ends_with(suffix_slice));

    let mut suffix2 = [3, 4];
    let suffix_slice2: &TypedSlice<MyIndex, i32> =
        TypedSlice::try_from_slice_mut(&mut suffix2).unwrap();
    assert!(!slice.ends_with(suffix_slice2));
}

#[test]
fn test_swap_with_slice() {
    let mut data1 = [1, 2, 3];
    let mut data2 = [4, 5, 6];
    let slice1: &mut TypedSlice<MyIndex, i32> = TypedSlice::try_from_slice_mut(&mut data1).unwrap();
    let slice2: &mut TypedSlice<MyIndex, i32> = TypedSlice::try_from_slice_mut(&mut data2).unwrap();

    slice1.swap_with_slice(slice2);
    assert_eq!(data1, [4, 5, 6]);
    assert_eq!(data2, [1, 2, 3]);
}

#[test]
fn test_rotate_left() {
    let mut data = [1, 2, 3, 4, 5];
    let slice: &mut TypedSlice<MyIndex, i32> = TypedSlice::try_from_slice_mut(&mut data).unwrap();

    slice.rotate_left(unsafe { MyIndex::from_raw_index_unchecked(2) });
    assert_eq!(data, [3, 4, 5, 1, 2]);
}

#[test]
fn test_rotate_right() {
    let mut data = [1, 2, 3, 4, 5];
    let slice: &mut TypedSlice<MyIndex, i32> = TypedSlice::try_from_slice_mut(&mut data).unwrap();

    slice.rotate_right(unsafe { MyIndex::from_raw_index_unchecked(2) });
    assert_eq!(data, [4, 5, 1, 2, 3]);
}
