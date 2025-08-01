use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields, parse_macro_input};

#[proc_macro_derive(FromValueVec)]
pub fn derive_from_value_vec(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let struct_name = input.ident;

    let fields = match input.data {
        Data::Struct(data) => match data.fields {
            Fields::Named(fields_named) => fields_named.named,
            _ => unimplemented!("FromValueVec only supports named fields"),
        },
        _ => unimplemented!("FromValueVec only supports structs"),
    };

    let field_inits = fields.iter().enumerate().map(|(i, field)| {
        let name = &field.ident;
        let idx = syn::Index::from(i);

        quote! {
            #name: serde_json::from_value(value[#idx].clone())
                .expect(&format!("Failed to parse field {}", #idx))
        }
    });

    let expanded = quote! {
        impl From<Vec<serde_json::Value>> for #struct_name {
            fn from(value: Vec<serde_json::Value>) -> Self {
                #struct_name {
                    #(#field_inits),*
                }
            }
        }
    };

    TokenStream::from(expanded)
}
