use crate::{IndexTooBigError, IndexType};

unsafe impl IndexType for usize {
    const ZERO: Self = 0;

    #[inline(always)]
    fn try_from_index(index: usize) -> Result<Self, IndexTooBigError> {
        Ok(index)
    }

    #[inline(always)]
    unsafe fn from_index_unchecked(index: usize) -> Self {
        index
    }

    #[inline(always)]
    fn to_index(self) -> usize {
        self
    }

    #[inline(always)]
    unsafe fn unchecked_add_usize(self, rhs: usize) -> Self {
        unsafe { self.unchecked_add(rhs) }
    }

    #[inline(always)]
    unsafe fn unchecked_sub(self, rhs: Self) -> Self {
        unsafe { self.unchecked_sub(rhs) }
    }
}
