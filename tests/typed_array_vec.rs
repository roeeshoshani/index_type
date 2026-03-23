use index_type::IndexType;
use index_type::typed_array::TypedArray;
use index_type::typed_array_vec::TypedArrayVec;

#[derive(IndexType, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct MyIndex(u32);

#[test]
fn test_new() {
    let vec: TypedArrayVec<MyIndex, i32, 16> = TypedArrayVec::new();
    assert_eq!(vec.len().to_raw_index(), 0);
    assert_eq!(vec.capacity().to_raw_index(), 16);
    assert!(vec.is_empty());
    assert!(!vec.is_full());
}

#[test]
fn test_push_pop() {
    let mut vec: TypedArrayVec<MyIndex, i32, 2> = TypedArrayVec::new();
    vec.push(1);
    vec.push(2);
    assert_eq!(vec.len().to_raw_index(), 2);
    assert!(vec.is_full());

    assert_eq!(vec.pop(), Some(2));
    assert_eq!(vec.pop(), Some(1));
    assert_eq!(vec.pop(), None);
    assert!(vec.is_empty());
}

#[test]
fn test_push_error() {
    let mut vec: TypedArrayVec<MyIndex, i32, 1> = TypedArrayVec::new();
    vec.try_push(1).unwrap();
    let err = vec.try_push(2).unwrap_err();
    assert_eq!(err.element(), 2);
}

#[test]
fn test_insert_remove() {
    let mut vec: TypedArrayVec<MyIndex, i32, 4> = TypedArrayVec::new();
    vec.push(1);
    vec.push(3);
    vec.insert(MyIndex(1), 2);
    assert_eq!(vec.as_slice().as_slice(), &[1, 2, 3]);

    assert_eq!(vec.remove(MyIndex(1)), 2);
    assert_eq!(vec.as_slice().as_slice(), &[1, 3]);
}

#[test]
fn test_swap_remove() {
    let mut vec: TypedArrayVec<MyIndex, i32, 4> = TypedArrayVec::new();
    vec.push(1);
    vec.push(2);
    vec.push(3);
    assert_eq!(vec.swap_remove(MyIndex(0)), 1);
    assert_eq!(vec.as_slice().as_slice(), &[3, 2]);
}

#[test]
fn test_truncate_clear() {
    let mut vec: TypedArrayVec<MyIndex, i32, 4> = TypedArrayVec::new();
    vec.push(1);
    vec.push(2);
    vec.push(3);
    vec.truncate(MyIndex(1));
    assert_eq!(vec.len().to_raw_index(), 1);
    assert_eq!(vec[MyIndex(0)], 1);

    vec.clear();
    assert!(vec.is_empty());
}

#[test]
fn test_into_iter() {
    let mut vec: TypedArrayVec<MyIndex, i32, 4> = TypedArrayVec::new();
    vec.push(1);
    vec.push(2);
    let collected: Vec<i32> = vec.into_iter().collect();
    assert_eq!(collected, vec![1, 2]);
}

#[test]
fn test_from_array() {
    let array = TypedArray::<MyIndex, i32, 3>::from_array([1, 2, 3]);
    let vec = TypedArrayVec::<MyIndex, i32, 3>::from(array);
    assert_eq!(vec.as_slice().as_slice(), &[1, 2, 3]);
}

#[test]
fn test_drop() {
    use core::cell::Cell;
    let counter = Cell::new(0);
    {
        #[derive(Debug)]
        struct DropCounter<'a>(&'a Cell<usize>);
        impl Drop for DropCounter<'_> {
            fn drop(&mut self) {
                self.0.set(self.0.get() + 1);
            }
        }
        let mut vec: TypedArrayVec<MyIndex, DropCounter, 4> = TypedArrayVec::new();
        vec.push(DropCounter(&counter));
        vec.push(DropCounter(&counter));
        vec.pop();
        assert_eq!(counter.get(), 1);
    }
    assert_eq!(counter.get(), 2);
}

#[test]
fn test_retain() {
    let mut vec: TypedArrayVec<MyIndex, i32, 4> = TypedArrayVec::new();
    vec.push(1);
    vec.push(2);
    vec.push(3);
    vec.push(4);
    vec.retain(|&x| x % 2 == 0);
    assert_eq!(vec.as_slice().as_slice(), &[2, 4]);
}

#[test]
fn test_drain() {
    let mut vec: TypedArrayVec<MyIndex, i32, 4> = TypedArrayVec::new();
    vec.push(1);
    vec.push(2);
    vec.push(3);
    vec.push(4);
    {
        let mut drain = vec.drain(MyIndex(1)..MyIndex(3));
        assert_eq!(drain.next(), Some(2));
        assert_eq!(drain.next(), Some(3));
        assert_eq!(drain.next(), None);
    }
    assert_eq!(vec.as_slice().as_slice(), &[1, 4]);
}

#[test]
fn test_extend_from_slice() {
    let mut vec: TypedArrayVec<MyIndex, i32, 4> = TypedArrayVec::new();
    vec.push(1);
    let other = index_type::typed_array![2, 3];
    vec.extend_from_slice(&other);
    assert_eq!(vec.as_slice().as_slice(), &[1, 2, 3]);
}

#[test]
fn test_extend_from_slice_copy() {
    let mut vec: TypedArrayVec<MyIndex, i32, 4> = TypedArrayVec::new();
    vec.push(1);
    let other = index_type::typed_array![2, 3];
    vec.extend_from_slice_copy(&other);
    assert_eq!(vec.as_slice().as_slice(), &[1, 2, 3]);
}

#[test]
fn test_remaining_capacity() {
    let mut vec: TypedArrayVec<MyIndex, i32, 4> = TypedArrayVec::new();
    assert_eq!(vec.remaining_capacity().to_raw_index(), 4);
    vec.push(1);
    assert_eq!(vec.remaining_capacity().to_raw_index(), 3);
}
