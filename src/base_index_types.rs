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

macro_rules! impl_for_uint_type {
    {$t: ty} => {
        unsafe impl IndexType for $t {
            const ZERO: Self = 0;

            #[inline(always)]
            fn try_from_index(index: usize) -> Result<Self, IndexTooBigError> {
                Self::try_from(index).map_err(|_| IndexTooBigError)
            }

            #[inline(always)]
            unsafe fn from_index_unchecked(index: usize) -> Self {
                index as Self
            }

            #[inline(always)]
            fn to_index(self) -> usize {
                const _: () = if <$t>::BITS > usize::BITS {
                    panic!()
                };
                self as usize
            }

            #[inline(always)]
            unsafe fn unchecked_add_usize(self, rhs: usize) -> Self {
                unsafe { self.unchecked_add(rhs as $t) }
            }

            #[inline(always)]
            unsafe fn unchecked_sub(self, rhs: Self) -> Self {
                unsafe { self.unchecked_sub(rhs) }
            }
        }
    };
}

impl_for_uint_type! {u8}
impl_for_uint_type! {u16}

#[cfg(any(target_pointer_width = "32", target_pointer_width = "64"))]
impl_for_uint_type! {u32}

#[cfg(target_pointer_width = "64")]
impl_for_uint_type! {u64}
