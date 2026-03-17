#[cfg(target_pointer_width = "64")]
use core::num::NonZeroU64;
use core::num::{NonZeroU8, NonZeroU16, NonZeroU32, NonZeroUsize};

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
    fn checked_add_usize(self, rhs: usize) -> Result<Self, IndexTooBigError> {
        self.checked_add(rhs).ok_or(IndexTooBigError)
    }

    #[inline(always)]
    unsafe fn unchecked_add_usize(self, rhs: usize) -> Self {
        unsafe { self.unchecked_add(rhs) }
    }

    #[inline(always)]
    unsafe fn unchecked_sub(self, rhs: Self) -> usize {
        unsafe { self.unchecked_sub(rhs) }
    }
}

macro_rules! impl_for_uint_type {
    {$t: ty} => {
        unsafe impl IndexType for $t {
            const ZERO: Self = 0;

            #[inline(always)]
            fn try_from_index(index: usize) -> Result<Self, IndexTooBigError> {
                index.try_into().map_err(|_| IndexTooBigError)
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
            fn checked_add_usize(self, rhs: usize) -> Result<Self, IndexTooBigError> {
                self.checked_add(rhs.try_into().map_err(|_| IndexTooBigError)?).ok_or(IndexTooBigError)
            }

            #[inline(always)]
            unsafe fn unchecked_add_usize(self, rhs: usize) -> Self {
                unsafe { self.unchecked_add(rhs as Self) }
            }

            #[inline(always)]
            unsafe fn unchecked_sub(self, rhs: Self) -> usize {
                unsafe { self.unchecked_sub(rhs) as usize }
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
    const ZERO: Self = unsafe { Self::new_unchecked(1) };

    fn try_from_index(index: usize) -> Result<Self, IndexTooBigError> {
        let raw = index.checked_add(1).ok_or(IndexTooBigError)?;
        Ok(unsafe { Self::new_unchecked(raw) })
    }

    unsafe fn from_index_unchecked(index: usize) -> Self {
        unsafe { Self::new_unchecked(index.unchecked_add(1)) }
    }

    fn to_index(self) -> usize {
        unsafe { self.get().unchecked_sub(1) }
    }

    fn checked_add_usize(self, rhs: usize) -> Result<Self, IndexTooBigError> {
        self.checked_add(rhs).ok_or(IndexTooBigError)
    }

    unsafe fn unchecked_add_usize(self, rhs: usize) -> Self {
        unsafe { Self::new_unchecked(self.get().unchecked_add(rhs)) }
    }

    unsafe fn unchecked_sub(self, rhs: Self) -> usize {
        unsafe { self.get().unchecked_sub(rhs.get()) }
    }
}

macro_rules! impl_for_nonzero_uint_type {
    {$t: ty} => {
        unsafe impl IndexType for $t {
            const ZERO: Self = unsafe { Self::new_unchecked(1) };

            fn try_from_index(index: usize) -> Result<Self, IndexTooBigError> {
                let raw = index
                    .checked_add(1)
                    .ok_or(IndexTooBigError)?
                    .try_into()
                    .map_err(|_| IndexTooBigError)?;
                Ok(unsafe { Self::new_unchecked(raw) })
            }

            unsafe fn from_index_unchecked(index: usize) -> Self {
                unsafe { Self::new_unchecked(index.unchecked_add(1) as _) }
            }

            fn to_index(self) -> usize {
                unsafe { self.get().unchecked_sub(1) as usize }
            }

            fn checked_add_usize(self, rhs: usize) -> Result<Self, IndexTooBigError> {
                self.checked_add(rhs.try_into().map_err(|_| IndexTooBigError)?).ok_or(IndexTooBigError)
            }

            unsafe fn unchecked_add_usize(self, rhs: usize) -> Self {
                unsafe { Self::new_unchecked(self.get().unchecked_add(rhs as _)) }
            }

            unsafe fn unchecked_sub(self, rhs: Self) -> usize {
                unsafe { self.get().unchecked_sub(rhs.get()) as usize }
            }
        }
    };
}
impl_for_nonzero_uint_type! {NonZeroU8}
impl_for_nonzero_uint_type! {NonZeroU16}

#[cfg(any(target_pointer_width = "32", target_pointer_width = "64"))]
impl_for_nonzero_uint_type! {NonZeroU32}

#[cfg(target_pointer_width = "64")]
impl_for_nonzero_uint_type! {NonZeroU64}
