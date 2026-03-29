use index_type::{
    IndexType,
    typed_slice::{GetDisjointMutError, TypedSlice},
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
fn test_iter_enumerated_supports_reverse_iteration() {
    let data = [10, 20, 30];
    let slice: &TypedSlice<MyIndex, i32> = TypedSlice::try_from_slice(&data).unwrap();

    let collected: Vec<_> = slice
        .iter_enumerated()
        .rev()
        .map(|(idx, value)| (idx.to_raw_index(), *value))
        .collect();
    assert_eq!(collected, vec![(2, 30), (1, 20), (0, 10)]);
}

#[test]
fn test_iter_mut_enumerated_supports_mixed_iteration() {
    let mut data = [10, 20, 30];
    let slice: &mut TypedSlice<MyIndex, i32> = TypedSlice::try_from_slice_mut(&mut data).unwrap();

    let mut iter = slice.iter_mut_enumerated();
    let (front_idx, front_value) = iter.next().unwrap();
    assert_eq!(front_idx.to_raw_index(), 0);
    *front_value += 1;

    let (back_idx, back_value) = iter.next_back().unwrap();
    assert_eq!(back_idx.to_raw_index(), 2);
    *back_value += 2;

    let (middle_idx, middle_value) = iter.next().unwrap();
    assert_eq!(middle_idx.to_raw_index(), 1);
    *middle_value += 3;

    assert!(iter.next_back().is_none());
    assert_eq!(data, [11, 23, 32]);
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
    let data = [1, 2];
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

#[test]
fn test_accessors_index_forms_and_mutation_helpers() {
    let mut data = [10, 20, 30, 40, 50];
    let slice = TypedSlice::<MyIndex, i32>::try_from_slice_mut(&mut data).unwrap();

    assert_eq!(slice.first(), Some(&10));
    assert_eq!(slice.last(), Some(&50));
    *slice.first_mut().unwrap() += 1;
    *slice.last_mut().unwrap() += 2;
    assert_eq!(slice.as_slice(), &[11, 20, 30, 40, 52]);

    let (first, rest) = slice.split_first().unwrap();
    assert_eq!(*first, 11);
    assert_eq!(rest.as_slice(), &[20, 30, 40, 52]);
    let (last, rest) = slice.split_last().unwrap();
    assert_eq!(*last, 52);
    assert_eq!(rest.as_slice(), &[11, 20, 30, 40]);

    {
        let (first, rest) = slice.split_first_mut().unwrap();
        *first = 100;
        rest[MyIndex::ZERO] = 200;
    }
    {
        let (last, rest) = slice.split_last_mut().unwrap();
        *last = 500;
        rest[MyIndex(2)] = 400;
    }
    assert_eq!(slice.as_slice(), &[100, 200, 400, 40, 500]);

    assert_eq!(slice.get(MyIndex(2)), Some(&400));
    *slice.get_mut(MyIndex(2)).unwrap() = 300;
    assert_eq!(slice[MyIndex(2)], 300);
    assert_eq!(slice.get(MyIndex(99)), None);
    assert_eq!(
        slice.get(MyIndex(1)..MyIndex(4)).unwrap().as_slice(),
        &[200, 300, 40]
    );
    assert_eq!(slice.get(..MyIndex(2)).unwrap().as_slice(), &[100, 200]);
    assert_eq!(slice.get(MyIndex(3)..).unwrap().as_slice(), &[40, 500]);
    assert_eq!(slice.get(..=MyIndex(1)).unwrap().as_slice(), &[100, 200]);
    assert_eq!(slice.get(..).unwrap().as_slice(), &[100, 200, 300, 40, 500]);

    unsafe {
        assert_eq!(slice.get_unchecked(MyIndex(4)), &500);
        *slice.get_unchecked_mut(MyIndex(0)) = 111;
    }
    assert_eq!(slice.as_slice(), &[111, 200, 300, 40, 500]);

    assert_eq!(
        slice.first_chunk::<2>().unwrap().as_slice().as_slice(),
        &[111, 200]
    );
    assert_eq!(
        slice.last_chunk::<2>().unwrap().as_slice().as_slice(),
        &[40, 500]
    );
    let (chunk, rest) = slice.split_first_chunk::<2>().unwrap();
    assert_eq!(chunk.as_slice().as_slice(), &[111, 200]);
    assert_eq!(rest.as_slice(), &[300, 40, 500]);
    let (rest, chunk) = slice.split_last_chunk::<2>().unwrap();
    assert_eq!(rest.as_slice(), &[111, 200, 300]);
    assert_eq!(chunk.as_slice().as_slice(), &[40, 500]);
    assert_eq!(
        slice.as_array::<5>().unwrap().as_slice().as_slice(),
        &[111, 200, 300, 40, 500]
    );
    slice.as_mut_array::<5>().unwrap()[MyIndex(1)] = 222;
    assert_eq!(slice.as_slice(), &[111, 222, 300, 40, 500]);

    let ptr = slice.as_ptr();
    let ptr_range = slice.as_ptr_range();
    assert_eq!(ptr, ptr_range.start);
    let mut_ptr = slice.as_mut_ptr();
    let mut_ptr_range = slice.as_mut_ptr_range();
    assert_eq!(mut_ptr, mut_ptr_range.start);

    let (left, right) = slice.split_at(MyIndex(2));
    assert_eq!(left.as_slice(), &[111, 222]);
    assert_eq!(right.as_slice(), &[300, 40, 500]);
    let (left, right) = slice.split_at_checked(MyIndex(2)).unwrap();
    assert_eq!(left.as_slice(), &[111, 222]);
    assert_eq!(right.as_slice(), &[300, 40, 500]);

    {
        let (left, right) = slice.split_at_mut(MyIndex(3));
        left[MyIndex(1)] += 1;
        right[MyIndex::ZERO] += 1;
    }
    unsafe {
        let (left, right) = slice.split_at_unchecked(MyIndex(1));
        assert_eq!(left.as_slice(), &[111]);
        assert_eq!(right.as_slice(), &[223, 300, 41, 500]);

        let (left, right) = slice.split_at_mut_unchecked(MyIndex(4));
        left[MyIndex(3)] += 10;
        right[MyIndex::ZERO] += 10;
    }
    assert_eq!(slice.as_slice(), &[111, 223, 300, 51, 510]);
}

#[test]
fn test_index_operator_variants_cover_range_impls() {
    let mut data = [10, 20, 30, 40, 50];
    let slice = TypedSlice::<MyIndex, i32>::try_from_slice_mut(&mut data).unwrap();

    assert_eq!(slice[MyIndex::ZERO], 10);
    assert_eq!((&slice[..]).as_slice(), &[10, 20, 30, 40, 50]);
    assert_eq!((&slice[..MyIndex(2)]).as_slice(), &[10, 20]);
    assert_eq!((&slice[..=MyIndex(2)]).as_slice(), &[10, 20, 30]);
    assert_eq!((&slice[MyIndex(2)..]).as_slice(), &[30, 40, 50]);
    assert_eq!((&slice[MyIndex(1)..MyIndex(4)]).as_slice(), &[20, 30, 40]);
    assert_eq!((&slice[MyIndex(1)..=MyIndex(3)]).as_slice(), &[20, 30, 40]);

    slice[MyIndex(1)..MyIndex(4)][MyIndex(1)] = 300;
    slice[..=MyIndex(1)][MyIndex::ZERO] = 11;
    slice[MyIndex(3)..][MyIndex::ZERO] = 400;
    slice[..][MyIndex(4)] = 55;
    assert_eq!(slice.as_slice(), &[11, 20, 300, 400, 55]);
}

#[test]
fn test_range_get_variants_cover_unchecked_paths() {
    let mut data = [1, 2, 3, 4, 5];
    let slice = TypedSlice::<MyIndex, i32>::try_from_slice_mut(&mut data).unwrap();

    assert_eq!(slice.get(..=MyIndex(2)).unwrap().as_slice(), &[1, 2, 3]);
    assert_eq!(slice.get(MyIndex(2)..).unwrap().as_slice(), &[3, 4, 5]);

    slice.get_mut(..MyIndex(2)).unwrap()[MyIndex::ZERO] = 10;
    slice.get_mut(MyIndex(1)..=MyIndex(3)).unwrap()[MyIndex(1)] = 30;
    slice.get_mut(MyIndex(4)..).unwrap()[MyIndex::ZERO] = 50;
    slice.get_mut(..).unwrap()[MyIndex(1)] = 20;

    unsafe {
        assert_eq!(slice.get_unchecked(..MyIndex(2)).as_slice(), &[10, 20]);
        assert_eq!(slice.get_unchecked(..=MyIndex(2)).as_slice(), &[10, 20, 30]);
        assert_eq!(slice.get_unchecked(MyIndex(2)..).as_slice(), &[30, 4, 50]);
        assert_eq!(
            slice.get_unchecked(MyIndex(1)..=MyIndex(3)).as_slice(),
            &[20, 30, 4]
        );

        slice.get_unchecked_mut(..MyIndex(2))[MyIndex(1)] = 21;
        slice.get_unchecked_mut(..=MyIndex(2))[MyIndex(2)] = 31;
        slice.get_unchecked_mut(MyIndex(3)..)[MyIndex::ZERO] = 40;
        slice.get_unchecked_mut(MyIndex(1)..=MyIndex(3))[MyIndex::ZERO] = 22;
        slice.get_unchecked_mut(..)[MyIndex(4)] = 55;
    }

    assert_eq!(slice.as_slice(), &[10, 22, 31, 40, 55]);
}

#[test]
fn test_chunk_split_sort_and_copy_apis() {
    let mut data = [1, 2, 0, 3, 0, 4];
    let slice = TypedSlice::<MyIndex, i32>::try_from_slice_mut(&mut data).unwrap();

    assert!(slice.contains(&3));

    let windows: Vec<Vec<_>> = slice
        .windows(3)
        .map(|chunk| chunk.as_slice().to_vec())
        .collect();
    assert_eq!(
        windows,
        vec![vec![1, 2, 0], vec![2, 0, 3], vec![0, 3, 0], vec![3, 0, 4]]
    );

    let chunks: Vec<Vec<_>> = slice
        .chunks(2)
        .map(|chunk| chunk.as_slice().to_vec())
        .collect();
    assert_eq!(chunks, vec![vec![1, 2], vec![0, 3], vec![0, 4]]);

    for chunk in slice.chunks_mut(2) {
        chunk.reverse();
    }
    assert_eq!(slice.as_slice(), &[2, 1, 3, 0, 4, 0]);

    let rchunks: Vec<Vec<_>> = slice
        .rchunks(2)
        .map(|chunk| chunk.as_slice().to_vec())
        .collect();
    assert_eq!(rchunks, vec![vec![4, 0], vec![3, 0], vec![2, 1]]);

    for chunk in slice.rchunks_mut(2) {
        chunk.fill(9);
    }
    assert_eq!(slice.as_slice(), &[9, 9, 9, 9, 9, 9]);

    slice.fill_with({
        let mut next = 0;
        move || {
            let current = next;
            next += 1;
            current
        }
    });
    slice.swap(MyIndex::ZERO, MyIndex(5));
    slice.copy_within(MyIndex(1)..MyIndex(3), MyIndex(3));
    assert_eq!(slice.as_slice(), &[5, 1, 2, 1, 2, 0]);
    slice.reverse();
    assert_eq!(slice.as_slice(), &[0, 2, 1, 2, 1, 5]);
    slice.sort_unstable();
    assert_eq!(slice.as_slice(), &[0, 1, 1, 2, 2, 5]);
    slice.sort_unstable_by(|a, b| b.cmp(a));
    assert_eq!(slice.as_slice(), &[5, 2, 2, 1, 1, 0]);
    slice.sort_unstable_by_key(|value| -*value);
    assert_eq!(slice.as_slice(), &[5, 2, 2, 1, 1, 0]);
    assert!(!slice.is_sorted());
    assert!(slice.is_sorted_by(|a, b| a >= b));
    assert!(slice.is_sorted_by_key(|value| -*value));
    assert_eq!(slice.partition_point(|value| *value >= 2), MyIndex(3));

    let (lower, pivot, upper) = slice.select_nth_unstable_by(MyIndex(2), |a, b| a.cmp(b));
    assert_eq!(*pivot, 1);
    assert!(lower.iter().all(|value| *value <= *pivot));
    assert!(upper.iter().all(|value| *value >= *pivot));
    let (lower, pivot, upper) = slice.select_nth_unstable_by_key(MyIndex(4), |value| *value);
    assert_eq!(*pivot, 2);
    assert!(lower.iter().all(|value| *value <= *pivot));
    assert!(upper.iter().all(|value| *value >= *pivot));

    let mut chunks_data = [1, 2, 3, 4, 5];
    let chunks_slice = TypedSlice::<MyIndex, i32>::try_from_slice_mut(&mut chunks_data).unwrap();
    let (pairs, rest) = chunks_slice.as_chunks::<2>();
    assert_eq!(pairs.len_usize(), 2);
    assert_eq!(pairs[MyIndex::ZERO].as_slice().as_slice(), &[1, 2]);
    assert_eq!(rest.as_slice(), &[5]);
    let (rest, pairs) = chunks_slice.as_rchunks::<2>();
    assert_eq!(rest.as_slice(), &[1]);
    assert_eq!(pairs[MyIndex::ZERO].as_slice().as_slice(), &[2, 3]);
    let (pairs, rest) = chunks_slice.as_chunks_mut::<2>();
    pairs[MyIndex::ZERO][MyIndex::ZERO] = 10;
    rest[MyIndex::ZERO] = 50;
    let (rest, pairs) = chunks_slice.as_rchunks_mut::<2>();
    rest[MyIndex::ZERO] = 11;
    pairs[MyIndex::ZERO][MyIndex(1)] = 33;
    assert_eq!(chunks_slice.as_slice(), &[11, 2, 33, 4, 50]);

    let exact_chunks: Vec<Vec<_>> = chunks_slice
        .chunks_exact(2)
        .map(|chunk| chunk.as_slice().to_vec())
        .collect();
    assert_eq!(exact_chunks, vec![vec![11, 2], vec![33, 4]]);
    for chunk in chunks_slice.chunks_exact_mut(2) {
        chunk.fill(7);
    }
    assert_eq!(chunks_slice.as_slice(), &[7, 7, 7, 7, 50]);
    let rchunks_exact: Vec<Vec<_>> = chunks_slice
        .rchunks_exact(2)
        .map(|chunk| chunk.as_slice().to_vec())
        .collect();
    assert_eq!(rchunks_exact, vec![vec![7, 50], vec![7, 7]]);
    for chunk in chunks_slice.rchunks_exact_mut(2) {
        chunk.fill(8);
    }
    assert_eq!(chunks_slice.as_slice(), &[7, 8, 8, 8, 8]);

    let split_groups: Vec<Vec<_>> = chunks_slice
        .split(|value| *value == 8)
        .map(|part| part.as_slice().to_vec())
        .collect();
    assert_eq!(split_groups, vec![vec![7], vec![], vec![], vec![], vec![]]);
    for part in chunks_slice.split_mut(|value| *value == 7) {
        part.fill(1);
    }
    assert_eq!(chunks_slice.as_slice(), &[7, 1, 1, 1, 1]);

    let inclusive: Vec<Vec<_>> = chunks_slice
        .split_inclusive(|value| *value == 1)
        .map(|part| part.as_slice().to_vec())
        .collect();
    assert_eq!(inclusive, vec![vec![7, 1], vec![1], vec![1], vec![1]]);
    for part in chunks_slice.split_inclusive_mut(|value| *value == 7) {
        part[MyIndex::ZERO] += 1;
    }
    assert_eq!(chunks_slice.as_slice(), &[8, 2, 1, 1, 1]);

    let splitn: Vec<Vec<_>> = chunks_slice
        .splitn(2, |value| *value == 1)
        .map(|part| part.as_slice().to_vec())
        .collect();
    assert_eq!(splitn, vec![vec![8, 2], vec![1, 1]]);
    for part in chunks_slice.splitn_mut(2, |value| *value == 8) {
        if !part.is_empty() {
            part[MyIndex::ZERO] += 10;
        }
    }
    assert_eq!(chunks_slice.as_slice(), &[8, 12, 1, 1, 1]);

    let rsplitn: Vec<Vec<_>> = chunks_slice
        .rsplitn(2, |value| *value == 1)
        .map(|part| part.as_slice().to_vec())
        .collect();
    assert_eq!(rsplitn, vec![vec![], vec![8, 12, 1, 1]]);
    for part in chunks_slice.rsplitn_mut(2, |value| *value == 11) {
        if !part.is_empty() {
            part[MyIndex::ZERO] += 1;
        }
    }
    assert_eq!(chunks_slice.as_slice(), &[9, 12, 1, 1, 1]);

    let rsplit: Vec<Vec<_>> = chunks_slice
        .rsplit(|value| *value == 2)
        .map(|part| part.as_slice().to_vec())
        .collect();
    assert_eq!(rsplit, vec![vec![9, 12, 1, 1, 1]]);
    for part in chunks_slice.rsplit_mut(|value| *value == 8) {
        if !part.is_empty() {
            part[MyIndex::ZERO] += 1;
        }
    }
    assert_eq!(chunks_slice.as_slice(), &[10, 12, 1, 1, 1]);

    let chunked: Vec<Vec<_>> = chunks_slice
        .chunk_by(|a, b| (*a % 2) == (*b % 2))
        .map(|part| part.as_slice().to_vec())
        .collect();
    assert_eq!(chunked, vec![vec![10, 12], vec![1, 1, 1]]);
    for part in chunks_slice.chunk_by_mut(|a, b| (*a % 2) == (*b % 2)) {
        part.reverse();
    }
    assert_eq!(chunks_slice.as_slice(), &[12, 10, 1, 1, 1]);

    let source = TypedSlice::<MyIndex, i32>::try_from_slice(&[4, 5, 6, 7, 8]).unwrap();
    chunks_slice.clone_from_slice(source);
    assert_eq!(chunks_slice.as_slice(), &[4, 5, 6, 7, 8]);
    let copy_source = TypedSlice::<MyIndex, i32>::try_from_slice(&[1, 2, 3, 4, 5]).unwrap();
    chunks_slice.copy_from_slice(copy_source);
    assert_eq!(chunks_slice.as_slice(), &[1, 2, 3, 4, 5]);

    let mut chunked_exact = [1, 2, 3, 4];
    let exact_slice = TypedSlice::<MyIndex, i32>::try_from_slice_mut(&mut chunked_exact).unwrap();
    let chunks = unsafe { exact_slice.as_chunks_unchecked::<2>() };
    assert_eq!(chunks.len_usize(), 2);
    assert_eq!(chunks[MyIndex::ZERO].as_slice().as_slice(), &[1, 2]);
    let chunks = unsafe { exact_slice.as_chunks_unchecked_mut::<2>() };
    chunks[MyIndex(1)][MyIndex::ZERO] = 9;
    assert_eq!(exact_slice.as_slice(), &[1, 2, 9, 4]);
}
