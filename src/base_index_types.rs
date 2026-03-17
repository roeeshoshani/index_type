#[cfg(target_pointer_width = "64")]
use core::num::NonZeroU64;
use core::num::{NonZeroU16, NonZeroU32, NonZeroU8, NonZeroUsize};

use crate::{IndexTooBigError, IndexType};

// SAFETY: usize is the native index type, so it trivially satisfies all requirements.
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
        // SAFETY: The caller guarantees that the result is representable by this type.
        unsafe { self.unchecked_add(rhs) }
    }

    #[inline(always)]
    unsafe fn unchecked_sub(self, rhs: Self) -> usize {
        // SAFETY: The caller guarantees that rhs <= self.
        unsafe { self.unchecked_sub(rhs) }
    }
}

macro_rules! impl_for_uint_type {
    {$t: ty} => {
        // SAFETY: These types are smaller or equal to usize (enforced by to_index).
        // They are native unsigned integers and satisfy the requirements.
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
            unsafe fn unchecked_add_usize(self, rhs: usize) -> Self {
                // SAFETY: The caller guarantees that the result is representable by $t.
                unsafe { self.unchecked_add(rhs as Self) }
            }

            #[inline(always)]
            unsafe fn unchecked_sub(self, rhs: Self) -> usize {
                // SAFETY: The caller guarantees that rhs <= self.
                unsafe { (self.unchecked_sub(rhs)) as usize }
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

// SAFETY: NonZeroUsize implements IndexType by mapping [0, max-1] to [1, max].
// ZERO is 1 (representing 0).
unsafe impl IndexType for NonZeroUsize {
    const ZERO: Self = unsafe {
        // SAFETY: 1 is non-zero.
        Self::new_unchecked(1)
    };

    fn try_from_index(index: usize) -> Result<Self, IndexTooBigError> {
        let raw = index.checked_add(1).ok_or(IndexTooBigError)?;
        // SAFETY: raw is index + 1, and index >= 0, so raw >= 1.
        Ok(unsafe { Self::new_unchecked(raw) })
    }

    unsafe fn from_index_unchecked(index: usize) -> Self {
        // SAFETY: The caller guarantees index is representable.
        // For NonZeroUsize, it means index+1 doesn't overflow.
        // index+1 will be >= 1.
        unsafe { Self::new_unchecked(index.unchecked_add(1)) }
    }

    fn to_index(self) -> usize {
        // SAFETY: self.get() is >= 1, so subtracting 1 is safe and won't underflow.
        unsafe { self.get().unchecked_sub(1) }
    }

    unsafe fn unchecked_add_usize(self, rhs: usize) -> Self {
        // SAFETY: The caller guarantees the result is representable.
        // self.get() + rhs is >= 1.
        unsafe { Self::new_unchecked(self.get().unchecked_add(rhs)) }
    }

    unsafe fn unchecked_sub(self, rhs: Self) -> usize {
        // SAFETY: The caller guarantees rhs <= self.
        // self.get() - rhs.get() is >= 0.
        unsafe { self.get().unchecked_sub(rhs.get()) }
    }
}

macro_rules! impl_for_nonzero_uint_type {
    {$t: ty} => {
        // SAFETY: These types represent [0, max-1] by mapping to [1, max].
        unsafe impl IndexType for $t {
            const ZERO: Self = unsafe {
                // SAFETY: 1 is non-zero.
                Self::new_unchecked(1)
            };

            fn try_from_index(index: usize) -> Result<Self, IndexTooBigError> {
                let raw = index
                    .checked_add(1)
                    .ok_or(IndexTooBigError)?
                    .try_into()
                    .map_err(|_| IndexTooBigError)?;
                // SAFETY: raw is index + 1, and index >= 0, so raw >= 1.
                Ok(unsafe { Self::new_unchecked(raw) })
            }

            unsafe fn from_index_unchecked(index: usize) -> Self {
                // SAFETY: The caller guarantees index is representable.
                // For $t, it means index+1 fits in $t.
                // index+1 will be >= 1.
                unsafe { Self::new_unchecked(index.unchecked_add(1) as _) }
            }

            fn to_index(self) -> usize {
                // SAFETY: self.get() is >= 1, so subtracting 1 is safe and won't underflow.
                unsafe { (self.get().unchecked_sub(1)) as usize }
            }

            unsafe fn unchecked_add_usize(self, rhs: usize) -> Self {
                // SAFETY: The caller guarantees the result is representable.
                // self.get() + rhs is >= 1 and fits in $t.
                unsafe { Self::new_unchecked(self.get().unchecked_add(rhs as _)) }
            }

            unsafe fn unchecked_sub(self, rhs: Self) -> usize {
                // SAFETY: The caller guarantees rhs <= self.
                // self.get() - rhs.get() is >= 0.
                unsafe { (self.get().unchecked_sub(rhs.get())) as usize }
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
