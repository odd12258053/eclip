use syn::punctuated::Punctuated;
use syn::{Lit, LitStr, Token};

use crate::term::Term;

pub struct OptionMeta {
    pub short: Option<LitStr>,
    pub long: Option<Option<LitStr>>,
    pub default: Option<Lit>,
    pub help: Option<LitStr>,
}

impl OptionMeta {
    pub fn new() -> Self {
        Self {
            short: None,
            long: None,
            default: None,
            help: None,
        }
    }

    pub fn from(attr: &syn::Attribute) -> Self {
        if attr.tokens.is_empty() {
            Self::new()
        } else {
            attr.parse_args().unwrap()
        }
    }

    pub fn short_key(&self) -> Option<String> {
        self.short
            .as_ref()
            .map(|short| format!("-{}", short.value()))
    }

    pub fn long_key(&self, name: &str) -> Option<String> {
        self.long.as_ref().map(|long| match long {
            Some(long) => format!("--{}", long.value()),
            None => format!("--{}", name),
        })
    }
}

impl syn::parse::Parse for OptionMeta {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let terms: Punctuated<Term, Token![,]> = Punctuated::parse_terminated(input)?;
        let mut meta = OptionMeta::new();
        for term in terms.into_iter() {
            match term {
                Term::Short(lit) => {
                    meta.short = Some(lit);
                }
                Term::Long(lit) => {
                    meta.long = Some(lit);
                }
                Term::Default(lit) => {
                    meta.default = Some(lit);
                }
                Term::Help(lit) => {
                    meta.help = Some(lit);
                }
            }
        }
        Ok(meta)
    }
}
