use syn::punctuated::Punctuated;
use syn::{Lit, LitStr, Token};

use crate::term::Term;

pub struct OptionMeta {
    pub short: Option<LitStr>,
    pub long: Option<LitStr>,
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

    pub fn help_message(&self, name: &String, padding: usize) -> String {
        let mut keys = Vec::new();
        if let Some(short) = &self.short {
            keys.push(format!("-{}", short.value()));
        }
        if let Some(long) = &self.long {
            keys.push(format!("--{}", long.value()));
        }
        if keys.is_empty() {
            keys.push(format!("--{}", name));
        }
        let message = keys.join(" ");
        if let Some(help) = &self.help {
            if message.len() >= padding {
                format!("  {}\n  {:padding$} {}", message, "", help.value())
            } else {
                format!("  {:<padding$} {}", message, help.value())
            }
        } else {
            format!("  {}", message)
        }
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
