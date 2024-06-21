extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields};

#[proc_macro_derive(UnwrapLazy)]
pub fn unwrap_lazy_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = &input.ident;
    let generics = &input.generics;
    let unwrap_lazy_impl = generate_unwrap_lazy_impl(&input.data, name);

    let expanded = quote! {
        const _: () = {
            use rmp_serde::decode::Error as UnwrapLazyError;
            #[automatically_derived]
            impl #generics UnwrapLazy for #name #generics {
                fn try_unwrap_lazy(self) -> Result<Self, UnwrapLazyError> {
                    Ok(#unwrap_lazy_impl)
                }
            }
        };
    };

    TokenStream::from(expanded)
}

fn generate_unwrap_lazy_impl(data: &Data, name: &syn::Ident) -> proc_macro2::TokenStream {
    match data {
        Data::Struct(data_struct) => {
            let fields = match &data_struct.fields {
                Fields::Named(fields) => &fields.named,
                _ => panic!("UnwrapLazy can only be derived for structs with named fields"),
            };

            let field_unwraps = fields.iter().map(|field| {
                let field_name = &field.ident;
                quote! {
                    #field_name: self.#field_name.try_unwrap_lazy()?,
                }
            });

            quote! {
                #name {
                    #(#field_unwraps)*
                }
            }
        }
        _ => panic!("UnwrapLazy can only be derived for structs"),
    }
}
