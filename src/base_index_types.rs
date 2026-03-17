#[cfg(target_pointer_width = "64")]
use core::num::NonZeroU64;
use core::num::{NonZeroU8, NonZeroU16, NonZeroU32, NonZeroUsize};

use crate::{IndexTooBigError, IndexType};

unsafe impl IndexType for usize {
    type Scalar = Self;

    const ZERO: Self = 0;

    #[inline(always)]
    fn try_from_raw_index(index: usize) -> Result<Self, IndexTooBigError> {
        Ok(index)
    }

    #[inline(always)]
    unsafe fn from_raw_index_unchecked(index: usize) -> Self {
        index
    }

    #[inline(always)]
    fn to_raw_index(self) -> usize {
        self
    }

    #[inline(always)]
    fn try_from_scalar(scalar: Self::Scalar) -> Result<Self, IndexTooBigError> {
        Ok(scalar)
    }

    #[inline(always)]
    unsafe fn from_scalar_unchecked(scalar: Self::Scalar) -> Self {
        scalar
    }

    #[inline(always)]
    fn to_scalar(self) -> Self::Scalar {
        self
    }

    #[inline(always)]
    fn checked_add_scalar(self, rhs: Self::Scalar) -> Result<Self, IndexTooBigError> {
        self.checked_add(rhs).ok_or(IndexTooBigError)
    }

    #[inline(always)]
    unsafe fn unchecked_add_scalar(self, rhs: Self::Scalar) -> Self {
        unsafe { self.unchecked_add(rhs) }
    }

    #[inline(always)]
    unsafe fn unchecked_sub_index(self, rhs: Self) -> Self::Scalar {
        unsafe { self.unchecked_sub(rhs) }
    }
}

macro_rules! impl_for_uint_type {
    {$t: ty} => {
        const _: () = if <$t>::BITS > usize::BITS {
            panic!()
        };
        unsafe impl IndexType for $t {
            type Scalar = Self;

            const ZERO: Self = 0;

            #[inline(always)]
            fn try_from_raw_index(index: usize) -> Result<Self, IndexTooBigError> {
                index.try_into().map_err(|_| IndexTooBigError)
            }

            #[inline(always)]
            unsafe fn from_raw_index_unchecked(index: usize) -> Self {
                index as Self
            }

            #[inline(always)]
            fn to_raw_index(self) -> usize {
                self as usize
            }

            #[inline(always)]
            fn try_from_scalar(scalar: Self::Scalar) -> Result<Self, IndexTooBigError> {
                Ok(scalar)
            }

            #[inline(always)]
            unsafe fn from_scalar_unchecked(scalar: Self::Scalar) -> Self {
                scalar
            }

            #[inline(always)]
            fn to_scalar(self) -> Self::Scalar {
                self
            }

            #[inline(always)]
            fn checked_add_scalar(self, rhs: Self::Scalar) -> Result<Self, IndexTooBigError> {
                self.checked_add(rhs).ok_or(IndexTooBigError)
            }

            #[inline(always)]
            unsafe fn unchecked_add_scalar(self, rhs: Self::Scalar) -> Self {
                unsafe { self.unchecked_add(rhs) }
            }

            #[inline(always)]
            unsafe fn unchecked_sub_index(self, rhs: Self) -> Self::Scalar {
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

unsafe impl IndexType for NonZeroUsize {
    type Scalar = usize;

    const ZERO: Self = unsafe { Self::new_unchecked(1) };

    #[inline(always)]
    fn try_from_raw_index(index: usize) -> Result<Self, IndexTooBigError> {
        let raw = index.checked_add(1).ok_or(IndexTooBigError)?;
        Ok(unsafe { Self::new_unchecked(raw) })
    }

    #[inline(always)]
    unsafe fn from_raw_index_unchecked(index: usize) -> Self {
        unsafe { Self::new_unchecked(index.unchecked_add(1)) }
    }

    #[inline(always)]
    fn to_raw_index(self) -> usize {
        unsafe { self.get().unchecked_sub(1) }
    }

    #[inline(always)]
    fn try_from_scalar(scalar: Self::Scalar) -> Result<Self, IndexTooBigError> {
        let raw = scalar.checked_add(1).ok_or(IndexTooBigError)?;
        Ok(unsafe { Self::new_unchecked(raw) })
    }

    #[inline(always)]
    unsafe fn from_scalar_unchecked(scalar: Self::Scalar) -> Self {
        unsafe { Self::new_unchecked(scalar.unchecked_add(1)) }
    }

    #[inline(always)]
    fn to_scalar(self) -> Self::Scalar {
        unsafe { self.get().unchecked_sub(1) }
    }

    #[inline(always)]
    fn checked_add_scalar(self, rhs: Self::Scalar) -> Result<Self, IndexTooBigError> {
        self.checked_add(rhs).ok_or(IndexTooBigError)
    }

    #[inline(always)]
    unsafe fn unchecked_add_scalar(self, rhs: Self::Scalar) -> Self {
        unsafe { Self::new_unchecked(self.get().unchecked_add(rhs)) }
    }

    #[inline(always)]
    unsafe fn unchecked_sub_index(self, rhs: Self) -> Self::Scalar {
        unsafe { self.get().unchecked_sub(rhs.get()) }
    }
}

macro_rules! impl_for_nonzero_uint_type {
    {$t: ty, $scalar: ty} => {
        const _: () = if <$t>::BITS > usize::BITS {
            panic!()
        };
        unsafe impl IndexType for $t {
            type Scalar = $scalar;

            const ZERO: Self = unsafe { Self::new_unchecked(1) };

            #[inline(always)]
            fn try_from_raw_index(index: usize) -> Result<Self, IndexTooBigError> {
                let raw = index
                    .checked_add(1)
                    .ok_or(IndexTooBigError)?
                    .try_into()
                    .map_err(|_| IndexTooBigError)?;
                Ok(unsafe { Self::new_unchecked(raw) })
            }

            #[inline(always)]
            unsafe fn from_raw_index_unchecked(index: usize) -> Self {
                unsafe { Self::new_unchecked(index.unchecked_add(1) as _) }
            }

            #[inline(always)]
            fn to_raw_index(self) -> usize {
                unsafe { self.get().unchecked_sub(1) as usize }
            }

            #[inline(always)]
            fn try_from_scalar(scalar: Self::Scalar) -> Result<Self, IndexTooBigError> {
                let raw = scalar.checked_add(1).ok_or(IndexTooBigError)?;
                Ok(unsafe { Self::new_unchecked(raw) })
            }

            #[inline(always)]
            unsafe fn from_scalar_unchecked(scalar: Self::Scalar) -> Self {
                unsafe { Self::new_unchecked(scalar.unchecked_add(1)) }
            }

            #[inline(always)]
            fn to_scalar(self) -> Self::Scalar {
                unsafe { self.get().unchecked_sub(1) }
            }

            #[inline(always)]
            fn checked_add_scalar(self, rhs: Self::Scalar) -> Result<Self, IndexTooBigError> {
                self.checked_add(rhs).ok_or(IndexTooBigError)
            }

            #[inline(always)]
            unsafe fn unchecked_add_scalar(self, rhs: Self::Scalar) -> Self {
                unsafe { Self::new_unchecked(self.get().unchecked_add(rhs)) }
            }

            #[inline(always)]
            unsafe fn unchecked_sub_index(self, rhs: Self) -> Self::Scalar {
                unsafe { self.get().unchecked_sub(rhs.get()) }
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
