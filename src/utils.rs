use core::ops::{Bound, RangeBounds};

use crate::IndexType;

/// Converts `RangeBounds<I>` to raw `usize` bounds.
#[inline]
pub fn range_bounds_to_raw<I: IndexType, R: RangeBounds<I>>(r: R) -> (Bound<usize>, Bound<usize>) {
    (
        r.start_bound().map(|x| x.to_raw_index()),
        r.end_bound().map(|x| x.to_raw_index()),
    )
}
