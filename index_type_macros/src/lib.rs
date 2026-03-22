use quote::quote_spanned;
use syn::{DeriveInput, LitStr, parse_macro_input};

fn is_empty_struct(derive_input: &DeriveInput) -> bool {
    match &derive_input.data {
        syn::Data::Struct(data_struct) => match data_struct.fields {
            syn::Fields::Named(_) | syn::Fields::Unnamed(_) => false,
            syn::Fields::Unit => true,
        },
        syn::Data::Enum(_) | syn::Data::Union(_) => false,
    }
}

#[proc_macro_derive(IndexTooBigError)]
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

    let error_msg = match parse_error_attr(&derive_input) {
        Ok(v) => v,
        Err(e) => return e.to_compile_error().into(),
    };

    let ident = &derive_input.ident;

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
                write!(f, #error_msg)
            }
        }

        #[automatically_derived]
        impl ::core::error::Error for #ident {}
    }
    .into()
}

fn parse_error_attr(input: &DeriveInput) -> syn::Result<LitStr> {
    for attr in &input.attrs {
        if attr.path().is_ident("error") {
            return attr.parse_args::<LitStr>();
        }
    }

    Err(syn::Error::new_spanned(
        &input.ident,
        "expected #[error = \"...\"]",
    ))
}
