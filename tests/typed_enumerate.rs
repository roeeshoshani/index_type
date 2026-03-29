use index_type::{IndexType, typed_enumerate::TypedIteratorExt};

#[derive(IndexType, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct SmallIndex(u8);

#[test]
fn test_adapter_contracts() {
    let mut iter = [10, 20, 30].into_iter().typed_enumerate::<SmallIndex>();
    assert_eq!(iter.size_hint(), (3, Some(3)));
    assert_eq!(iter.len(), 3);
    assert_eq!(
        iter.next_back()
            .map(|(idx, value)| (idx.to_raw_index(), value)),
        Some((2, 30))
    );
    assert_eq!(iter.len(), 2);
    assert_eq!(
        iter.next().map(|(idx, value)| (idx.to_raw_index(), value)),
        Some((0, 10))
    );
    assert_eq!(
        iter.next().map(|(idx, value)| (idx.to_raw_index(), value)),
        Some((1, 20))
    );
    assert_eq!(iter.next(), None);
    assert_eq!(iter.next_back(), None);
}
