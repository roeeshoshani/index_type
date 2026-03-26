use core::ops::{Bound, Range, RangeBounds};

use crate::{IndexScalarType, IndexType};

/// Converts `RangeBounds<I>` to raw `usize` bounds.
#[inline]
pub fn range_bounds_to_raw<I: IndexType, R: RangeBounds<I>>(r: &R) -> (Bound<usize>, Bound<usize>) {
    (
        r.start_bound().map(|x| x.to_raw_index()),
        r.end_bound().map(|x| x.to_raw_index()),
    )
}

pub fn resolve_range_bounds<I: IndexType, R: RangeBounds<I>>(r: &R, length: I) -> Range<I> {
    let start = match r.start_bound() {
        core::ops::Bound::Included(i) => *i,
        core::ops::Bound::Excluded(i) => i
            .checked_add_scalar(I::Scalar::ONE)
            .expect("range out of bounds"),
        core::ops::Bound::Unbounded => I::ZERO,
    };

    let end = match r.end_bound() {
        core::ops::Bound::Included(i) => i
            .checked_add_scalar(I::Scalar::ONE)
            .expect("range out of bounds"),
        core::ops::Bound::Excluded(i) => *i,
        core::ops::Bound::Unbounded => length,
    };

    start..end
}
