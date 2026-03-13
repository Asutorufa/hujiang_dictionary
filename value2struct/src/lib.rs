use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields, parse_macro_input};

#[proc_macro_derive(FromValueVec)]
pub fn derive_from_value_vec(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let struct_name = input.ident;

    let (field_inits, field_count) = match input.data {
        Data::Struct(data) => match data.fields {
            Fields::Named(fields_named) => {
                let count = fields_named.named.len();
                let inits = fields_named.named.iter().enumerate().map(|(i, field)| {
                    let name = &field.ident;
                    let idx = syn::Index::from(i);
                    quote! {
                        #name: serde_json::from_value(value[#idx].clone())
                            .map_err(|e| format!("Failed to parse field '{}' at index {}: {}", stringify!(#name), #idx, e))?
                    }
                });
                (quote! { { #(#inits),* } }, count)
            }
            Fields::Unnamed(fields_unnamed) => {
                let count = fields_unnamed.unnamed.len();
                let inits = fields_unnamed.unnamed.iter().enumerate().map(|(i, _)| {
                    let idx = syn::Index::from(i);
                    quote! {
                        serde_json::from_value(value[#idx].clone())
                            .map_err(|e| format!("Failed to parse field at index {}: {}", #idx, e))?
                    }
                });
                (quote! { ( #(#inits),* ) }, count)
            }
            _ => unimplemented!("FromValueVec only supports named or unnamed fields"),
        },
        _ => unimplemented!("FromValueVec only supports structs"),
    };

    let expanded = quote! {
        impl std::convert::TryFrom<Vec<serde_json::Value>> for #struct_name {
            type Error = String;

            fn try_from(value: Vec<serde_json::Value>) -> Result<Self, Self::Error> {
                if value.len() != #field_count {
                    return Err(format!(
                        "Incorrect number of values for struct {}: expected {}, got {}",
                        stringify!(#struct_name),
                        #field_count,
                        value.len()
                    ));
                }
                Ok(Self #field_inits)
            }
        }
    };

    TokenStream::from(expanded)
}
