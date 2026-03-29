use std::{
    borrow::{Borrow, BorrowMut},
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};

use index_type::{IndexType, typed_array::TypedArray, typed_slice::TypedSlice};

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
fn test_iter_enumerated_supports_reverse_iteration() {
    let arr: TypedArray<MyIndex, i32, 3> = TypedArray::try_from_array([10, 20, 30]).unwrap();

    let collected: Vec<_> = arr
        .iter_enumerated()
        .rev()
        .map(|(idx, value)| (idx.to_raw_index(), *value))
        .collect();
    assert_eq!(collected, vec![(2, 30), (1, 20), (0, 10)]);
}

#[test]
fn test_into_iter_enumerated_supports_mixed_iteration() {
    let arr: TypedArray<MyIndex, i32, 3> = TypedArray::try_from_array([10, 20, 30]).unwrap();
    let mut iter = arr.into_iter_enumerated();

    assert_eq!(iter.next(), Some((MyIndex(0), 10)));
    assert_eq!(iter.next_back(), Some((MyIndex(2), 30)));
    assert_eq!(iter.next(), Some((MyIndex(1), 20)));
    assert_eq!(iter.next_back(), None);
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

#[test]
fn test_helper_methods_and_traits() {
    #[derive(IndexType, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    struct SmallIndex(u8);

    let mut raw = [1, 2, 3];
    let mut arr = TypedArray::<MyIndex, i32, 3>::from_array(raw);

    assert_eq!(arr.len().to_raw_index(), 3);
    assert_eq!(
        arr.indices()
            .map(|idx| idx.to_raw_index())
            .collect::<Vec<_>>(),
        vec![0, 1, 2]
    );
    assert_eq!(arr.as_slice().as_slice(), &[1, 2, 3]);
    assert_eq!(arr.as_array(), &[1, 2, 3]);

    arr.as_mut_array()[1] = 20;
    arr.as_mut_slice()[MyIndex(2)] = 30;
    assert_eq!(arr.as_slice().as_slice(), &[1, 20, 30]);

    let refs = arr.each_ref();
    assert_eq!(*refs[MyIndex::ZERO], 1);
    assert_eq!(*refs[MyIndex(2)], 30);

    let mut refs_mut = arr.each_mut();
    **refs_mut.get_mut(MyIndex(1)).unwrap() += 2;
    assert_eq!(arr.as_slice().as_slice(), &[1, 22, 30]);

    let mapped = arr.map(|value| value.to_string());
    assert_eq!(
        mapped.into_array(),
        ["1".to_string(), "22".to_string(), "30".to_string()]
    );

    let arr_ref = TypedArray::<MyIndex, i32, 3>::try_from_array_ref(&raw).unwrap();
    assert_eq!(arr_ref[MyIndex(1)], 2);

    {
        let arr_mut = TypedArray::<MyIndex, i32, 3>::try_from_array_mut(&mut raw).unwrap();
        arr_mut[MyIndex(1)] = 99;
    }
    assert_eq!(raw, [1, 99, 3]);

    let arr_from_ref = TypedArray::<MyIndex, i32, 3>::from_array_ref(&raw);
    assert_eq!(arr_from_ref[MyIndex(1)], 99);
    let arr_from_mut = TypedArray::<MyIndex, i32, 3>::from_array_mut(&mut raw);
    arr_from_mut[MyIndex(2)] = 42;
    assert_eq!(raw, [1, 99, 42]);

    assert!(TypedArray::<SmallIndex, i32, 300>::try_from_array([0; 300]).is_err());
    assert!(TypedArray::<SmallIndex, i32, 300>::try_from_array_ref(&[0; 300]).is_err());

    let mut too_large = [0; 300];
    assert!(TypedArray::<SmallIndex, i32, 300>::try_from_array_mut(&mut too_large).is_err());

    let smaller = TypedArray::<MyIndex, i32, 3>::from_array([1, 2, 3]);
    let larger = TypedArray::<MyIndex, i32, 3>::from_array([1, 2, 4]);
    assert!(smaller < larger);
    assert_eq!(smaller.partial_cmp(&larger), Some(std::cmp::Ordering::Less));
    assert_eq!(format!("{smaller:?}"), "[1, 2, 3]");

    let mut array_from_slice_storage = [7, 8, 9];
    let typed_slice =
        TypedSlice::<MyIndex, i32>::try_from_slice_mut(&mut array_from_slice_storage).unwrap();
    let array_ref = <&TypedArray<MyIndex, i32, 3>>::try_from(&*typed_slice).unwrap();
    assert_eq!(array_ref[MyIndex(1)], 8);
    let array_mut = <&mut TypedArray<MyIndex, i32, 3>>::try_from(typed_slice).unwrap();
    array_mut[MyIndex(2)] = 90;
    assert_eq!(array_from_slice_storage, [7, 8, 90]);

    let copied = TypedArray::<MyIndex, i32, 3>::try_from(
        TypedSlice::<MyIndex, i32>::try_from_slice(&array_from_slice_storage).unwrap(),
    )
    .unwrap();
    assert_eq!(copied.into_array(), [7, 8, 90]);

    let as_ref_slice: &TypedSlice<MyIndex, i32> = smaller.as_ref();
    assert_eq!(as_ref_slice.as_slice(), &[1, 2, 3]);
    let borrowed_slice: &TypedSlice<MyIndex, i32> = smaller.borrow();
    assert_eq!(borrowed_slice.as_slice(), &[1, 2, 3]);
    let mut borrowed_mut_source = TypedArray::<MyIndex, i32, 3>::from_array([4, 5, 6]);
    let borrowed_mut: &mut TypedSlice<MyIndex, i32> = borrowed_mut_source.borrow_mut();
    borrowed_mut[MyIndex::ZERO] = 40;
    let as_mut_slice: &mut TypedSlice<MyIndex, i32> = borrowed_mut_source.as_mut();
    as_mut_slice[MyIndex(1)] = 50;
    assert_eq!(borrowed_mut_source.into_array(), [40, 50, 6]);

    let by_ref: Vec<_> = (&smaller).into_iter().copied().collect();
    assert_eq!(by_ref, vec![1, 2, 3]);
    let mut iter_mut_source = TypedArray::<MyIndex, i32, 3>::from_array([1, 2, 3]);
    for value in &mut iter_mut_source {
        *value *= 2;
    }
    assert_eq!(iter_mut_source.into_array(), [2, 4, 6]);
    let by_value: Vec<_> = TypedArray::<MyIndex, i32, 3>::from_array([1, 2, 3])
        .into_iter()
        .collect();
    assert_eq!(by_value, vec![1, 2, 3]);

    let mut defaulted = TypedArray::<MyIndex, i32, 3>::default();
    defaulted.clone_from(&smaller);
    assert_eq!(defaulted, smaller);
    let cloned = smaller.clone();
    assert_eq!(cloned, smaller);
    let mut hasher = DefaultHasher::new();
    cloned.hash(&mut hasher);
    assert_ne!(hasher.finish(), 0);
}
