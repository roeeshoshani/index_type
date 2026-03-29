use index_type::{IndexType, typed_enumerate::TypedIteratorExt, typed_vec::TypedVec};

#[derive(IndexType, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct MyIndex(u32);

#[test]
fn test_typed_vec_basic() {
    let mut vec: TypedVec<MyIndex, i32> = TypedVec::new();
    assert!(vec.is_empty());
    assert_eq!(vec.len_usize(), 0);

    let idx0 = vec.push(10);
    let idx1 = vec.push(20);
    let idx2 = vec.push(30);

    assert_eq!(vec.len_usize(), 3);
    assert_eq!(vec[idx0], 10);
    assert_eq!(vec[idx1], 20);
    assert_eq!(vec[idx2], 30);

    vec[idx1] = 25;
    assert_eq!(vec[idx1], 25);

    assert_eq!(vec.pop(), Some(30));
    assert_eq!(vec.len_usize(), 2);
}

#[test]
fn test_iter_enumerated_supports_reverse_and_mixed_iteration() {
    let vec: TypedVec<MyIndex, i32> = TypedVec::from_vec(vec![10, 20, 30]);

    let reversed: Vec<_> = vec
        .iter_enumerated()
        .rev()
        .map(|(idx, value)| (idx.to_raw_index(), *value))
        .collect();
    assert_eq!(reversed, vec![(2, 30), (1, 20), (0, 10)]);

    let mut iter = vec.iter_enumerated();
    assert_eq!(
        iter.next().map(|(idx, value)| (idx.to_raw_index(), *value)),
        Some((0, 10))
    );
    assert_eq!(
        iter.next_back()
            .map(|(idx, value)| (idx.to_raw_index(), *value)),
        Some((2, 30))
    );
    assert_eq!(
        iter.next().map(|(idx, value)| (idx.to_raw_index(), *value)),
        Some((1, 20))
    );
    assert_eq!(iter.next_back(), None);
}

#[test]
fn test_into_iter_enumerated_preserves_indices_in_reverse() {
    let vec: TypedVec<MyIndex, i32> = TypedVec::from_vec(vec![10, 20, 30]);

    let collected: Vec<_> = vec
        .into_iter_enumerated()
        .rev()
        .map(|(idx, value)| (idx.to_raw_index(), value))
        .collect();
    assert_eq!(collected, vec![(2, 30), (1, 20), (0, 10)]);
}

#[test]
fn test_typed_enumerate_on_slice_iterator() {
    let vec: TypedVec<MyIndex, i32> = TypedVec::from_vec(vec![10, 20, 30]);

    let collected: Vec<_> = vec
        .iter()
        .typed_enumerate::<MyIndex>()
        .map(|(idx, value)| (idx.to_raw_index(), *value))
        .collect();
    assert_eq!(collected, vec![(0, 10), (1, 20), (2, 30)]);
}

#[test]
fn test_typed_vec_capacity_limit() {
    #[derive(IndexType, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    struct SmallIndex(u8);

    let mut vec: TypedVec<SmallIndex, i32> = TypedVec::new();
    for i in 0..255 {
        vec.push(i as i32);
    }
    assert_eq!(vec.len_usize(), 255);
    assert!(vec.try_push(255).is_err());
}

#[test]
fn test_try_from_vec_too_big() {
    #[derive(IndexType, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    struct SmallIndex(u8);

    let vec = (0..256).map(|i| i as i32).collect::<Vec<_>>();
    let result: Result<TypedVec<SmallIndex, i32>, _> = TypedVec::try_from_vec(vec);
    assert!(result.is_err());

    let vec = (0..255).map(|i| i as i32).collect::<Vec<_>>();
    let result: Result<TypedVec<SmallIndex, i32>, _> = TypedVec::try_from_vec(vec);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().len_usize(), 255);
}

#[test]
fn test_try_from_vec_max_value() {
    #[derive(IndexType, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    struct SmallIndex(u8);

    let vec = (0..256).map(|i| i as i32).collect::<Vec<_>>();
    let result: Result<TypedVec<SmallIndex, i32>, _> = TypedVec::try_from_vec(vec);
    assert!(result.is_err());
}

#[test]
fn test_from_vec_panic() {
    #[derive(IndexType, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    struct SmallIndex(u8);

    let vec = (0..256).map(|i| i as i32).collect::<Vec<_>>();
    let result = std::panic::catch_unwind(|| TypedVec::<SmallIndex, i32>::from_vec(vec));
    assert!(result.is_err());
}

#[test]
fn test_try_push_overflow() {
    #[derive(IndexType, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    struct SmallIndex(u8);

    let mut vec: TypedVec<SmallIndex, i32> = TypedVec::new();
    vec.push(1);
    vec.push(2);
    assert_eq!(vec.len_usize(), 2);

    let result = vec.try_push(3);
    assert!(result.is_ok());
    assert_eq!(vec.len_usize(), 3);

    for i in 3..255 {
        vec.push(i as i32);
    }
    assert_eq!(vec.len_usize(), 255);

    assert!(vec.try_push(255).is_err());
    assert_eq!(vec.len_usize(), 255);
}

#[test]
fn test_push_panic_overflow() {
    #[derive(IndexType, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    struct SmallIndex(u8);

    let mut vec: TypedVec<SmallIndex, i32> = TypedVec::new();
    for i in 0..255 {
        vec.push(i as i32);
    }

    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        vec.push(255);
    }));
    assert!(result.is_err());
}

#[test]
fn test_try_append() {
    let mut vec1: TypedVec<MyIndex, i32> = TypedVec::new();
    vec1.push(1);
    vec1.push(2);

    let mut vec2: TypedVec<MyIndex, i32> = TypedVec::new();
    vec2.push(3);
    vec2.push(4);
    vec2.push(5);

    vec1.try_append(&mut vec2).unwrap();
    assert_eq!(vec1.len_usize(), 5);
    assert_eq!(vec1[MyIndex::ZERO], 1);
    assert_eq!(vec1[unsafe { MyIndex::from_raw_index_unchecked(4) }], 5);
    assert!(vec2.is_empty());
}

#[test]
fn test_try_append_overflow() {
    #[derive(IndexType, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    struct SmallIndex(u8);

    let mut vec1: TypedVec<SmallIndex, i32> = TypedVec::new();
    for i in 0..200 {
        vec1.push(i as i32);
    }

    let mut vec2: TypedVec<SmallIndex, i32> = TypedVec::new();
    for i in 0..100 {
        vec2.push(i as i32);
    }

    let result = vec1.try_append(&mut vec2);
    assert!(result.is_err());
    assert_eq!(vec1.len_usize(), 200);
}

#[test]
fn test_append_overflow_panic() {
    #[derive(IndexType, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    struct SmallIndex(u8);

    let mut vec1: TypedVec<SmallIndex, i32> = TypedVec::new();
    for i in 0..200 {
        vec1.push(i as i32);
    }

    let mut vec2: TypedVec<SmallIndex, i32> = TypedVec::new();
    for i in 0..100 {
        vec2.push(i as i32);
    }

    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        vec1.append(&mut vec2);
    }));
    assert!(result.is_err());
}

#[test]
fn test_try_insert() {
    let mut vec: TypedVec<MyIndex, i32> = TypedVec::new();
    vec.push(1);
    vec.push(3);

    vec.try_insert(unsafe { MyIndex::from_raw_index_unchecked(1) }, 2)
        .unwrap();
    assert_eq!(vec.len_usize(), 3);
    assert_eq!(vec[unsafe { MyIndex::from_raw_index_unchecked(0) }], 1);
    assert_eq!(vec[unsafe { MyIndex::from_raw_index_unchecked(1) }], 2);
    assert_eq!(vec[unsafe { MyIndex::from_raw_index_unchecked(2) }], 3);
}

#[test]
fn test_try_insert_overflow() {
    #[derive(IndexType, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    struct SmallIndex(u8);

    let mut vec: TypedVec<SmallIndex, i32> = TypedVec::new();
    for i in 0..255 {
        vec.push(i as i32);
    }

    let result = vec.try_insert(unsafe { SmallIndex::from_raw_index_unchecked(255) }, 999);
    assert!(result.is_err());
    assert_eq!(vec.len_usize(), 255);
}

#[test]
fn test_insert_overflow_panic() {
    #[derive(IndexType, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    struct SmallIndex(u8);

    let mut vec: TypedVec<SmallIndex, i32> = TypedVec::new();
    for i in 0..255 {
        vec.push(i as i32);
    }

    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        vec.insert(unsafe { SmallIndex::from_raw_index_unchecked(255) }, 999);
    }));
    assert!(result.is_err());
}

#[test]
fn test_try_insert_out_of_bounds() {
    let mut vec: TypedVec<MyIndex, i32> = TypedVec::new();
    vec.push(1);
    vec.push(2);

    let res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        vec.insert(unsafe { MyIndex::from_raw_index_unchecked(5) }, 3);
    }));
    assert!(res.is_err());
}

#[test]
fn test_try_extend() {
    let mut vec: TypedVec<MyIndex, i32> = TypedVec::new();
    vec.push(1);
    vec.push(2);

    vec.try_extend(vec![3, 4, 5]).unwrap();
    assert_eq!(vec.len_usize(), 5);
}

#[test]
fn test_try_extend_overflow() {
    #[derive(IndexType, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    struct SmallIndex(u8);

    let mut vec: TypedVec<SmallIndex, i32> = TypedVec::new();
    for i in 0..200 {
        vec.push(i as i32);
    }

    let result = vec.try_extend(0..100);
    assert!(result.is_err());
    assert_eq!(vec.len_usize(), 200);
    assert_eq!(vec[SmallIndex::ZERO], 0);
    assert_eq!(
        vec[unsafe { SmallIndex::from_raw_index_unchecked(199) }],
        199
    );
}

#[test]
fn test_extend_overflow_panic() {
    #[derive(IndexType, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    struct SmallIndex(u8);

    let mut vec: TypedVec<SmallIndex, i32> = TypedVec::new();
    for i in 0..200 {
        vec.push(i as i32);
    }

    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        vec.extend(0..100);
    }));
    assert!(result.is_err());
}

#[test]
fn test_extend_from_slice() {
    let mut vec: TypedVec<MyIndex, i32> = TypedVec::new();
    vec.push(1);
    vec.push(2);

    let mut data = [3, 4, 5];
    let slice =
        index_type::typed_slice::TypedSlice::<MyIndex, i32>::try_from_slice_mut(&mut data).unwrap();
    vec.extend_from_slice(slice);
    assert_eq!(vec.len_usize(), 5);
}

#[test]
fn test_extend_from_slice_overflow_panics_without_mutating_vec() {
    #[derive(IndexType, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    struct SmallIndex(u8);

    let mut vec: TypedVec<SmallIndex, i32> = TypedVec::new();
    for i in 0..200 {
        vec.push(i);
    }

    let data = [1; 100];
    let slice =
        index_type::typed_slice::TypedSlice::<SmallIndex, i32>::try_from_slice(&data).unwrap();

    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        vec.extend_from_slice(slice);
    }));
    assert!(result.is_err());
    assert_eq!(vec.len_usize(), 200);
    assert_eq!(vec[SmallIndex::ZERO], 0);
    assert_eq!(
        vec[unsafe { SmallIndex::from_raw_index_unchecked(199) }],
        199
    );
}

#[test]
fn test_try_extend_from_slice_overflow_leaves_vec_unchanged() {
    #[derive(IndexType, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    struct SmallIndex(u8);

    let mut vec: TypedVec<SmallIndex, i32> = TypedVec::new();
    for i in 0..200 {
        vec.push(i);
    }

    let data = [1; 100];
    let slice =
        index_type::typed_slice::TypedSlice::<SmallIndex, i32>::try_from_slice(&data).unwrap();

    let result = vec.try_extend_from_slice(slice);
    assert!(result.is_err());
    assert_eq!(vec.len_usize(), 200);
    assert_eq!(vec[SmallIndex::ZERO], 0);
    assert_eq!(
        vec[unsafe { SmallIndex::from_raw_index_unchecked(199) }],
        199
    );
}

#[test]
fn test_try_into_flattened() {
    let mut vec: TypedVec<MyIndex, [i32; 2]> = TypedVec::new();
    vec.push([1, 2]);
    vec.push([3, 4]);
    vec.push([5, 6]);

    let flattened: TypedVec<MyIndex, i32> = vec.try_into_flattened().unwrap();
    assert_eq!(flattened.len_usize(), 6);
    assert_eq!(flattened[MyIndex::ZERO], 1);
    assert_eq!(
        flattened[unsafe { MyIndex::from_raw_index_unchecked(1) }],
        2
    );
    assert_eq!(
        flattened[unsafe { MyIndex::from_raw_index_unchecked(2) }],
        3
    );
}

#[test]
fn test_try_into_flattened_overflow() {
    #[derive(IndexType, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    struct SmallIndex(u8);

    let mut vec: TypedVec<SmallIndex, [i32; 2]> = TypedVec::new();
    for i in 0..128 {
        vec.push([i as i32, 0]);
    }

    let result = vec.try_into_flattened();
    assert!(result.is_err());
}

#[test]
fn test_into_flattened_panic() {
    #[derive(IndexType, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    struct SmallIndex(u8);

    let mut vec: TypedVec<SmallIndex, [i32; 2]> = TypedVec::new();
    for i in 0..128 {
        vec.push([i as i32, 0]);
    }

    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        vec.into_flattened();
    }));
    assert!(result.is_err());
}

#[test]
fn test_split_off() {
    let mut vec: TypedVec<MyIndex, i32> = TypedVec::new();
    vec.push(1);
    vec.push(2);
    vec.push(3);
    vec.push(4);
    vec.push(5);

    let rest = vec.split_off(unsafe { MyIndex::from_raw_index_unchecked(2) });
    assert_eq!(vec.len_usize(), 2);
    assert_eq!(rest.len_usize(), 3);
    assert_eq!(vec[MyIndex::ZERO], 1);
    assert_eq!(rest[MyIndex::ZERO], 3);
}

#[test]
fn test_truncate() {
    let mut vec: TypedVec<MyIndex, i32> = TypedVec::new();
    vec.push(1);
    vec.push(2);
    vec.push(3);
    vec.push(4);
    vec.push(5);

    vec.truncate(unsafe { MyIndex::from_raw_index_unchecked(3) });
    assert_eq!(vec.len_usize(), 3);
    assert_eq!(vec[MyIndex::ZERO], 1);
    assert_eq!(vec[unsafe { MyIndex::from_raw_index_unchecked(2) }], 3);
}

#[test]
fn test_drain() {
    let mut vec: TypedVec<MyIndex, i32> = TypedVec::new();
    vec.push(1);
    vec.push(2);
    vec.push(3);
    vec.push(4);
    vec.push(5);

    let drain: Vec<i32> = vec
        .drain(MyIndex::ZERO..unsafe { MyIndex::from_raw_index_unchecked(2) })
        .collect();
    assert_eq!(drain, vec![1, 2]);
    assert_eq!(vec.len_usize(), 3);
    assert_eq!(vec[MyIndex::ZERO], 3);
}

#[test]
fn test_splice() {
    let mut vec: TypedVec<MyIndex, i32> = TypedVec::new();
    vec.push(1);
    vec.push(2);
    vec.push(3);

    let removed: Vec<i32> = vec
        .splice(
            MyIndex::ZERO..unsafe { MyIndex::from_raw_index_unchecked(1) },
            vec![10, 20],
        )
        .collect();
    assert_eq!(removed, vec![1]);
    assert_eq!(vec.len_usize(), 4);
    assert_eq!(vec[MyIndex::ZERO], 10);
}

#[test]
fn test_leak_preserves_typed_slice_api() {
    let data: TypedVec<MyIndex, i32> = TypedVec::from_vec(vec![10, 20, 30]);
    let cap = data.capacity();
    let leaked: &'static mut index_type::typed_slice::TypedSlice<MyIndex, i32> = data.leak();
    leaked[MyIndex::ZERO] = 99;
    assert_eq!(leaked[MyIndex::ZERO], 99);
    assert_eq!(leaked.len_usize(), 3);

    // Avoid actually leaking the memory
    let _ =
        unsafe { TypedVec::<MyIndex, i32>::from_raw_parts(leaked.as_mut_ptr(), leaked.len(), cap) };
}

#[test]
#[should_panic(expected = "range out of bounds")]
fn test_extend_from_within_excluded_len_panics() {
    let mut vec: TypedVec<MyIndex, i32> = TypedVec::new();
    vec.push(1);
    vec.push(2);
    vec.push(3);

    vec.extend_from_within((
        std::ops::Bound::Excluded(MyIndex::MAX_INDEX),
        std::ops::Bound::Unbounded,
    ));
}

#[test]
fn test_extend_from_within_overflow_panics_without_mutating_vec() {
    #[derive(IndexType, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    struct SmallIndex(u8);

    let mut vec: TypedVec<SmallIndex, i32> = TypedVec::new();
    for i in 0..200 {
        vec.push(i);
    }

    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        vec.extend_from_within(
            SmallIndex::ZERO..unsafe { SmallIndex::from_raw_index_unchecked(100) },
        );
    }));
    assert!(result.is_err());
    assert_eq!(vec.len_usize(), 200);
    assert_eq!(vec[SmallIndex::ZERO], 0);
    assert_eq!(
        vec[unsafe { SmallIndex::from_raw_index_unchecked(199) }],
        199
    );
}

#[test]
fn test_extend_from_within_inclusive_single_element_overflow_panics_without_mutating_vec() {
    #[derive(IndexType, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    struct SmallIndex(u8);

    let mut vec: TypedVec<SmallIndex, i32> = TypedVec::new();
    for i in 0..255 {
        vec.push(i);
    }

    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        vec.extend_from_within(
            unsafe { SmallIndex::from_raw_index_unchecked(254) }..=unsafe {
                SmallIndex::from_raw_index_unchecked(254)
            },
        );
    }));
    assert!(result.is_err());
    assert_eq!(vec.len_usize(), 255);
    assert_eq!(vec[SmallIndex::ZERO], 0);
    assert_eq!(
        vec[unsafe { SmallIndex::from_raw_index_unchecked(254) }],
        254
    );
}

#[test]
fn test_splice_overflow_panics_without_exceeding_index_bounds() {
    #[derive(IndexType, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    struct SmallIndex(u8);

    let mut vec: TypedVec<SmallIndex, i32> = TypedVec::new();
    for i in 0..200 {
        vec.push(i);
    }

    let replacement = 0..100;
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let _ = vec.splice(
            SmallIndex::ZERO..unsafe { SmallIndex::from_raw_index_unchecked(10) },
            replacement,
        );
    }));
    assert!(result.is_err());
    assert_eq!(vec.len_usize(), 255);
}

#[test]
fn test_splice_inclusive_single_element_overflow_panics_without_exceeding_index_bounds() {
    #[derive(IndexType, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    struct SmallIndex(u8);

    let mut vec: TypedVec<SmallIndex, i32> = TypedVec::new();
    for i in 0..255 {
        vec.push(i);
    }

    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let _ = vec.splice(
            unsafe { SmallIndex::from_raw_index_unchecked(254) }..=unsafe {
                SmallIndex::from_raw_index_unchecked(254)
            },
            [999, 1000],
        );
    }));
    assert!(result.is_err());
    assert_eq!(vec.len_usize(), 255);
}

#[test]
fn test_resize() {
    let mut vec: TypedVec<MyIndex, i32> = TypedVec::new();
    vec.push(1);
    vec.push(2);

    vec.resize(unsafe { MyIndex::from_raw_index_unchecked(4) }, 9);
    assert_eq!(vec.len_usize(), 4);
    assert_eq!(vec[MyIndex::ZERO], 1);
    assert_eq!(vec[unsafe { MyIndex::from_raw_index_unchecked(3) }], 9);
}

#[test]
fn test_resize_with() {
    let mut vec: TypedVec<MyIndex, i32> = TypedVec::new();
    vec.push(1);

    vec.resize_with(unsafe { MyIndex::from_raw_index_unchecked(3) }, || 99);
    assert_eq!(vec.len_usize(), 3);
    assert_eq!(vec[MyIndex::ZERO], 1);
    assert_eq!(vec[unsafe { MyIndex::from_raw_index_unchecked(2) }], 99);
}

#[test]
fn test_len() {
    let mut vec: TypedVec<MyIndex, i32> = TypedVec::new();
    assert_eq!(vec.len(), MyIndex::ZERO);

    vec.push(1);
    assert_eq!(vec.len(), unsafe { MyIndex::from_raw_index_unchecked(1) });

    vec.push(2);
    assert_eq!(vec.len(), unsafe { MyIndex::from_raw_index_unchecked(2) });
}

#[test]
fn test_try_from_raw_parts() {
    let (ptr, len, cap) = vec![1, 2, 3, 4, 5].into_raw_parts();
    let vec = unsafe { TypedVec::<MyIndex, i32>::try_from_raw_parts(ptr, len, cap).unwrap() };
    assert_eq!(vec.len_usize(), 5);
}

#[test]
fn test_from_raw_parts() {
    let (ptr, len, cap) = vec![1, 2, 3, 4, 5].into_raw_parts();
    let len = unsafe { MyIndex::from_raw_index_unchecked(len) };
    let vec = unsafe { TypedVec::<MyIndex, i32>::from_raw_parts(ptr, len, cap) };
    assert_eq!(vec.len(), len);
}

#[test]
fn test_try_from_raw_parts_overflow() {
    #[derive(IndexType, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    struct SmallIndex(u8);

    let (ptr, len, cap) = vec![1; 256].into_raw_parts();

    let result: Result<TypedVec<SmallIndex, i32>, _> =
        unsafe { TypedVec::try_from_raw_parts(ptr, 256, cap) };
    assert!(result.is_err());

    // Avoid leaking the vec
    let _ = unsafe { Vec::from_raw_parts(ptr, len, cap) };
}

#[test]
fn test_cast_index_type_upcast() {
    #[derive(IndexType, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    struct SmallIndex(u8);

    let vec: TypedVec<SmallIndex, i32> = TypedVec::from_vec(vec![1, 2, 3, 4, 5]);

    let cast: TypedVec<MyIndex, i32> = vec.cast_index_type::<MyIndex>().unwrap();
    assert_eq!(cast.len_usize(), 5);
    assert_eq!(cast[MyIndex::ZERO], 1);
}

#[test]
fn test_cast_index_type_downcast() {
    #[derive(IndexType, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    struct SmallIndex(u8);

    let vec: TypedVec<MyIndex, i32> = TypedVec::from_vec(vec![1, 2, 3, 4, 5]);

    let result = vec.cast_index_type::<SmallIndex>();
    assert!(result.is_ok());
}

#[test]
fn test_cast_index_type_downcast_fails() {
    #[derive(IndexType, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    struct SmallIndex(u8);

    let vec: TypedVec<MyIndex, i32> = TypedVec::from_vec(vec![1; 300]);

    let result = vec.cast_index_type::<SmallIndex>();
    assert!(result.is_err());
}

#[test]
fn test_cast_index_type_same() {
    let vec: TypedVec<MyIndex, i32> = TypedVec::from_vec(vec![1, 2, 3, 4, 5]);

    let cast: TypedVec<MyIndex, i32> = vec.cast_index_type::<MyIndex>().unwrap();
    assert_eq!(cast.len_usize(), 5);
}

#[test]
fn test_extend_via_trait_impl() {
    let mut vec: TypedVec<MyIndex, i32> = TypedVec::new();
    Extend::extend(&mut vec, [10, 20, 30]);
    assert_eq!(vec.as_slice().as_slice(), &[10, 20, 30]);
}

#[test]
fn test_into_iter_does_not_require_clone() {
    #[derive(Debug, PartialEq, Eq)]
    struct NotClone(i32);

    let vec: TypedVec<MyIndex, NotClone> =
        TypedVec::from_vec(vec![NotClone(1), NotClone(2), NotClone(3)]);
    let collected: Vec<_> = vec.into_iter().map(|v| v.0).collect();
    assert_eq!(collected, vec![1, 2, 3]);
}
