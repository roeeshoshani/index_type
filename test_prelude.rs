#![no_std]
use core::num::NonZeroU32;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct SafeIndex(NonZeroU32);

fn main() {
    // This should fail if size_of is not in prelude
    let _ = size_of::<SafeIndex>();
}
