use syn::punctuated::Punctuated;
use syn::{LitStr, Token};

use crate::term::Term;

pub struct ArgumentMeta {
    pub help: Option<LitStr>,
}

impl ArgumentMeta {
    pub fn new() -> Self {
        Self { help: None }
    }

    pub fn from(attr: &syn::Attribute) -> Self {
        if attr.tokens.is_empty() {
            Self::new()
        } else {
            attr.parse_args().unwrap()
        }
    }
}

impl syn::parse::Parse for ArgumentMeta {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let terms: Punctuated<Term, Token![,]> = Punctuated::parse_terminated(input)?;
        let mut meta = ArgumentMeta::new();
        for term in terms.into_iter() {
            match term {
                Term::Help(lit) => {
                    meta.help = Some(lit);
                }
                _ => return Err(input.error("un support type")),
            }
        }
        Ok(meta)
    }
}
