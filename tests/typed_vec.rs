use index_type::{typed_vec::TypedVec, IndexType};

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
