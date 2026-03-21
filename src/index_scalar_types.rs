use crate::{IndexScalarType, error::GenericIndexTooBigError};

macro_rules! impl_for_uint_type {
    {$t: ty} => {
        const _: () = if <$t>::BITS > usize::BITS {
            panic!()
        };
        impl crate::index_scalar_type_private::Sealed for $t {}
        unsafe impl IndexScalarType for $t {
            type IndexTooBigError = GenericIndexTooBigError;

            const ZERO: Self = 0;
            const ONE: Self = 1;

            #[inline]
            fn try_from_usize(value: usize) -> Result<Self, Self::IndexTooBigError> {
                value.try_into().map_err(|_| GenericIndexTooBigError)
            }

            #[inline]
            unsafe fn from_usize_unchecked(value: usize) -> Self {
                value as Self
            }

            #[inline]
            fn to_usize(self) -> usize {
                self as usize
            }

            #[inline]
            fn checked_add_scalar(self, rhs: Self) -> Result<Self, Self::IndexTooBigError> {
                self.checked_add(rhs).ok_or(GenericIndexTooBigError)
            }

            #[inline]
            unsafe fn unchecked_add_scalar(self, rhs: Self) -> Self {
                unsafe { self.unchecked_add(rhs) }
            }
        }
    };
}
impl_for_uint_type! {u8}
impl_for_uint_type! {u16}
impl_for_uint_type! {u32}
impl_for_uint_type! {u64}
impl_for_uint_type! {usize}
