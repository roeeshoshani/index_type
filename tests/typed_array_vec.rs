use std::{cell::Cell, rc::Rc};

use index_type::{IndexType, typed_array::TypedArray, typed_array_vec::TypedArrayVec};

#[derive(IndexType, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct MyIndex(u32);

#[derive(Debug, Clone)]
struct DropCounter<T> {
    value: T,
    drop_counter: Rc<Cell<usize>>,
}
impl<T> Drop for DropCounter<T> {
    fn drop(&mut self) {
        self.drop_counter.update(|x| x + 1);
    }
}

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
fn test_into_iter_panic() {
    let drop_counter = Rc::new(Cell::new(0));
    let mut vec: TypedArrayVec<MyIndex, DropCounter<i32>, 8> = TypedArrayVec::new();
    for i in 1..=5 {
        vec.push(DropCounter {
            value: i,
            drop_counter: Rc::clone(&drop_counter),
        });
    }
    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let mut iter = vec.into_iter();
        assert_eq!(iter.next().unwrap().value, 1);
        assert_eq!(iter.next().unwrap().value, 2);
        panic!("panic during iteration");
    }));
    assert_eq!(drop_counter.get(), 5);
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
fn test_retain_panic() {
    let drop_counter = Rc::new(Cell::new(0));
    let mut vec: TypedArrayVec<MyIndex, DropCounter<i32>, 8> = TypedArrayVec::new();
    for i in 1..=5 {
        vec.push(DropCounter {
            value: i,
            drop_counter: Rc::clone(&drop_counter),
        });
    }
    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        vec.retain(|e| {
            if e.value == 3 {
                panic!("panic during retain");
            }
            e.value % 2 == 0
        });
    }));
    drop(vec);
    assert_eq!(drop_counter.get(), 5);
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
fn test_drain_back_iteration() {
    let mut vec: TypedArrayVec<MyIndex, i32, 8> = TypedArrayVec::new();
    vec.push(1);
    vec.push(2);
    vec.push(3);
    vec.push(4);
    vec.push(5);
    vec.push(6);
    vec.push(7);
    vec.push(8);
    {
        let mut drain = vec.drain(MyIndex(1)..MyIndex(6));
        assert_eq!(drain.next(), Some(2));
        assert_eq!(drain.next_back(), Some(6));
        assert_eq!(drain.next(), Some(3));
        assert_eq!(drain.next_back(), Some(5));
        assert_eq!(drain.next_back(), Some(4));
        assert_eq!(drain.next(), None);
        assert_eq!(drain.next_back(), None);
    }
    assert_eq!(vec.as_slice().as_slice(), &[1, 7, 8]);
}

#[test]
fn test_drain_panic() {
    let drop_counter = Rc::new(Cell::new(0));
    let mut vec: TypedArrayVec<MyIndex, DropCounter<i32>, 8> = TypedArrayVec::new();
    vec.push(DropCounter {
        value: 1,
        drop_counter: Rc::clone(&drop_counter),
    });
    vec.push(DropCounter {
        value: 2,
        drop_counter: Rc::clone(&drop_counter),
    });
    vec.push(DropCounter {
        value: 3,
        drop_counter: Rc::clone(&drop_counter),
    });
    vec.push(DropCounter {
        value: 4,
        drop_counter: Rc::clone(&drop_counter),
    });
    vec.push(DropCounter {
        value: 5,
        drop_counter: Rc::clone(&drop_counter),
    });
    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let mut drain = vec.drain(MyIndex(1)..MyIndex(4));
        assert_eq!(drain.next().unwrap().value, 2);
        assert_eq!(drain.next_back().unwrap().value, 4);
        panic!("panic while holding drain");
    }));
    assert_eq!(drop_counter.get(), 3);
    assert_eq!(vec.len().to_raw_index(), 2);
    assert_eq!(vec[MyIndex(0)].value, 1);
    assert_eq!(vec[MyIndex(1)].value, 5);
    drop(vec);
    assert_eq!(drop_counter.get(), 5);
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

#[test]
fn test_cast_index_type_upcast() {
    #[derive(IndexType, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    struct SmallIndex(u8);

    let mut vec: TypedArrayVec<SmallIndex, i32, 16> = TypedArrayVec::new();
    vec.push(1);
    vec.push(2);
    vec.push(3);
    vec.push(4);
    vec.push(5);

    let cast: TypedArrayVec<MyIndex, i32, 16> = vec.cast_index_type::<MyIndex>().unwrap();
    assert_eq!(cast.len().to_raw_index(), 5);
    assert_eq!(cast[MyIndex::ZERO], 1);
}

#[test]
fn test_cast_index_type_downcast() {
    #[derive(IndexType, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    struct SmallIndex(u8);

    let mut vec: TypedArrayVec<MyIndex, i32, 16> = TypedArrayVec::new();
    vec.push(1);
    vec.push(2);
    vec.push(3);
    vec.push(4);
    vec.push(5);

    let result = vec.cast_index_type::<SmallIndex>();
    assert!(result.is_ok());
}

#[test]
fn test_cast_index_type_same() {
    let mut vec: TypedArrayVec<MyIndex, i32, 16> = TypedArrayVec::new();
    vec.push(1);
    vec.push(2);
    vec.push(3);
    vec.push(4);
    vec.push(5);

    let cast: TypedArrayVec<MyIndex, i32, 16> = vec.cast_index_type::<MyIndex>().unwrap();
    assert_eq!(cast.len().to_raw_index(), 5);
}
