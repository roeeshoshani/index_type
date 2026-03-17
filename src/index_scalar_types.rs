use crate::IndexScalarType;

macro_rules! impl_for_uint_type {
    {$t: ty} => {
        impl crate::index_scalar_type_private::Sealed for $t {}
        unsafe impl IndexScalarType for $t {
            const ZERO: Self = 0;
            const ONE: Self = 1;
        }
    };
}
impl_for_uint_type! {u8}
impl_for_uint_type! {u16}
impl_for_uint_type! {u32}
impl_for_uint_type! {u64}
impl_for_uint_type! {usize}
