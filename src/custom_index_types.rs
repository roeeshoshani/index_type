#[macro_export]
macro_rules! impl_index_type {
    ($ty: ty, inner = $inner_ty: ty, err = $err_ty: ty) => {
        unsafe impl $crate::IndexType for $ty {
            type IndexTooBigError = $err_ty;

            type Scalar = <$inner_ty as $crate::IndexType>::Scalar;

            const ZERO: Self = Self(<$inner_ty as $crate::IndexType>::ZERO);

            const MAX_RAW_INDEX: usize = <$inner_ty as $crate::IndexType>::MAX_RAW_INDEX;

            fn try_from_raw_index(index: usize) -> Result<Self, Self::IndexTooBigError> {
                <$inner_ty as $crate::IndexType>::try_from_raw_index(index)
                    .map(Self)
                    .map_err(|_| <$err_ty as $crate::IndexTooBigError>::new())
            }

            unsafe fn from_raw_index_unchecked(index: usize) -> Self {
                Self(unsafe { <$inner_ty as $crate::IndexType>::from_raw_index_unchecked(index) })
            }

            fn to_raw_index(self) -> usize {
                <$inner_ty as $crate::IndexType>::to_raw_index(self.0)
            }

            fn try_from_scalar(scalar: Self::Scalar) -> Result<Self, Self::IndexTooBigError> {
                <$inner_ty as $crate::IndexType>::try_from_scalar(scalar)
                    .map(Self)
                    .map_err(|_| <$err_ty as $crate::IndexTooBigError>::new())
            }

            unsafe fn from_scalar_unchecked(scalar: Self::Scalar) -> Self {
                Self(unsafe { <$inner_ty as $crate::IndexType>::from_scalar_unchecked(scalar) })
            }

            fn to_scalar(self) -> Self::Scalar {
                <$inner_ty as $crate::IndexType>::to_scalar(self.0)
            }

            fn checked_add_scalar(self, rhs: Self::Scalar) -> Result<Self, Self::IndexTooBigError> {
                <$inner_ty as $crate::IndexType>::checked_add_scalar(self.0, rhs)
                    .map(Self)
                    .map_err(|_| <$err_ty as $crate::IndexTooBigError>::new())
            }

            fn checked_mul_scalar(self, rhs: Self::Scalar) -> Result<Self, Self::IndexTooBigError> {
                <$inner_ty as $crate::IndexType>::checked_mul_scalar(self.0, rhs)
                    .map(Self)
                    .map_err(|_| <$err_ty as $crate::IndexTooBigError>::new())
            }

            unsafe fn unchecked_add_scalar(self, rhs: Self::Scalar) -> Self {
                Self(unsafe { <$inner_ty as $crate::IndexType>::unchecked_add_scalar(self.0, rhs) })
            }

            unsafe fn unchecked_sub_index(self, rhs: Self) -> Self::Scalar {
                unsafe { <$inner_ty as $crate::IndexType>::unchecked_sub_index(self.0, rhs.0) }
            }
        }
    };
}

#[macro_export]
macro_rules! define_index_type {
    {
        $vis:vis struct $ident: ident($inner_ty: ty);
        err = $err_ty: ty;
    } => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        $vis struct $ident(pub $inner_ty);

        $crate::impl_index_type!($ident, inner = $inner_ty, err = $err_ty);
    };
}

#[macro_export]
macro_rules! impl_index_too_big_error {
    ($ident: ident = $err_msg:literal ) => {
        impl core::fmt::Display for $ident {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                write!(f, $err_msg)
            }
        }

        impl core::error::Error for $ident {}

        impl $crate::IndexTooBigError for $ident {
            fn new() -> Self {
                $ident
            }
        }
    };
}

#[macro_export]
macro_rules! define_index_too_big_error {
    ($vis:vis struct $ident: ident = $err_msg:literal ) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
        $vis struct $ident;

        $crate::impl_index_too_big_error!($ident = $err_msg );
    };
}

#[macro_export]
macro_rules! define_index_and_error_type {
    {
        $vis:vis struct $ident: ident($inner_ty: ty);
        $err_vis:vis struct $err_ident: ident = $err_msg:literal ;
    } => {
        $crate::define_index_too_big_error!($vis struct $err_ident = $err_msg);
        $crate::define_index_type!(
            $err_vis struct $ident($inner_ty);
            err = $err_ident;
        );
    };
}

define_index_and_error_type!(
    pub struct FooId(core::num::NonZeroU32);
    pub struct FooIdTooBigError = "foo id too big";
);

define_index_type!(
    pub struct FooId2(core::num::NonZeroU32);
    err = FooIdTooBigError;
);
