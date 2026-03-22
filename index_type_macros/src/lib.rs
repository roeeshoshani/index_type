use darling::FromDeriveInput;
use quote::quote_spanned;
use syn::{Attribute, DeriveInput, LitStr, parse_macro_input};

fn is_empty_struct(derive_input: &DeriveInput) -> bool {
    match &derive_input.data {
        syn::Data::Struct(data_struct) => match data_struct.fields {
            syn::Fields::Named(_) | syn::Fields::Unnamed(_) => false,
            syn::Fields::Unit => true,
        },
        syn::Data::Enum(_) | syn::Data::Union(_) => false,
    }
}

fn as_newtype_struct(derive_input: &DeriveInput) -> Option<&syn::Field> {
    match &derive_input.data {
        syn::Data::Struct(data_struct) => match &data_struct.fields {
            syn::Fields::Named(_) | syn::Fields::Unit => None,
            syn::Fields::Unnamed(fields_unnamed) => {
                if fields_unnamed.unnamed.len() == 1 {
                    Some(&fields_unnamed.unnamed[0])
                } else {
                    None
                }
            }
        },
        syn::Data::Enum(_) | syn::Data::Union(_) => None,
    }
}

#[derive(FromDeriveInput)]
#[darling(attributes(index_type))]
struct IndexTypeArgs {
    error: syn::Type,
}

#[proc_macro_derive(IndexType, attributes(index_type))]
pub fn derive_index_type(input_tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let derive_input = parse_macro_input!(input_tokens as DeriveInput);

    let Some(field) = as_newtype_struct(&derive_input) else {
        return quote_spanned! {
            proc_macro2::Span::call_site() => {
                compile_error!("only structs with a single unnamed field are supported (e.g `struct Foo(u32);`)")
            };
        }
        .into();
    };

    let args = match IndexTypeArgs::from_derive_input(&derive_input) {
        Ok(v) => v,
        Err(e) => return e.write_errors().into(),
    };

    let ident = &derive_input.ident;
    let err_ty = &args.error;
    let inner_ty = &field.ty;

    quote::quote! {
        #[automatically_generated]
        unsafe impl ::index_type::IndexType for #ident {
            type IndexTooBigError = #err_ty;

            type Scalar = <#inner_ty as ::index_type::IndexType>::Scalar;

            const ZERO: Self = Self(<#inner_ty as ::index_type::IndexType>::ZERO);

            const MAX_RAW_INDEX: usize = <#inner_ty as ::index_type::IndexType>::MAX_RAW_INDEX;

            fn try_from_raw_index(index: usize) -> Result<Self, Self::IndexTooBigError> {
                <#inner_ty as ::index_type::IndexType>::try_from_raw_index(index)
                    .map(Self)
                    .map_err(|_| <#err_ty as ::index_type::IndexTooBigError>::new())
            }

            unsafe fn from_raw_index_unchecked(index: usize) -> Self {
                Self(unsafe { <#inner_ty as ::index_type::IndexType>::from_raw_index_unchecked(index) })
            }

            fn to_raw_index(self) -> usize {
                <#inner_ty as ::index_type::IndexType>::to_raw_index(self.0)
            }

            fn try_from_scalar(scalar: Self::Scalar) -> Result<Self, Self::IndexTooBigError> {
                <#inner_ty as ::index_type::IndexType>::try_from_scalar(scalar)
                    .map(Self)
                    .map_err(|_| <#err_ty as ::index_type::IndexTooBigError>::new())
            }

            unsafe fn from_scalar_unchecked(scalar: Self::Scalar) -> Self {
                Self(unsafe { <#inner_ty as ::index_type::IndexType>::from_scalar_unchecked(scalar) })
            }

            fn to_scalar(self) -> Self::Scalar {
                <#inner_ty as ::index_type::IndexType>::to_scalar(self.0)
            }

            fn checked_add_scalar(self, rhs: Self::Scalar) -> Result<Self, Self::IndexTooBigError> {
                <#inner_ty as ::index_type::IndexType>::checked_add_scalar(self.0, rhs)
                    .map(Self)
                    .map_err(|_| <#err_ty as ::index_type::IndexTooBigError>::new())
            }

            fn checked_mul_scalar(self, rhs: Self::Scalar) -> Result<Self, Self::IndexTooBigError> {
                <#inner_ty as ::index_type::IndexType>::checked_mul_scalar(self.0, rhs)
                    .map(Self)
                    .map_err(|_| <#err_ty as ::index_type::IndexTooBigError>::new())
            }

            unsafe fn unchecked_add_scalar(self, rhs: Self::Scalar) -> Self {
                Self(unsafe { <#inner_ty as ::index_type::IndexType>::unchecked_add_scalar(self.0, rhs) })
            }

            unsafe fn unchecked_sub_index(self, rhs: Self) -> Self::Scalar {
                unsafe { <#inner_ty as ::index_type::IndexType>::unchecked_sub_index(self.0, rhs.0) }
            }
        }
    }
    .into()
}

#[derive(FromDeriveInput)]
#[darling(attributes(error))]
struct IndexTooBigErrorArgs {
    msg: String,
}

#[proc_macro_derive(IndexTooBigError, attributes(error))]
pub fn derive_index_too_big_error(
    input_tokens: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let derive_input = parse_macro_input!(input_tokens as DeriveInput);

    if !is_empty_struct(&derive_input) {
        return quote_spanned! {
            proc_macro2::Span::call_site() => compile_error!("only empty structs are supported (e.g `struct Foo;`)");
        }
        .into();
    }

    let args = match IndexTooBigErrorArgs::from_derive_input(&derive_input) {
        Ok(v) => v,
        Err(e) => return e.write_errors().into(),
    };

    let ident = &derive_input.ident;
    let err_msg = &args.msg;

    quote::quote! {
        #[automatically_derived]
        impl ::index_type::IndexTooBigError for #ident {
            fn new() -> Self {
                Self
            }
        }

        #[automatically_derived]
        impl ::core::fmt::Display for #ident {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                write!(f, #err_msg)
            }
        }

        #[automatically_derived]
        impl ::core::error::Error for #ident {}
    }
    .into()
}
