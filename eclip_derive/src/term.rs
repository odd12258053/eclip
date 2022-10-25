use proc_macro2::Ident;
use syn::{Lit, LitStr, Token};

pub enum Term {
    Short(LitStr),
    Long(LitStr),
    Default(Lit),
    Help(LitStr),
}

impl syn::parse::Parse for Term {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let ident: Ident = input.parse()?;
        let _eq_token: Token![=] = input.parse()?;
        if ident == "short" {
            Ok(Term::Short(input.parse()?))
        } else if ident == "long" {
            Ok(Term::Long(input.parse()?))
        } else if ident == "default" {
            Ok(Term::Default(input.parse()?))
        } else if ident == "help" {
            Ok(Term::Help(input.parse()?))
        } else {
            Err(input.error("un support type"))
        }
    }
}
