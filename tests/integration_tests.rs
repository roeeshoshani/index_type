use core::num::{NonZeroU8, NonZeroUsize};
use index_type::{
    IndexType, typed_array::TypedArray, typed_array_vec::TypedArrayVec, typed_slice::TypedSlice,
    typed_vec::TypedVec,
};

#[derive(IndexType, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct MyIndex(u32);

#[test]
fn test_nonzero_index() {
    #[derive(IndexType, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    struct NonZeroIndex(NonZeroUsize);

    let mut vec: TypedVec<NonZeroIndex, i32> = TypedVec::new();
    let idx0 = vec.push(100);
    assert_eq!(vec[idx0], 100);
    assert_eq!(idx0.to_raw_index(), 0);
}

#[test]
fn test_binary_search() {
    let mut vec: TypedVec<MyIndex, i32> = TypedVec::new();
    vec.push(10);
    vec.push(20);
    vec.push(30);
    vec.push(40);

    assert_eq!(
        vec.binary_search(&20),
        Ok(unsafe { MyIndex::from_raw_index_unchecked(1) })
    );
    assert_eq!(
        vec.binary_search(&25),
        Err(unsafe { MyIndex::from_raw_index_unchecked(2) })
    );
}

#[test]
fn test_get_disjoint_mut() {
    let mut vec: TypedVec<MyIndex, i32> = TypedVec::new();
    vec.push(10);
    vec.push(20);
    vec.push(30);

    let [a, b] = vec
        .get_disjoint_mut([MyIndex::ZERO, unsafe {
            MyIndex::from_raw_index_unchecked(2)
        }])
        .unwrap();
    *a += 1;
    *b += 1;

    assert_eq!(vec[MyIndex::ZERO], 11);
    assert_eq!(vec[unsafe { MyIndex::from_raw_index_unchecked(2) }], 31);

    // Overlapping indices should fail
    assert!(
        vec.get_disjoint_mut([MyIndex::ZERO, MyIndex::ZERO])
            .is_err()
    );
}

#[test]
fn test_nonzero_capacity_limit() {
    #[derive(IndexType, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    struct NonZeroSmallIndex(NonZeroU8);

    let mut vec: TypedVec<NonZeroSmallIndex, i32> = TypedVec::new();
    // NonZeroU8 MAX is 255. ZERO is 1. MAX_RAW_INDEX is 254.
    // Raw indices are 0..=254.
    for i in 0..254 {
        vec.push(i);
    }
    assert_eq!(vec.len_usize(), 254);
    // When len is 254, push returns raw index 254 and len becomes 255.
    // checked_add_scalar(254, 1) returns Err because 255 > 254.
    assert!(vec.try_push(254).is_err());
}

#[test]
fn test_nonzero_checked_mul_scalar_uses_raw_index_semantics() {
    let idx = NonZeroU8::try_from_raw_index(2).unwrap();
    assert_eq!(idx.checked_mul_scalar(2).unwrap().to_raw_index(), 4);

    let zero = NonZeroU8::ZERO;
    assert_eq!(zero.checked_mul_scalar(10).unwrap().to_raw_index(), 0);

    assert!(NonZeroU8::MAX_INDEX.checked_mul_scalar(2).is_err());
}

#[test]
fn test_macros() {
    use index_type::{typed_array, typed_array_vec, typed_slice, typed_slice_mut, typed_vec};

    // Test typed_array_vec!
    let av: TypedArrayVec<MyIndex, i32, 3> = typed_array_vec![1, 2, 3];
    assert_eq!(av.len_usize(), 3);
    assert_eq!(av[MyIndex::ZERO], 1);

    let av2: TypedArrayVec<MyIndex, i32, 5> = typed_array_vec![0; 5];
    assert_eq!(av2.len_usize(), 5);
    assert_eq!(av2[MyIndex::ZERO], 0);

    // Ensure it works with non-const lengths
    let count = 4;
    let av3: TypedArrayVec<MyIndex, i32, 4> = typed_array_vec![1; count];
    assert_eq!(av3.len_usize(), 4);
    assert_eq!(av3[MyIndex::ZERO], 1);

    // Test typed_vec!
    let v: TypedVec<MyIndex, i32> = typed_vec![1, 2, 3];
    assert_eq!(v.len_usize(), 3);
    assert_eq!(v[MyIndex::ZERO], 1);

    let v2: TypedVec<MyIndex, i32> = typed_vec![0; 5];
    assert_eq!(v2.len_usize(), 5);
    assert_eq!(v2[MyIndex::ZERO], 0);

    // Test typed_array!
    let a: TypedArray<MyIndex, i32, 3> = typed_array![1, 2, 3];
    assert_eq!(a.len_usize(), 3);
    assert_eq!(a[MyIndex::ZERO], 1);

    let a2: TypedArray<MyIndex, i32, 5> = typed_array![0; 5];
    assert_eq!(a2.len_usize(), 5);
    assert_eq!(a2[MyIndex::ZERO], 0);

    // Test typed_slice!
    let s: &TypedSlice<MyIndex, i32> = typed_slice![1, 2, 3];
    assert_eq!(s.len_usize(), 3);
    assert_eq!(s[MyIndex::ZERO], 1);

    // Test typed_slice_mut!
    fn check_typed_slice_mut(s: &mut TypedSlice<MyIndex, i32>) {
        assert_eq!(s.len_usize(), 3);
        assert_eq!(s[MyIndex::ZERO], 1);
        s[MyIndex::ZERO] = 10;
        assert_eq!(s[MyIndex::ZERO], 10);
    }
    check_typed_slice_mut(typed_slice_mut![1, 2, 3]);

    // Basic verify of macro existence and return types
    let _v: TypedVec<MyIndex, i32> = typed_vec![1, 2, 3];
    let _a: TypedArray<MyIndex, i32, 3> = typed_array![1, 2, 3];
    let _av: TypedArrayVec<MyIndex, i32, 3> = typed_array_vec![1, 2, 3];
    let _s: &TypedSlice<MyIndex, i32> = typed_slice![1, 2, 3];
    let _s: &mut TypedSlice<MyIndex, i32> = typed_slice_mut![1, 2, 3];
}
