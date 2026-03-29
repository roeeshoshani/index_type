use std::{
    borrow::{Borrow, BorrowMut},
    cell::Cell,
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
    rc::Rc,
};

use index_type::{
    IndexType,
    typed_array::TypedArray,
    typed_array_vec::{CapacityError, TypedArrayVec},
    typed_slice::TypedSlice,
};

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
fn test_iter_enumerated_supports_reverse_and_mixed_iteration() {
    let mut vec: TypedArrayVec<MyIndex, i32, 4> = TypedArrayVec::new();
    vec.push(10);
    vec.push(20);
    vec.push(30);

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
fn test_into_iter_enumerated_supports_reverse_iteration() {
    let mut vec: TypedArrayVec<MyIndex, i32, 4> = TypedArrayVec::new();
    vec.push(10);
    vec.push(20);
    vec.push(30);

    let collected: Vec<_> = vec
        .into_iter_enumerated()
        .rev()
        .map(|(idx, value)| (idx.to_raw_index(), value))
        .collect();
    assert_eq!(collected, vec![(2, 30), (1, 20), (0, 10)]);
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

#[test]
fn test_cast_index_type_error_drops_elements() {
    #[derive(IndexType, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    struct SmallIndex(u8);

    let drop_counter = Rc::new(Cell::new(0));
    let mut vec: TypedArrayVec<MyIndex, DropCounter<i32>, 300> = TypedArrayVec::new();
    vec.push(DropCounter {
        value: 1,
        drop_counter: drop_counter.clone(),
    });
    vec.push(DropCounter {
        value: 2,
        drop_counter: drop_counter.clone(),
    });

    let result = vec.cast_index_type::<SmallIndex>();
    assert!(result.is_err());
    assert_eq!(drop_counter.get(), 2);
}

#[test]
#[should_panic(expected = "range out of bounds")]
fn test_drain_inclusive_len_panics() {
    let mut vec: TypedArrayVec<MyIndex, i32, 4> = TypedArrayVec::new();
    vec.push(1);
    vec.push(2);
    vec.push(3);

    let len = vec.len();
    let _ = vec.drain(MyIndex::ZERO..=len);
}

#[test]
#[should_panic(expected = "range out of bounds")]
fn test_drain_excluded_len_panics() {
    let mut vec: TypedArrayVec<MyIndex, i32, 4> = TypedArrayVec::new();
    vec.push(1);
    vec.push(2);
    vec.push(3);

    let _ = vec.drain((
        std::ops::Bound::Excluded(MyIndex::MAX_INDEX),
        std::ops::Bound::Unbounded,
    ));
}

#[test]
fn test_traits_iterators_and_capacity_error() {
    let mut vec = TypedArrayVec::<MyIndex, i32, 4>::default();
    assert_eq!(vec.capacity().to_raw_index(), 4);
    assert_eq!(
        vec.indices()
            .map(|idx| idx.to_raw_index())
            .collect::<Vec<_>>(),
        Vec::<usize>::new()
    );

    vec.extend([1, 2, 3]);
    assert_eq!(
        vec.indices()
            .map(|idx| idx.to_raw_index())
            .collect::<Vec<_>>(),
        vec![0, 1, 2]
    );
    assert_eq!(vec.as_ptr(), vec.as_slice().as_ptr());
    assert_eq!(vec.as_mut_ptr(), vec.as_mut_slice().as_mut_ptr());

    let as_ref_slice: &TypedSlice<MyIndex, i32> = vec.as_ref();
    assert_eq!(as_ref_slice.as_slice(), &[1, 2, 3]);
    let borrowed_slice: &TypedSlice<MyIndex, i32> = vec.borrow();
    assert_eq!(borrowed_slice.as_slice(), &[1, 2, 3]);
    let as_mut_slice: &mut TypedSlice<MyIndex, i32> = vec.as_mut();
    as_mut_slice[MyIndex(1)] = 20;
    let borrowed_mut: &mut TypedSlice<MyIndex, i32> = vec.borrow_mut();
    borrowed_mut[MyIndex(2)] = 30;
    assert_eq!(vec.as_slice().as_slice(), &[1, 20, 30]);

    let cloned = vec.clone();
    assert_eq!(format!("{cloned:?}"), "[1, 20, 30]");
    assert_eq!(
        cloned.partial_cmp(&TypedArrayVec::from_iter([1, 20, 31])),
        Some(std::cmp::Ordering::Less)
    );
    let mut hasher = DefaultHasher::new();
    cloned.hash(&mut hasher);
    assert_ne!(hasher.finish(), 0);

    let by_ref: Vec<_> = (&vec).into_iter().copied().collect();
    assert_eq!(by_ref, vec![1, 20, 30]);
    for value in &mut vec {
        *value += 1;
    }
    assert_eq!(vec.as_slice().as_slice(), &[2, 21, 31]);

    let mut into_iter = vec.clone().into_iter();
    assert_eq!(into_iter.len(), 3);
    assert_eq!(into_iter.size_hint(), (3, Some(3)));
    assert_eq!(into_iter.next_back(), Some(31));
    assert_eq!(into_iter.len(), 2);
    assert_eq!(into_iter.next(), Some(2));
    drop(into_iter);

    let enumerated: Vec<_> = vec
        .clone()
        .into_iter_enumerated()
        .map(|(idx, value)| (idx.to_raw_index(), value))
        .collect();
    assert_eq!(enumerated, vec![(0, 2), (1, 21), (2, 31)]);

    let mut uninit = TypedArrayVec::<MyIndex, i32, 4>::new();
    unsafe {
        let ptr = uninit.as_mut_ptr();
        ptr.write(7);
        ptr.add(1).write(8);
        uninit.set_len(MyIndex(2));
    }
    assert_eq!(uninit.as_slice().as_slice(), &[7, 8]);

    let err = CapacityError::new("x");
    assert_eq!(err.to_string(), "insufficient capacity");
    fn assert_error<E: std::error::Error>(_err: &E) {}
    assert_error(&err);
}

#[test]
fn test_drain_len_and_partial_drop_behavior() {
    let mut vec = TypedArrayVec::<MyIndex, i32, 6>::from_iter([1, 2, 3, 4, 5, 6]);
    {
        let mut drain = vec.drain(MyIndex(1)..MyIndex(5));
        assert_eq!(drain.len(), 4);
        assert_eq!(drain.size_hint(), (4, Some(4)));
        assert_eq!(drain.next(), Some(2));
        assert_eq!(drain.next_back(), Some(5));
        assert_eq!(drain.len(), 2);
    }
    assert_eq!(vec.as_slice().as_slice(), &[1, 6]);
}
