use core::num::NonZeroUsize;
use index_type::{IndexType, typed_range_iter::TypedRangeIterExt};

#[derive(IndexType, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct MyIndex(u32);

#[derive(IndexType, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct SmallIndex(u8);

mod typed_range_iter {
    use super::*;

    #[test]
    fn test_normal_range() {
        let iter = (MyIndex(0)..MyIndex(5)).iter();
        let collected: Vec<_> = iter.collect();
        assert_eq!(
            collected,
            vec![MyIndex(0), MyIndex(1), MyIndex(2), MyIndex(3), MyIndex(4)]
        );
    }

    #[test]
    fn test_empty_range_equal() {
        let iter = (MyIndex(5)..MyIndex(5)).iter();
        let collected: Vec<_> = iter.collect();
        assert!(collected.is_empty());
    }

    #[test]
    fn test_empty_range_start_greater() {
        let iter = (MyIndex(5)..MyIndex(3)).iter();
        let collected: Vec<_> = iter.collect();
        assert!(collected.is_empty());
    }

    #[test]
    fn test_single_element_range() {
        let iter = (MyIndex(3)..MyIndex(4)).iter();
        let collected: Vec<_> = iter.collect();
        assert_eq!(collected, vec![MyIndex(3)]);
    }

    #[test]
    fn test_len() {
        assert_eq!((MyIndex(0)..MyIndex(5)).iter().len(), 5);
        assert_eq!((MyIndex(5)..MyIndex(5)).iter().len(), 0);
        assert_eq!((MyIndex(5)..MyIndex(3)).iter().len(), 0);
        assert_eq!((MyIndex(0)..MyIndex(1)).iter().len(), 1);
    }

    #[test]
    fn test_count() {
        assert_eq!((MyIndex(0)..MyIndex(5)).iter().count(), 5);
        assert_eq!((MyIndex(5)..MyIndex(5)).iter().count(), 0);
    }

    #[test]
    fn test_nth() {
        let mut iter = (MyIndex(0)..MyIndex(10)).iter();
        assert_eq!(iter.nth(0), Some(MyIndex(0)));
        assert_eq!(iter.nth(2), Some(MyIndex(3)));
        assert_eq!(iter.nth(100), None);
    }

    #[test]
    fn test_last() {
        assert_eq!((MyIndex(0)..MyIndex(5)).iter().last(), Some(MyIndex(4)));
        assert_eq!((MyIndex(5)..MyIndex(5)).iter().last(), None);
    }

    #[test]
    fn test_size_hint() {
        let iter = (MyIndex(0)..MyIndex(5)).iter();
        assert_eq!(iter.size_hint(), (5, Some(5)));
    }

    #[test]
    fn test_double_ended() {
        let iter = (MyIndex(0)..MyIndex(5)).iter();
        let collected: Vec<_> = iter.rev().collect();
        assert_eq!(
            collected,
            vec![MyIndex(4), MyIndex(3), MyIndex(2), MyIndex(1), MyIndex(0)]
        );
    }

    #[test]
    fn test_double_ended_empty() {
        let iter = (MyIndex(5)..MyIndex(5)).iter();
        let collected: Vec<_> = iter.rev().collect();
        assert!(collected.is_empty());
    }

    #[test]
    fn test_nth_back() {
        let mut iter = (MyIndex(0)..MyIndex(10)).iter();
        assert_eq!(iter.nth_back(0), Some(MyIndex(9)));
        assert_eq!(iter.nth_back(2), Some(MyIndex(6)));
        assert_eq!(iter.nth_back(100), None);
    }

    #[test]
    fn test_exact_size_iterator() {
        let iter = (MyIndex(0)..MyIndex(5)).iter();
        assert_eq!(iter.len(), 5);
    }

    #[test]
    fn test_fused_iterator() {
        let mut iter = (MyIndex(0)..MyIndex(3)).iter();
        assert_eq!(iter.next(), Some(MyIndex(0)));
        assert_eq!(iter.next(), Some(MyIndex(1)));
        assert_eq!(iter.next(), Some(MyIndex(2)));
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_is_sorted() {
        assert!((MyIndex(0)..MyIndex(5)).iter().is_sorted());
    }

    #[test]
    fn test_min_max() {
        assert_eq!((MyIndex(0)..MyIndex(5)).iter().min(), Some(MyIndex(0)));
        assert_eq!((MyIndex(0)..MyIndex(5)).iter().max(), Some(MyIndex(4)));
        assert_eq!((MyIndex(5)..MyIndex(5)).iter().min(), None);
        assert_eq!((MyIndex(5)..MyIndex(5)).iter().max(), None);
    }
}

mod typed_range_from_iter {
    use super::*;

    #[test]
    fn test_range_from() {
        let iter = (MyIndex(0)..).iter();
        let collected: Vec<_> = iter.take(5).collect();
        assert_eq!(
            collected,
            vec![MyIndex(0), MyIndex(1), MyIndex(2), MyIndex(3), MyIndex(4)]
        );
    }

    #[test]
    fn test_nth() {
        let mut iter = (MyIndex(10)..).iter();
        assert_eq!(iter.nth(0), Some(MyIndex(10)));
        assert_eq!(iter.nth(5), Some(MyIndex(16)));
    }

    #[test]
    fn test_min() {
        let iter = (MyIndex(5)..).iter();
        assert_eq!(iter.min(), Some(MyIndex(5)));
    }

    #[test]
    fn test_is_sorted() {
        assert!((MyIndex(0)..).iter().is_sorted());
    }

    #[test]
    fn test_fused_iterator() {
        let mut iter = (MyIndex(0)..).iter();
        assert_eq!(iter.next(), Some(MyIndex(0)));
        assert_eq!(iter.next(), Some(MyIndex(1)));
    }
}

mod typed_range_inclusive_iter {
    use super::*;

    #[test]
    fn test_normal_range() {
        let iter = (MyIndex(0)..=MyIndex(4)).iter();
        let collected: Vec<_> = iter.collect();
        assert_eq!(
            collected,
            vec![MyIndex(0), MyIndex(1), MyIndex(2), MyIndex(3), MyIndex(4)]
        );
    }

    #[test]
    fn test_empty_range_start_greater() {
        let iter = (MyIndex(5)..=MyIndex(3)).iter();
        let collected: Vec<_> = iter.collect();
        assert!(collected.is_empty());
    }

    #[test]
    fn test_empty_range_equal() {
        let iter = (MyIndex(5)..=MyIndex(4)).iter();
        let collected: Vec<_> = iter.collect();
        assert!(collected.is_empty());
    }

    #[test]
    fn test_single_element_range() {
        let iter = (MyIndex(3)..=MyIndex(3)).iter();
        let collected: Vec<_> = iter.collect();
        assert_eq!(collected, vec![MyIndex(3)]);
    }

    #[test]
    fn test_len() {
        assert_eq!((MyIndex(0)..=MyIndex(4)).iter().len(), 5);
        assert_eq!((MyIndex(5)..=MyIndex(3)).iter().len(), 0);
        assert_eq!((MyIndex(3)..=MyIndex(3)).iter().len(), 1);
        assert_eq!((MyIndex(5)..=MyIndex(4)).iter().len(), 0);
    }

    #[test]
    fn test_count() {
        assert_eq!((MyIndex(0)..=MyIndex(4)).iter().count(), 5);
        assert_eq!((MyIndex(5)..=MyIndex(3)).iter().count(), 0);
        assert_eq!((MyIndex(3)..=MyIndex(3)).iter().count(), 1);
    }

    #[test]
    fn test_nth() {
        let mut iter = (MyIndex(0)..=MyIndex(9)).iter();
        assert_eq!(iter.nth(0), Some(MyIndex(0)));
        assert_eq!(iter.nth(2), Some(MyIndex(3)));
        assert_eq!(iter.nth(9), None);
    }

    #[test]
    fn test_nth_independent() {
        let mut iter = (MyIndex(0)..=MyIndex(9)).iter();
        assert_eq!(iter.nth(2), Some(MyIndex(2)));
    }

    #[test]
    fn test_nth_exhaust() {
        let mut iter = (MyIndex(0)..=MyIndex(9)).iter();
        assert_eq!(iter.nth(10), None);
        assert_eq!(iter.nth(100), None);
    }

    #[test]
    fn test_last() {
        assert_eq!((MyIndex(0)..=MyIndex(4)).iter().last(), Some(MyIndex(4)));
        assert_eq!((MyIndex(3)..=MyIndex(3)).iter().last(), Some(MyIndex(3)));
        assert_eq!((MyIndex(5)..=MyIndex(3)).iter().last(), None);
    }

    #[test]
    fn test_size_hint() {
        let iter = (MyIndex(0)..=MyIndex(4)).iter();
        assert_eq!(iter.size_hint(), (5, Some(5)));
    }

    #[test]
    fn test_double_ended() {
        let iter = (MyIndex(0)..=MyIndex(4)).iter();
        let collected: Vec<_> = iter.rev().collect();
        assert_eq!(
            collected,
            vec![MyIndex(4), MyIndex(3), MyIndex(2), MyIndex(1), MyIndex(0)]
        );
    }

    #[test]
    fn test_double_ended_single_element() {
        let iter = (MyIndex(3)..=MyIndex(3)).iter();
        let collected: Vec<_> = iter.rev().collect();
        assert_eq!(collected, vec![MyIndex(3)]);
    }

    #[test]
    fn test_double_ended_empty() {
        let iter = (MyIndex(5)..=MyIndex(3)).iter();
        let collected: Vec<_> = iter.rev().collect();
        assert!(collected.is_empty());
    }

    #[test]
    fn test_nth_back() {
        let mut iter = (MyIndex(0)..=MyIndex(9)).iter();
        assert_eq!(iter.nth_back(0), Some(MyIndex(9)));
        assert_eq!(iter.nth_back(2), Some(MyIndex(6)));
        assert_eq!(iter.nth_back(9), None);
    }

    #[test]
    fn test_exact_size_iterator() {
        let iter = (MyIndex(0)..=MyIndex(4)).iter();
        assert_eq!(iter.len(), 5);
    }

    #[test]
    fn test_fused_iterator() {
        let mut iter = (MyIndex(0)..=MyIndex(2)).iter();
        assert_eq!(iter.next(), Some(MyIndex(0)));
        assert_eq!(iter.next(), Some(MyIndex(1)));
        assert_eq!(iter.next(), Some(MyIndex(2)));
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_min_max() {
        assert_eq!((MyIndex(0)..=MyIndex(4)).iter().min(), Some(MyIndex(0)));
        assert_eq!((MyIndex(0)..=MyIndex(4)).iter().max(), Some(MyIndex(4)));
        assert_eq!((MyIndex(5)..=MyIndex(3)).iter().min(), None);
        assert_eq!((MyIndex(5)..=MyIndex(3)).iter().max(), None);
    }

    #[test]
    fn test_is_sorted() {
        assert!((MyIndex(0)..=MyIndex(4)).iter().is_sorted());
    }
}

mod boundary_tests {
    use super::*;

    #[test]
    fn test_small_index_wrapping() {
        let iter = (SmallIndex(253)..SmallIndex(255)).iter();
        let collected: Vec<_> = iter.collect();
        assert_eq!(collected, vec![SmallIndex(253), SmallIndex(254)]);
    }

    #[test]
    fn test_small_index_at_limit() {
        let iter = (SmallIndex(0)..SmallIndex(254)).iter();
        let count = iter.count();
        assert_eq!(count, 254);
    }

    #[test]
    fn test_small_index_inclusive_at_limit() {
        let iter = (SmallIndex(0)..=SmallIndex(253)).iter();
        let count = iter.count();
        assert_eq!(count, 254);
    }

    #[test]
    fn test_nonzero_index_range() {
        #[derive(IndexType, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
        struct NonZeroIdx(NonZeroUsize);

        let iter = (NonZeroIdx(NonZeroUsize::new(1).unwrap())
            ..NonZeroIdx(NonZeroUsize::new(5).unwrap()))
            .iter();
        let collected: Vec<_> = iter.collect();
        assert_eq!(collected.len(), 4);
    }
}

mod double_ended_equivalence {
    use super::*;

    #[test]
    fn test_range_rev_equals_reversed() {
        let forward: Vec<_> = (MyIndex(0)..MyIndex(5)).iter().collect();
        let reversed: Vec<_> = (MyIndex(0)..MyIndex(5)).iter().rev().collect();
        let expected_rev: Vec<_> = forward.iter().copied().rev().collect();
        assert_eq!(reversed, expected_rev);
    }

    #[test]
    fn test_inclusive_rev_equals_reversed() {
        let forward: Vec<_> = (MyIndex(0)..=MyIndex(4)).iter().collect();
        let reversed: Vec<_> = (MyIndex(0)..=MyIndex(4)).iter().rev().collect();
        let expected_rev: Vec<_> = forward.iter().copied().rev().collect();
        assert_eq!(reversed, expected_rev);
    }

    #[test]
    fn test_mixed_iteration() {
        let mut iter = (MyIndex(0)..MyIndex(10)).iter();
        assert_eq!(iter.next(), Some(MyIndex(0)));
        assert_eq!(iter.next_back(), Some(MyIndex(9)));
        assert_eq!(iter.next(), Some(MyIndex(1)));
        assert_eq!(iter.next_back(), Some(MyIndex(8)));
    }

    #[test]
    fn test_mixed_iteration_inclusive() {
        let mut iter = (MyIndex(0)..=MyIndex(9)).iter();
        assert_eq!(iter.next(), Some(MyIndex(0)));
        assert_eq!(iter.next_back(), Some(MyIndex(9)));
        assert_eq!(iter.next(), Some(MyIndex(1)));
        assert_eq!(iter.next_back(), Some(MyIndex(8)));
    }
}

mod edge_case_iteration {
    use super::*;

    #[test]
    fn test_take() {
        let iter = (MyIndex(0)..).iter().take(3);
        let collected: Vec<_> = iter.collect();
        assert_eq!(collected, vec![MyIndex(0), MyIndex(1), MyIndex(2)]);
    }

    #[test]
    fn test_skip() {
        let iter = (MyIndex(0)..MyIndex(5)).iter().skip(2);
        let collected: Vec<_> = iter.collect();
        assert_eq!(collected, vec![MyIndex(2), MyIndex(3), MyIndex(4)]);
    }

    #[test]
    fn test_skip_inclusive() {
        let iter = (MyIndex(0)..=MyIndex(4)).iter().skip(2);
        let collected: Vec<_> = iter.collect();
        assert_eq!(collected, vec![MyIndex(2), MyIndex(3), MyIndex(4)]);
    }

    #[test]
    fn test_chain() {
        let a: Vec<_> = (MyIndex(0)..MyIndex(2)).iter().collect();
        let b: Vec<_> = (MyIndex(2)..MyIndex(4)).iter().collect();
        let combined: Vec<_> = a.into_iter().chain(b.into_iter()).collect();
        assert_eq!(
            combined,
            vec![MyIndex(0), MyIndex(1), MyIndex(2), MyIndex(3)]
        );
    }

    #[test]
    fn test_zip() {
        let indices: Vec<_> = (MyIndex(0)..MyIndex(3)).iter().collect();
        let values = vec![10, 20, 30];
        let zipped: Vec<_> = indices.iter().zip(values.iter()).collect();
        assert_eq!(zipped.len(), 3);
    }

    #[test]
    fn test_enumerate() {
        let indexed: Vec<_> = (MyIndex(0)..MyIndex(3)).iter().enumerate().collect();
        assert_eq!(
            indexed,
            vec![(0, MyIndex(0)), (1, MyIndex(1)), (2, MyIndex(2))]
        );
    }

    #[test]
    fn test_map() {
        let mapped: Vec<_> = (MyIndex(0)..MyIndex(3)).iter().map(|i| i.0 * 2).collect();
        assert_eq!(mapped, vec![0, 2, 4]);
    }

    #[test]
    fn test_filter() {
        let filtered: Vec<_> = (MyIndex(0)..MyIndex(5))
            .iter()
            .filter(|i| i.0 % 2 == 0)
            .collect();
        assert_eq!(filtered, vec![MyIndex(0), MyIndex(2), MyIndex(4)]);
    }

    #[test]
    fn test_fold() {
        let sum = (MyIndex(0)..MyIndex(5))
            .iter()
            .fold(0u32, |acc, i| acc + i.0);
        assert_eq!(sum, 10);
    }
}
