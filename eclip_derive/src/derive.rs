use quote::quote;
use syn::DeriveInput;

use crate::parser::{parse_named_fields, parse_unit, parse_unnamed_fields};

pub fn derive_command(input: &DeriveInput) -> proc_macro2::TokenStream {
    let target = &input.ident;
    let (new_token, help_token) = match &input.data {
        syn::Data::Struct(data) => match &data.fields {
            syn::Fields::Named(fields) => parse_named_fields(fields),
            syn::Fields::Unnamed(fields) => parse_unnamed_fields(fields),
            syn::Fields::Unit => parse_unit(),
        },
        syn::Data::Union(_data) => panic!("Unsupported type"),
        syn::Data::Enum(_) => panic!("Unsupported type"),
    };

    quote! {
        impl eclip::Help for #target {
            fn help(helper: eclip::Helper) { #help_token }
        }
        impl eclip::ArgsNew for #target {
            fn new(mut args: std::env::Args) -> Self { #new_token }
        }
    }
}
