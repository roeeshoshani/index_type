#[cfg(target_pointer_width = "64")]
use core::num::NonZeroU64;
use core::num::{NonZeroU16, NonZeroU32, NonZeroU8, NonZeroUsize};

use crate::{error::GenericIndexTooBigError, IndexType};

macro_rules! impl_for_uint_type {
    {$t: ty} => {
        const _: () = if <$t>::BITS > usize::BITS {
            panic!()
        };
        unsafe impl IndexType for $t {
            type IndexTooBigError = GenericIndexTooBigError;

            type Scalar = Self;

            const ZERO: Self = 0;

            const MAX_INDEX: Self = Self::MAX;

            const MAX_RAW_INDEX: usize = (Self::MAX) as usize;

            #[inline]
            fn try_from_raw_index(index: usize) -> Result<Self, Self::IndexTooBigError> {
                index.try_into().map_err(|_| GenericIndexTooBigError)
            }

            #[inline]
            unsafe fn from_raw_index_unchecked(index: usize) -> Self {
                index as Self
            }

            #[inline]
            fn to_raw_index(self) -> usize {
                self as usize
            }

            #[inline]
            fn try_from_scalar(scalar: Self::Scalar) -> Result<Self, Self::IndexTooBigError> {
                Ok(scalar)
            }

            #[inline]
            unsafe fn from_scalar_unchecked(scalar: Self::Scalar) -> Self {
                scalar
            }

            #[inline]
            fn to_scalar(self) -> Self::Scalar {
                self
            }

            #[inline]
            fn checked_add_scalar(self, rhs: Self::Scalar) -> Result<Self, Self::IndexTooBigError> {
                self.checked_add(rhs).ok_or(GenericIndexTooBigError)
            }

            #[inline]
            fn checked_mul_scalar(self, rhs: Self::Scalar) -> Result<Self, Self::IndexTooBigError> {
                self.checked_mul(rhs).ok_or(GenericIndexTooBigError)
            }

            #[inline]
            unsafe fn unchecked_add_scalar(self, rhs: Self::Scalar) -> Self {
                // SAFETY: The caller ensures the result is in bounds.
                unsafe { self.unchecked_add(rhs) }
            }

            #[inline]
            unsafe fn unchecked_sub_scalar(self, rhs: Self::Scalar) -> Self {
                // SAFETY: The caller ensures the result is in bounds.
                unsafe { self.unchecked_sub(rhs) }
            }

            #[inline]
            unsafe fn unchecked_sub_index(self, rhs: Self) -> Self::Scalar {
                // SAFETY: The caller ensures the result is in bounds.
                unsafe { self.unchecked_sub(rhs) }
            }

            #[inline]
            fn checked_sub_scalar(self, rhs: Self::Scalar) -> Option<Self> {
                self.checked_sub(rhs)
            }

            #[inline]
            fn checked_sub_index(self, rhs: Self) -> Option<Self::Scalar> {
                self.checked_sub(rhs)
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

impl_for_uint_type! {usize}

macro_rules! impl_for_nonzero_uint_type {
    {$t: ty, $scalar: ty} => {
        const _: () = if <$t>::BITS > usize::BITS {
            panic!()
        };
        unsafe impl IndexType for $t {
            type IndexTooBigError = GenericIndexTooBigError;

            type Scalar = $scalar;

            const ZERO: Self = unsafe { Self::new_unchecked(1) };

            const MAX_INDEX: Self = unsafe { Self::new_unchecked(<$scalar>::MAX) };

            const MAX_RAW_INDEX: usize = (<$scalar>::MAX - 1) as usize;

            #[inline]
            fn try_from_raw_index(index: usize) -> Result<Self, Self::IndexTooBigError> {
                let raw = index
                    .checked_add(1)
                    .ok_or(GenericIndexTooBigError)?
                    .try_into()
                    .map_err(|_| GenericIndexTooBigError)?;
                Ok(unsafe { Self::new_unchecked(raw) })
            }

            #[inline]
            unsafe fn from_raw_index_unchecked(index: usize) -> Self {
                // SAFETY: The caller ensures the index is in bounds.
                unsafe { Self::new_unchecked(index.unchecked_add(1) as _) }
            }

            #[inline]
            fn to_raw_index(self) -> usize {
                // SAFETY: NonZero uint is at least 1.
                unsafe { self.get().unchecked_sub(1) as usize }
            }

            #[inline]
            fn try_from_scalar(scalar: Self::Scalar) -> Result<Self, Self::IndexTooBigError> {
                let raw = scalar.checked_add(1).ok_or(GenericIndexTooBigError)?;
                // SAFETY: scalar + 1 is at least 1.
                Ok(unsafe { Self::new_unchecked(raw) })
            }

            #[inline]
            unsafe fn from_scalar_unchecked(scalar: Self::Scalar) -> Self {
                // SAFETY: The caller ensures the scalar is in bounds.
                unsafe { Self::new_unchecked(scalar.unchecked_add(1)) }
            }

            #[inline]
            fn to_scalar(self) -> Self::Scalar {
                // SAFETY: NonZero uint is at least 1.
                unsafe { self.get().unchecked_sub(1) }
            }

            #[inline]
            fn checked_add_scalar(self, rhs: Self::Scalar) -> Result<Self, Self::IndexTooBigError> {
                self.checked_add(rhs).ok_or(GenericIndexTooBigError)
            }

            #[inline]
            fn checked_mul_scalar(self, rhs: Self::Scalar) -> Result<Self, Self::IndexTooBigError> {
                self.to_scalar()
                    .checked_mul(rhs)
                    .ok_or(GenericIndexTooBigError)
                    .and_then(Self::try_from_scalar)
            }

            #[inline]
            unsafe fn unchecked_add_scalar(self, rhs: Self::Scalar) -> Self {
                // SAFETY: The caller ensures the result is in bounds.
                unsafe { Self::new_unchecked(self.get().unchecked_add(rhs)) }
            }

            #[inline]
            unsafe fn unchecked_sub_scalar(self, rhs: Self::Scalar) -> Self {
                // SAFETY: The caller ensures the result is in bounds.
                unsafe { Self::new_unchecked(self.get().unchecked_sub(rhs)) }
            }

            #[inline]
            unsafe fn unchecked_sub_index(self, rhs: Self) -> Self::Scalar {
                // SAFETY: The caller ensures the result is in bounds.
                unsafe { self.get().unchecked_sub(rhs.get()) }
            }

            #[inline]
            fn checked_sub_scalar(self, rhs: Self::Scalar) -> Option<Self> {
                let val = self.get().checked_sub(rhs)?;
                Self::new(val)
            }

            #[inline]
            fn checked_sub_index(self, rhs: Self) -> Option<Self::Scalar> {
                self.get().checked_sub(rhs.get())
            }
        }
    };
}
impl_for_nonzero_uint_type! {NonZeroU8, u8}
impl_for_nonzero_uint_type! {NonZeroU16, u16}

#[cfg(any(target_pointer_width = "32", target_pointer_width = "64"))]
impl_for_nonzero_uint_type! {NonZeroU32, u32}

#[cfg(target_pointer_width = "64")]
impl_for_nonzero_uint_type! {NonZeroU64, u64}

impl_for_nonzero_uint_type! {NonZeroUsize, usize}
