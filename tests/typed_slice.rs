use index_type::IndexType;
use index_type::typed_slice::TypedSlice;

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
