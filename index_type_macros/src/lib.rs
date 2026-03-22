use quote::quote_spanned;
use syn::{DeriveInput, parse_macro_input};

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

    let ident = &derive_input.ident;

    quote::quote! {
        #[automatically_derived]
        impl ::index_type::IndexTooBigError for #ident {
            fn new() -> Self {
                Self
            }
        }
    }
    .into()
}
