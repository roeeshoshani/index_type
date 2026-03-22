use std::num::NonZeroU16;

use index_type::IndexType;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, IndexType)]
pub struct ItemId(NonZeroU16);

fn main() {
    let x = ItemId::try_from_raw_index(5);
    println!("{:?}", x);
}
