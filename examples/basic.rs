use std::num::NonZeroU16;

use index_type::{IndexTooBigError, IndexType};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, IndexTooBigError)]
#[error(msg = "item id too big")]
pub struct ItemIdTooBigError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, IndexType)]
#[index_type(error = ItemIdTooBigError)]
pub struct ItemId(NonZeroU16);
fn main() {}
