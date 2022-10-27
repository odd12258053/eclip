use proc_macro2::Ident;
use syn::{Lit, LitStr, Token};

pub enum Term {
    Short(LitStr),
    Long(Option<LitStr>),
    Default(Lit),
    Help(LitStr),
}

impl syn::parse::Parse for Term {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let ident: Ident = input.parse()?;
        if ident == "short" {
            input.parse::<Token![=]>()?;
            Ok(Term::Short(input.parse()?))
        } else if ident == "long" {
            if input.parse::<Token![=]>().is_ok() {
                Ok(Term::Long(Some(input.parse()?)))
            } else {
                Ok(Term::Long(None))
            }
        } else if ident == "default" {
            input.parse::<Token![=]>()?;
            Ok(Term::Default(input.parse()?))
        } else if ident == "help" {
            input.parse::<Token![=]>()?;
            Ok(Term::Help(input.parse()?))
        } else {
            Err(input.error("Unsupported type"))
        }
    }
}
