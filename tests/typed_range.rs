use core::{
    num::NonZeroUsize,
    ops::{Bound, RangeBounds},
};
use index_type::{
    IndexType,
    enumerate::TypedIteratorExt,
    range::{TypedRange, TypedRangeFrom, TypedRangeInclusive, TypedRangeIterExt},
};

mod utils;

#[derive(IndexType, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
struct MyIndex(u32);

#[derive(IndexType, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct SmallIndex(u8);

mod typed_range {
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
    #[allow(clippy::iter_nth_zero)]
    fn test_nth() {
        let mut iter = (MyIndex(0)..MyIndex(10)).iter();
        assert_eq!(iter.nth(0), Some(MyIndex(0)));
        assert_eq!(iter.nth(2), Some(MyIndex(3)));
        assert_eq!(iter.nth(100), None);
        assert_eq!(iter.next(), None);
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
        assert_eq!(iter.next_back(), None);
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

    #[test]
    fn test_typed_enumerate() {
        let indexed: Vec<_> = (MyIndex(10)..MyIndex(13))
            .iter()
            .typed_enumerate::<MyIndex>()
            .collect();
        assert_eq!(
            indexed,
            vec![
                (MyIndex(0), MyIndex(10)),
                (MyIndex(1), MyIndex(11)),
                (MyIndex(2), MyIndex(12))
            ]
        );
    }

    #[test]
    fn test_typed_enumerate_supports_reverse_and_mixed_iteration() {
        let reversed: Vec<_> = (MyIndex(10)..MyIndex(13))
            .iter()
            .typed_enumerate::<MyIndex>()
            .rev()
            .map(|(idx, value)| (idx.to_raw_index(), value.to_raw_index()))
            .collect();
        assert_eq!(reversed, vec![(2, 12), (1, 11), (0, 10)]);

        let mut iter = (MyIndex(10)..MyIndex(13))
            .iter()
            .typed_enumerate::<MyIndex>();
        assert_eq!(iter.next(), Some((MyIndex(0), MyIndex(10))));
        assert_eq!(iter.next_back(), Some((MyIndex(2), MyIndex(12))));
        assert_eq!(iter.next(), Some((MyIndex(1), MyIndex(11))));
        assert_eq!(iter.next_back(), None);
    }

    #[test]
    #[should_panic(expected = "typed enumerate index overflow")]
    fn test_typed_enumerate_panics_on_overflow() {
        let _ = (0usize..256).typed_enumerate::<SmallIndex>().count();
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
    #[allow(clippy::iter_nth_zero)]
    fn test_nth() {
        let mut iter = (MyIndex(10)..).iter();
        assert_eq!(iter.nth(0), Some(MyIndex(10)));
        assert_eq!(iter.nth(5), Some(MyIndex(16)));
    }

    #[test]
    fn test_range_from_panics_on_overflow_like_std() {
        let mut iter = (SmallIndex(254)..).iter();
        assert_eq!(iter.next(), Some(SmallIndex(254)));
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _ = iter.next();
        }));
        assert!(result.is_err());
    }

    #[test]
    #[should_panic(expected = "called `Result::unwrap()` on an `Err` value")]
    fn test_range_from_nth_panics_on_overflow_like_std() {
        let mut iter = (SmallIndex(250)..).iter();
        let _ = iter.nth(10);
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

mod raw_conversions {
    use super::*;

    #[test]
    fn test_typed_range_from_raw_and_into_raw_preserve_bounds() {
        let range = TypedRange::from_raw(MyIndex(2)..MyIndex(5));
        assert_eq!(range.start, MyIndex(2));
        assert_eq!(range.end, MyIndex(5));

        let raw = range.into_raw();
        assert_eq!(raw.start, MyIndex(2));
        assert_eq!(raw.end, MyIndex(5));
    }

    #[test]
    fn test_typed_range_from_raw_preserves_empty_bounds() {
        let range = TypedRange::from_raw(MyIndex(5)..MyIndex(3));
        assert_eq!(range.start, MyIndex(5));
        assert_eq!(range.end, MyIndex(3));
        assert!(range.is_empty());

        let raw = range.into_raw();
        assert_eq!(raw.start, MyIndex(5));
        assert_eq!(raw.end, MyIndex(3));
    }

    #[test]
    fn test_typed_range_fields_update_during_iteration() {
        let mut range = TypedRange::from_raw(MyIndex(2)..MyIndex(5));

        assert_eq!(range.next(), Some(MyIndex(2)));
        assert_eq!(range.start, MyIndex(3));
        assert_eq!(range.end, MyIndex(5));

        assert_eq!(range.next_back(), Some(MyIndex(4)));
        assert_eq!(range.start, MyIndex(3));
        assert_eq!(range.end, MyIndex(4));
    }

    #[test]
    fn test_typed_range_from_from_raw_and_into_raw_preserve_start() {
        let range = TypedRangeFrom::from_raw(MyIndex(7)..);
        assert_eq!(range.start, MyIndex(7));

        let raw = range.into_raw();
        assert_eq!(raw.start, MyIndex(7));
    }

    #[test]
    fn test_typed_range_from_field_updates_during_iteration() {
        let mut range = TypedRangeFrom::from_raw(MyIndex(7)..);

        assert_eq!(range.next(), Some(MyIndex(7)));
        assert_eq!(range.start, MyIndex(8));

        assert_eq!(range.nth(2), Some(MyIndex(10)));
        assert_eq!(range.start, MyIndex(11));
    }

    #[test]
    fn test_typed_range_inclusive_from_raw_and_into_raw_preserve_non_empty_bounds() {
        let range = TypedRangeInclusive::from_raw(MyIndex(2)..=MyIndex(5));
        assert_eq!(range.start, MyIndex(2));
        assert_eq!(range.end, MyIndex(5));
        assert!(!range.is_empty());

        let raw = range.into_raw();
        assert_eq!(raw.start(), &MyIndex(2));
        assert_eq!(raw.end(), &MyIndex(5));
        assert_eq!(raw.end_bound(), Bound::Included(&MyIndex(5)));
        assert!(!raw.is_empty());
    }

    #[test]
    fn test_typed_range_inclusive_from_raw_preserves_empty_non_exhausted_bounds() {
        let range = TypedRangeInclusive::from_raw(MyIndex(5)..=MyIndex(3));
        assert_eq!(range.start, MyIndex(5));
        assert_eq!(range.end, MyIndex(3));
        assert!(range.is_empty());

        let raw = range.into_raw();
        assert_eq!(raw.start(), &MyIndex(5));
        assert_eq!(raw.end(), &MyIndex(3));
        assert_eq!(raw.end_bound(), Bound::Included(&MyIndex(3)));
        assert!(raw.is_empty());
    }

    #[test]
    fn test_typed_range_inclusive_from_raw_exhausted_does_not_panic() {
        let mut raw = 2usize..=2usize;
        let _ = raw.next();
        let _ = TypedRangeInclusive::from_raw(raw);
    }

    #[test]
    fn test_typed_range_inclusive_iter_accessors_update_during_iteration() {
        let mut iter = TypedRangeInclusive::from_raw(MyIndex(2)..=MyIndex(5)).iter();

        assert_eq!(iter.next(), Some(MyIndex(2)));
        assert_eq!(iter.start(), MyIndex(3));
        assert_eq!(iter.next(), Some(MyIndex(3)));
        assert_eq!(iter.next_back(), Some(MyIndex(5)));
        assert_eq!(iter.end(), MyIndex(4));
        assert_eq!(iter.next_back(), Some(MyIndex(4)));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_typed_range_inclusive_iter_exhaustion() {
        let mut iter = (SmallIndex(3)..=SmallIndex(3)).iter();
        assert_eq!(iter.next(), Some(SmallIndex(3)));
        assert!(iter.is_empty());
        assert_eq!(iter.next(), None);
        assert!(iter.is_empty());
        assert_eq!(iter.next(), None);
        assert!(iter.is_empty());

        let mut iter = (SmallIndex(254)..=SmallIndex(255)).iter();
        assert_eq!(iter.next(), Some(SmallIndex(254)));
        assert_eq!(iter.next(), Some(SmallIndex(255)));
        assert!(iter.is_empty());
        assert_eq!(iter.next(), None);
        assert!(iter.is_empty());
        assert_eq!(iter.next(), None);
        assert!(iter.is_empty());
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
    #[allow(clippy::iter_nth_zero)]
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
        assert_eq!(iter.next(), None);
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
        assert_eq!(iter.next_back(), None);
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
        let combined: Vec<_> = a.into_iter().chain(b).collect();
        assert_eq!(
            combined,
            vec![MyIndex(0), MyIndex(1), MyIndex(2), MyIndex(3)]
        );
    }

    #[test]
    fn test_zip() {
        let indices: Vec<_> = (MyIndex(0)..MyIndex(3)).iter().collect();
        let values = [10, 20, 30];
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

mod overflow_edge_cases {
    use super::*;

    #[test]
    fn test_typed_range_nth_gets_last_element() {
        // There are 5 elements: 0, 1, 2, 3, 4. nth(4) skips 4 and yields the last.
        let mut iter = (SmallIndex(0)..SmallIndex(5)).iter();
        assert_eq!(iter.nth(4), Some(SmallIndex(4)));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_typed_range_nth_back_gets_first_element() {
        // There are 5 elements: 0, 1, 2, 3, 4. nth_back(4) skips 4 from the back and yields the first.
        let mut iter = (SmallIndex(0)..SmallIndex(5)).iter();
        assert_eq!(iter.nth_back(4), Some(SmallIndex(0)));
        assert_eq!(iter.next_back(), None);
    }

    #[test]
    fn test_typed_range_nth_overflow_scalar_type() {
        // SmallIndex 250..255 has elements 250, 251, 252, 253, 254.
        // nth(5) would skip all 5 and return None (no overflow, just past the end).
        let mut iter = (SmallIndex(250)..SmallIndex(255)).iter();
        assert_eq!(iter.nth(5), None);
        assert_eq!(iter.next(), None);

        // nth(6) causes checked_add_scalar(250, 6) to overflow u8.
        let mut iter = (SmallIndex(250)..SmallIndex(255)).iter();
        assert_eq!(iter.nth(6), None);
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_typed_range_nth_back_underflow_scalar_type() {
        // SmallIndex 0..5 has elements 0, 1, 2, 3, 4.
        // nth_back(5) would skip all 5 from the back and return None.
        let mut iter = (SmallIndex(0)..SmallIndex(5)).iter();
        assert_eq!(iter.nth_back(5), None);
        assert_eq!(iter.next_back(), None);

        // nth_back(6) causes checked_sub_scalar(5, 6) to underflow.
        let mut iter = (SmallIndex(0)..SmallIndex(5)).iter();
        assert_eq!(iter.nth_back(6), None);
        assert_eq!(iter.next_back(), None);
    }

    #[test]
    fn test_typed_range_size_hint_empty() {
        let iter = (SmallIndex(5)..SmallIndex(3)).iter();
        assert_eq!(iter.size_hint(), (0, Some(0)));

        let iter = (SmallIndex(5)..SmallIndex(5)).iter();
        assert_eq!(iter.size_hint(), (0, Some(0)));
    }

    #[test]
    #[allow(clippy::iter_nth_zero)]
    fn test_typed_range_nth_after_nth_back_exhausts() {
        let mut iter = (SmallIndex(0)..SmallIndex(5)).iter();
        // Exhaust from the back.
        let _ = iter.nth_back(4);
        assert_eq!(iter.nth(0), None);
    }

    #[test]
    #[allow(clippy::iter_nth_zero)]
    fn test_typed_range_nth_back_after_nth_exhausts() {
        let mut iter = (SmallIndex(0)..SmallIndex(5)).iter();
        // Exhaust from the front.
        let _ = iter.nth(4);
        assert_eq!(iter.nth_back(0), None);
    }

    #[test]
    #[allow(clippy::iter_nth_zero)]
    fn test_typed_range_from_nth_at_max_boundary() {
        // SmallIndex(254).. yields 254, 255, then panics on overflow.
        let mut iter = (SmallIndex(254)..).iter();
        // nth(0) gives the current start (254) and advances to 255.
        assert_eq!(iter.nth(0), Some(SmallIndex(254)));
        // next() gives 255 and tries to advance past u8::MAX, which panics.
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _ = iter.next();
        }));
        assert!(result.is_err());
    }

    #[test]
    fn test_typed_range_from_nth_overflow_panics() {
        let mut iter = (SmallIndex(254)..).iter();
        // nth(1) skips 254 and tries to yield 255, but the +1 past it overflows.
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _ = iter.nth(1);
        }));
        assert!(result.is_err());
    }

    #[test]
    fn test_inclusive_nth_overflow_at_max() {
        // SmallIndex(254)..=SmallIndex(255) has elements 254, 255.
        // nth(1) skips 254 and yields 255, but the +1 after it overflows.
        let mut iter = (SmallIndex(254)..=SmallIndex(255)).iter();
        assert_eq!(iter.nth(1), Some(SmallIndex(255)));
        assert!(iter.is_empty());
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_inclusive_nth_back_overflow_at_min() {
        // SmallIndex(0)..=SmallIndex(1) has elements 0, 1.
        // nth_back(1) skips 1 and yields 0, but the -1 after it underflows.
        let mut iter = (SmallIndex(0)..=SmallIndex(1)).iter();
        assert_eq!(iter.nth_back(1), Some(SmallIndex(0)));
        assert!(iter.is_empty());
        assert_eq!(iter.next_back(), None);
    }

    #[test]
    fn test_inclusive_nth_gets_last_element() {
        // SmallIndex(0)..=SmallIndex(4) has 5 elements.
        // nth(4) skips 0, 1, 2, 3 and yields 4.
        let mut iter = (SmallIndex(0)..=SmallIndex(4)).iter();
        assert_eq!(iter.nth(4), Some(SmallIndex(4)));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_inclusive_nth_back_gets_first_element() {
        // SmallIndex(0)..=SmallIndex(4) has 5 elements.
        // nth_back(4) skips 4, 3, 2, 1 from the back and yields 0.
        let mut iter = (SmallIndex(0)..=SmallIndex(4)).iter();
        assert_eq!(iter.nth_back(4), Some(SmallIndex(0)));
        assert_eq!(iter.next_back(), None);
    }

    #[test]
    #[allow(clippy::iter_nth_zero)]
    fn test_inclusive_nth_zero_on_empty() {
        let mut iter = (SmallIndex(5)..=SmallIndex(3)).iter();
        assert_eq!(iter.nth(0), None);

        let mut iter = (SmallIndex(5)..=SmallIndex(4)).iter();
        assert_eq!(iter.nth(0), None);
    }

    #[test]
    #[allow(clippy::iter_nth_zero)]
    fn test_inclusive_nth_back_zero_on_empty() {
        let mut iter = (SmallIndex(5)..=SmallIndex(3)).iter();
        assert_eq!(iter.nth_back(0), None);

        let mut iter = (SmallIndex(5)..=SmallIndex(4)).iter();
        assert_eq!(iter.nth_back(0), None);
    }

    #[test]
    fn test_inclusive_size_hint_empty() {
        let iter = (SmallIndex(5)..=SmallIndex(3)).iter();
        assert_eq!(iter.size_hint(), (0, Some(0)));

        let iter = (SmallIndex(5)..=SmallIndex(4)).iter();
        assert_eq!(iter.size_hint(), (0, Some(0)));
    }

    #[test]
    fn test_inclusive_size_hint_after_partial_iteration() {
        let mut iter = (SmallIndex(0)..=SmallIndex(4)).iter();
        assert_eq!(iter.size_hint(), (5, Some(5)));
        let _ = iter.next();
        assert_eq!(iter.size_hint(), (4, Some(4)));
        let _ = iter.next_back();
        assert_eq!(iter.size_hint(), (3, Some(3)));
    }

    #[test]
    #[allow(clippy::iter_nth_zero)]
    fn test_inclusive_nth_after_nth_back_exhausts() {
        let mut iter = (SmallIndex(0)..=SmallIndex(4)).iter();
        // Exhaust from the back.
        let _ = iter.nth_back(4);
        assert_eq!(iter.nth(0), None);
    }

    #[test]
    #[allow(clippy::iter_nth_zero)]
    fn test_inclusive_nth_back_after_nth_exhausts() {
        let mut iter = (SmallIndex(0)..=SmallIndex(4)).iter();
        // Exhaust from the front.
        let _ = iter.nth(4);
        assert_eq!(iter.nth_back(0), None);
    }

    #[test]
    fn test_inclusive_next_back_overflow_at_min() {
        // SmallIndex(0)..=SmallIndex(0) has one element.
        let mut iter = (SmallIndex(0)..=SmallIndex(0)).iter();
        assert_eq!(iter.next_back(), Some(SmallIndex(0)));
        // The -1 underflows, setting exhausted = true.
        assert!(iter.is_empty());
        assert_eq!(iter.next_back(), None);
    }

    #[test]
    fn test_inclusive_mixed_iteration_across_the_whole_range() {
        let mut iter = (SmallIndex(0)..=SmallIndex(5)).iter();
        // Collect from both ends.
        assert_eq!(iter.next(), Some(SmallIndex(0)));
        assert_eq!(iter.next_back(), Some(SmallIndex(5)));
        assert_eq!(iter.next(), Some(SmallIndex(1)));
        assert_eq!(iter.next_back(), Some(SmallIndex(4)));
        assert_eq!(iter.next(), Some(SmallIndex(2)));
        assert_eq!(iter.next_back(), Some(SmallIndex(3)));
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next_back(), None);
    }

    #[test]
    fn test_inclusive_nth_with_n_equals_range_length_minus_one() {
        // SmallIndex(0)..=SmallIndex(4) has 5 elements.
        // nth(3) skips 0, 1, 2, yields 3, leaving 4.
        let mut iter = (SmallIndex(0)..=SmallIndex(4)).iter();
        assert_eq!(iter.nth(3), Some(SmallIndex(3)));
        assert_eq!(iter.next(), Some(SmallIndex(4)));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_inclusive_nth_back_overflow_at_max() {
        // SmallIndex(255)..=SmallIndex(255) has one element.
        // nth_back(0) yields 255, then the -1 underflows.
        let mut iter = (SmallIndex(255)..=SmallIndex(255)).iter();
        assert_eq!(iter.nth_back(0), Some(SmallIndex(255)));
        assert!(iter.is_empty());

        // SmallIndex(254)..=SmallIndex(255) has two elements.
        // nth_back(1) skips 255, yields 254, then the -1 underflows.
        let mut iter = (SmallIndex(254)..=SmallIndex(255)).iter();
        assert_eq!(iter.nth_back(1), Some(SmallIndex(254)));
        assert!(iter.is_empty());
    }

    #[test]
    #[allow(clippy::iter_nth_zero)]
    fn test_inclusive_nth_exhaust_after_backward_exhaustion() {
        let mut iter = (SmallIndex(0)..=SmallIndex(5)).iter();
        // Exhaust from the front.
        assert_eq!(iter.nth(5), Some(SmallIndex(5)));
        assert!(iter.is_empty());
        // nth should still return None.
        assert_eq!(iter.nth(0), None);
        assert_eq!(iter.nth(1), None);
        assert_eq!(iter.nth(100), None);
    }

    #[test]
    #[allow(clippy::iter_nth_zero)]
    fn test_inclusive_nth_back_exhaust_after_forward_exhaustion() {
        let mut iter = (SmallIndex(0)..=SmallIndex(5)).iter();
        // Exhaust from the front.
        assert_eq!(iter.nth(5), Some(SmallIndex(5)));
        assert!(iter.is_empty());
        // nth_back should still return None.
        assert_eq!(iter.nth_back(0), None);
        assert_eq!(iter.nth_back(1), None);
        assert_eq!(iter.nth_back(100), None);
    }
}

#[cfg(feature = "serde")]
mod serde_tests {
    use crate::utils::test_serde_roundtrip;

    use super::*;

    #[test]
    fn test_typed_range_roundtrip() {
        test_serde_roundtrip(&TypedRange {
            start: MyIndex(3),
            end: MyIndex(10),
        });
    }

    #[test]
    fn test_typed_range_from_roundtrip() {
        test_serde_roundtrip(&TypedRangeFrom { start: MyIndex(5) });
    }

    #[test]
    fn test_typed_range_inclusive_roundtrip() {
        test_serde_roundtrip(&TypedRangeInclusive {
            start: MyIndex(3),
            end: MyIndex(10),
        });
    }
}
