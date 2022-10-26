use proc_macro2::TokenStream;
use quote::quote;

use crate::argument::ArgumentMeta;
use crate::option::OptionMeta;
use crate::PADDING_SIZE;

struct NewFactory {
    inits: Vec<TokenStream>,
    setter: Vec<TokenStream>,
    keys: Vec<TokenStream>,
    args: Vec<TokenStream>,
    opts: Vec<TokenStream>,
    arg_idx: syn::Index,
}

impl NewFactory {
    fn new() -> Self {
        Self {
            inits: Vec::new(),
            setter: Vec::new(),
            keys: Vec::new(),
            args: Vec::new(),
            opts: Vec::new(),
            arg_idx: syn::Index::from(0),
        }
    }

    fn add_key(&mut self, ident: &syn::Ident) {
        self.keys.push(quote!(#ident));
    }

    fn add_option(&mut self, idx: &syn::Index, name: &str, ty: &syn::Type, meta: &OptionMeta) {
        if let Some(default) = &meta.default {
            self.inits.push(quote!(#default));
        } else {
            self.inits.push(quote!(#ty ::default()));
        }
        self.setter.push(quote!(arguments.#idx));

        let mut conditions = Vec::new();
        if let Some(short_key) = meta.short_key() {
            conditions.push(quote!(val == #short_key));
        }
        if let Some(long_key) = meta.long_key() {
            conditions.push(quote!(val == #long_key));
        }
        if conditions.is_empty() {
            let key = format!("--{}", name);
            conditions.push(quote!(val == #key));
        }
        match ty {
            syn::Type::Path(path) if path.path.segments.first().unwrap().ident == "bool" => {
                self.opts.push(quote!(
                    if #(#conditions)||* { arguments.#idx = true; }
                ))
            }
            _ => self.opts.push(quote!(
                if #(#conditions)||* {
                    arguments.#idx = args.next().unwrap().parse().unwrap();
                }
            )),
        }
    }

    fn add_argument(&mut self, idx: &syn::Index, _name: &str, _meta: &ArgumentMeta) {
        let arg_idx = &self.arg_idx;
        self.inits.push(quote!(None));
        self.setter.push(quote!(arguments.#idx.unwrap()));
        self.args.push(quote!(
            if cnt == #arg_idx {
                arguments.#idx = Some(val.parse().unwrap());
                cnt += 1;
            }
        ));
        self.arg_idx.index += 1;
    }

    fn build_default_named_fields() -> TokenStream {
        quote!(Self {})
    }

    fn build_named_fields(self) -> TokenStream {
        let opts = &self.opts;
        let args = &self.args;
        let cond = if !opts.is_empty() && !args.is_empty() {
            quote!(#(#opts) else * else { #(#args) else * })
        } else if opts.is_empty() {
            quote!(#(#args) else * )
        } else {
            quote!(#(#opts) else *)
        };
        let inits = &self.inits;
        let setter: Vec<TokenStream> = self
            .keys
            .iter()
            .zip(self.setter.iter())
            .map(|(i, j)| quote!(#i: #j))
            .collect();
        quote! (
            let mut arguments = ( #(#inits),*, );
            let mut cnt = 0;
            while let Some(val) = args.next() { #cond }
            Self { #(#setter),* }
        )
    }

    fn build_unit() -> TokenStream {
        quote!(Self)
    }

    fn build_default_unnamed_fields() -> TokenStream {
        quote!(Self())
    }

    fn build_unnamed_fields(self) -> TokenStream {
        let opts = &self.opts;
        let args = &self.args;
        let cond = if !opts.is_empty() && !args.is_empty() {
            quote!(#(#opts) else * else { #(#args) else * })
        } else if opts.is_empty() {
            quote!(#(#args) else * )
        } else {
            quote!(#(#opts) else *)
        };
        let inits = &self.inits;
        let setter = &self.setter;
        quote! (
            let mut arguments = ( #(#inits),*, );
            let mut cnt = 0;
            while let Some(val) = args.next() { #cond }
            Self ( #(#setter),* )
        )
    }
}

struct HelpFactory {
    arguments: Vec<String>,
    arg_helps: Vec<String>,
    opt_helps: Vec<String>,
}

impl HelpFactory {
    fn new() -> Self {
        Self {
            arguments: Vec::new(),
            arg_helps: Vec::new(),
            opt_helps: Vec::new(),
        }
    }

    fn add_arg_help(&mut self, name: &str, meta: &ArgumentMeta) {
        self.arg_helps.push(meta.help_message(name, PADDING_SIZE));
    }

    fn add_opt_help(&mut self, name: &str, meta: &OptionMeta) {
        self.opt_helps.push(meta.help_message(name, PADDING_SIZE));
    }

    fn add_argument(&mut self, name: String) {
        self.arguments.push(name);
    }

    fn build_default() -> TokenStream {
        quote! (
            println!(
                "Usage:\n  {} [OPTIONS]\n\nOptions:\n{}",
                helper.command(),
                eclip::help_message(eclip::PADDING_SIZE)
            );
        )
    }

    fn build(&self) -> TokenStream {
        let args = if self.arguments.is_empty() {
            "".to_string()
        } else {
            format!(" {}", self.arguments.join(" "))
        };
        let args_help = if self.arg_helps.is_empty() {
            "".to_string()
        } else {
            format!("\n\nArguments:\n{}", self.arg_helps.join("\n"))
        };
        let opts_help = format!("\n\nOptions:\n{}", self.opt_helps.join("\n"))
            .trim_end()
            .to_string()
            + "\n";
        quote! (
            println!(
                "Usage:\n  {} [OPTIONS]{}{}{}{}",
                helper.command(),
                #args,
                #args_help,
                #opts_help,
                eclip::help_message(eclip::PADDING_SIZE)
            );
        )
    }
}

pub fn parse_named_fields(fields: &syn::FieldsNamed) -> (TokenStream, TokenStream) {
    if fields.named.is_empty() {
        return (
            NewFactory::build_default_named_fields(),
            HelpFactory::build_default(),
        );
    }

    let mut help_factory = HelpFactory::new();
    let mut new_factory = NewFactory::new();
    let mut idx = syn::Index::from(0);

    for field in fields.named.iter() {
        let ident = field.ident.as_ref().unwrap();
        new_factory.add_key(ident);
        let name = ident.to_string();
        for attr in &field.attrs {
            let attr_ident = &attr.path.segments.first().unwrap().ident;
            if attr_ident == "option" {
                let meta = OptionMeta::from(attr);
                new_factory.add_option(&idx, &name, &field.ty, &meta);
                help_factory.add_opt_help(&name, &meta);
                break;
            } else if attr_ident == "argument" {
                let meta = ArgumentMeta::from(attr);
                new_factory.add_argument(&idx, &name, &meta);
                help_factory.add_arg_help(&name, &meta);
                help_factory.add_argument(name);
                break;
            }
        }
        idx.index += 1;
    }

    (new_factory.build_named_fields(), help_factory.build())
}

pub fn parse_unit() -> (TokenStream, TokenStream) {
    (NewFactory::build_unit(), HelpFactory::build_default())
}

pub fn parse_unnamed_fields(fields: &syn::FieldsUnnamed) -> (TokenStream, TokenStream) {
    if fields.unnamed.is_empty() {
        return (
            NewFactory::build_default_unnamed_fields(),
            HelpFactory::build_default(),
        );
    }
    let mut help_factory = HelpFactory::new();
    let mut new_factory = NewFactory::new();
    let mut idx = syn::Index::from(0);

    for field in fields.unnamed.iter() {
        let name = format!("{}", idx.index);
        for attr in &field.attrs {
            let attr_ident = &attr.path.segments.first().unwrap().ident;
            if attr_ident == "option" {
                let meta = OptionMeta::from(attr);
                new_factory.add_option(&idx, &name, &field.ty, &meta);
                help_factory.add_opt_help(&name, &meta);
                break;
            } else if attr_ident == "argument" {
                let meta = ArgumentMeta::from(attr);
                new_factory.add_argument(&idx, &name, &meta);
                help_factory.add_arg_help(&name, &meta);
                help_factory.add_argument(name);
                break;
            }
        }
        idx.index += 1;
    }

    (new_factory.build_unnamed_fields(), help_factory.build())
}
