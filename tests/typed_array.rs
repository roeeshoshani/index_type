use index_type::IndexType;
use index_type::typed_array::TypedArray;

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
