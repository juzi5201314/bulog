use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{Data, DeriveInput, Fields};

#[proc_macro_derive(Optional)]
pub fn optional(input: TokenStream) -> TokenStream {
    let ast = syn::parse_macro_input!(input as DeriveInput);
    let name = format_ident!("{}Option", &ast.ident);
    let fields = match ast.data {
        Data::Struct(ref data) => match data.fields {
            Fields::Named(ref fields) => &fields.named,
            Fields::Unnamed(ref fields) => &fields.unnamed,
            Fields::Unit => panic!("Cannot apply MakeOptional to a unit struct."),
        },
        _ => panic!("MakeOptional can only be used with structs."),
    };

    let optional_fields: Vec<_> = fields
        .iter()
        .map(|f| {
            let field_name = &f.ident;
            let field_type = &f.ty;
            quote! { #field_name: Option<#field_type> }
        })
        .collect();

    (quote! {
        #[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
        pub struct #name {
            #(#[serde(skip_serializing_if = "Option::is_none")] pub #optional_fields),*
        }
    })
    .into()
}
