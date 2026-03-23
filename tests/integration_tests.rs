use index_type::IndexType;
use index_type::typed_vec::TypedVec;
use index_type::typed_array::TypedArray;
use index_type::typed_slice::TypedSlice;
use core::num::NonZeroUsize;

#[derive(IndexType, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct MyIndex(u32);

#[test]
fn test_typed_vec_basic() {
    let mut vec: TypedVec<MyIndex, i32> = TypedVec::new();
    assert!(vec.is_empty());
    assert_eq!(vec.len_usize(), 0);

    let idx0 = vec.push(10).unwrap();
    let idx1 = vec.push(20).unwrap();
    let idx2 = vec.push(30).unwrap();

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
fn test_typed_vec_capacity_limit() {
    #[derive(IndexType, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    struct SmallIndex(u8);

    let mut vec: TypedVec<SmallIndex, i32> = TypedVec::new();
    // u8 MAX is 255.
    // MAX_RAW_INDEX for u8 is 255.
    for i in 0..255 {
        vec.push(i as i32).unwrap();
    }
    assert_eq!(vec.len_usize(), 255);
    // When len is 255, push returns index 255 and len becomes 256.
    // However, checked_add_scalar(255, 1) will fail if MAX_RAW_INDEX is 255.
    // Actually, u8's MAX_RAW_INDEX is 255.
    // checked_add_scalar(255, 1) returns Err because 256 > 255.
    assert!(vec.push(255).is_err());
}

#[test]
fn test_typed_array_basic() {
    let arr: TypedArray<MyIndex, i32, 3> = TypedArray::try_from_array([1, 2, 3]).unwrap();
    assert_eq!(arr.len_usize(), 3);
    assert_eq!(arr[MyIndex::ZERO], 1);
    assert_eq!(arr[unsafe { MyIndex::from_raw_index_unchecked(1) }], 2);
    assert_eq!(arr[unsafe { MyIndex::from_raw_index_unchecked(2) }], 3);
}

#[test]
fn test_typed_slice_basic() {
    let mut data = [10, 20, 30, 40, 50];
    let slice = TypedSlice::<MyIndex, i32>::try_from_slice_mut(&mut data).unwrap();

    assert_eq!(slice.len_usize(), 5);
    assert_eq!(slice[MyIndex::ZERO], 10);

    let sub_slice = &slice[unsafe { MyIndex::from_raw_index_unchecked(1) }..unsafe { MyIndex::from_raw_index_unchecked(4) }];
    assert_eq!(sub_slice.len_usize(), 3);
    assert_eq!(sub_slice[MyIndex::ZERO], 20);
}

#[test]
fn test_nonzero_index() {
    #[derive(IndexType, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    struct NonZeroIndex(NonZeroUsize);

    let mut vec: TypedVec<NonZeroIndex, i32> = TypedVec::new();
    let idx0 = vec.push(100).unwrap();
    assert_eq!(vec[idx0], 100);
    assert_eq!(idx0.to_raw_index(), 0);
}

#[test]
fn test_binary_search() {
    let mut vec: TypedVec<MyIndex, i32> = TypedVec::new();
    vec.push(10).unwrap();
    vec.push(20).unwrap();
    vec.push(30).unwrap();
    vec.push(40).unwrap();

    assert_eq!(vec.binary_search(&20), Ok(unsafe { MyIndex::from_raw_index_unchecked(1) }));
    assert_eq!(vec.binary_search(&25), Err(unsafe { MyIndex::from_raw_index_unchecked(2) }));
}

#[test]
fn test_get_disjoint_mut() {
    let mut vec: TypedVec<MyIndex, i32> = TypedVec::new();
    vec.push(10).unwrap();
    vec.push(20).unwrap();
    vec.push(30).unwrap();

    let [a, b] = vec.get_disjoint_mut([MyIndex::ZERO, unsafe { MyIndex::from_raw_index_unchecked(2) }]).unwrap();
    *a += 1;
    *b += 1;

    assert_eq!(vec[MyIndex::ZERO], 11);
    assert_eq!(vec[unsafe { MyIndex::from_raw_index_unchecked(2) }], 31);

    // Overlapping indices should fail
    assert!(vec.get_disjoint_mut([MyIndex::ZERO, MyIndex::ZERO]).is_err());
}

#[test]
fn test_nonzero_capacity_limit() {
    use core::num::NonZeroU8;
    #[derive(IndexType, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    struct NonZeroSmallIndex(NonZeroU8);

    let mut vec: TypedVec<NonZeroSmallIndex, i32> = TypedVec::new();
    // NonZeroU8 MAX is 255. ZERO is 1. MAX_RAW_INDEX is 254.
    // Raw indices are 0..=254.
    for i in 0..254 {
        vec.push(i as i32).unwrap();
    }
    assert_eq!(vec.len_usize(), 254);
    // When len is 254, push returns raw index 254 and len becomes 255.
    // checked_add_scalar(254, 1) returns Err because 255 > 254.
    assert!(vec.push(254).is_err());
}

#[test]
fn test_macros() {
    use index_type::{typed_vec, typed_array, typed_slice, typed_slice_mut};

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
    let _s: &TypedSlice<MyIndex, i32> = typed_slice![1, 2, 3];
}
