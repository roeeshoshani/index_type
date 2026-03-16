use alloc::vec::Vec;
use uniq::Unique;

use crate::{IndexTooBigError, IndexType};

pub struct TypedVec<I: IndexType, T> {
    ptr: Unique<T>,
    len: I,
    cap: I,
}
impl<I: IndexType, T> TypedVec<I, T> {
    pub const fn new() -> Self {
        Self {
            ptr: Unique::dangling(),
            len: I::ZERO,
            cap: I::ZERO,
        }
    }

    pub fn into_vec(self) -> Vec<T> {
        unsafe { Vec::from_raw_parts(self.ptr.as_ptr(), self.len.to_index(), self.cap.to_index()) }
    }

    pub fn modify_as_vec<F, R>(&mut self, f: F) -> Result<R, IndexTooBigError>
    where
        F: FnOnce(&mut Vec<T>) -> R,
    {
        let mut vec = core::mem::take(self).into_vec();
        let res = f(&mut vec);
        let (new_ptr, new_len, new_cap) = vec.into_raw_parts();
        *self = TypedVec {
            ptr: unsafe {
                // SAFETY: the pointer of a vec is never null. it is stored internally as a non-null pointer.
                Unique::new_unchecked(new_ptr)
            },
            len: I::try_from_index(new_len)?,
            cap: I::try_from_index(new_cap)?,
        };
        Ok(res)
    }

    pub fn push(&mut self, value: T) -> Result<I, IndexTooBigError> {
        let res = self.len;
        self.modify_as_vec(|v| {
            v.push(value);
        })?;
        Ok(res)
    }

    pub fn append(&mut self, other: &mut TypedVec<I, T>) -> Result<(), IndexTooBigError> {
        self.modify_as_vec(|self_vec| {
            other.modify_as_vec(|other_vec| {
                self_vec.append(other_vec);
            })
        })?
    }

    pub fn as_mut_ptr(&mut self) -> *mut T {
        self.ptr.as_ptr()
    }
}
impl<I: IndexType, T> Drop for TypedVec<I, T> {
    fn drop(&mut self) {
        let _ = unsafe {
            Vec::from_raw_parts(self.ptr.as_ptr(), self.len.to_index(), self.cap.to_index())
        };
    }
}
impl<I: IndexType, T> Default for TypedVec<I, T> {
    fn default() -> Self {
        Self::new()
    }
}
