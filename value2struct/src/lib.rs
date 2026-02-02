use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields, parse_macro_input};

#[proc_macro_derive(FromValueVec)]
pub fn derive_from_value_vec(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let struct_name = input.ident;

    let field_inits = match input.data {
        Data::Struct(data) => match data.fields {
            Fields::Named(fields_named) => {
                let inits = fields_named.named.iter().enumerate().map(|(i, field)| {
                    let name = &field.ident;
                    let idx = syn::Index::from(i);
                    quote! {
                        #name: serde_json::from_value(value[#idx].clone())
                            .expect(&format!("Failed to parse field {}", #idx))
                    }
                });
                quote! { { #(#inits),* } }
            }
            Fields::Unnamed(fields_unnamed) => {
                let inits = fields_unnamed.unnamed.iter().enumerate().map(|(i, _)| {
                    let idx = syn::Index::from(i);
                    quote! {
                        serde_json::from_value(value[#idx].clone())
                            .expect(&format!("Failed to parse field {}", #idx))
                    }
                });
                quote! { ( #(#inits),* ) }
            }
            _ => unimplemented!("FromValueVec only supports named or unnamed fields"),
        },
        _ => unimplemented!("FromValueVec only supports structs"),
    };

    let expanded = quote! {
        impl From<Vec<serde_json::Value>> for #struct_name {
            fn from(value: Vec<serde_json::Value>) -> Self {
                #struct_name #field_inits
            }
        }
    };

    TokenStream::from(expanded)
}
