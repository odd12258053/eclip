use syn::punctuated::Punctuated;
use syn::{LitStr, Token};

use crate::term::Term;

pub struct ArgumentMeta {
    help: Option<LitStr>,
}

impl ArgumentMeta {
    pub fn new() -> Self {
        Self { help: None }
    }

    pub fn help_message(&self, name: &String, padding: usize) -> String {
        if let Some(help) = &self.help {
            if name.len() >= padding {
                format!("  {}\n  {:padding$} {}", name, "", help.value())
            } else {
                format!("  {:<padding$} {}", name, help.value())
            }
        } else {
            format!("  {}", name)
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
