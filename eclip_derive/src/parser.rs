use proc_macro2::TokenStream;
use quote::quote;

use crate::argument::ArgumentMeta;
use crate::option::OptionMeta;

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

    fn add_option(&mut self, idx: &syn::Index, name: &str, _ty: &syn::Type, meta: &OptionMeta) {
        if let Some(default) = &meta.default {
            self.inits.push(quote!(#default));
        } else {
            self.inits.push(quote!(Default::default()));
        }
        self.setter.push(quote!(arguments.#idx));

        let mut conditions = Vec::new();
        if let Some(short_key) = meta.short_key() {
            conditions.push(quote!(val == #short_key));
        }
        if let Some(long_key) = meta.long_key(name) {
            conditions.push(quote!(val == #long_key));
        }
        if conditions.is_empty() {
            let key = format!("--{}", name);
            conditions.push(quote!(val == #key));
        }
        self.opts.push(quote!(
            if #(#conditions)||* {
                arguments.#idx = eclip::Validator::validate(
                    arguments.#idx, eclip::ArgValue::Option(val), &mut args
                );
            }
        ));
    }

    fn add_argument(
        &mut self,
        idx: &syn::Index,
        _name: &str,
        _ty: &syn::Type,
        _meta: &ArgumentMeta,
    ) {
        let arg_idx = &self.arg_idx;
        self.inits.push(quote!(None));
        self.setter.push(quote!({
            match arguments.#idx {
                Some(val) => val,
                Nome => {
                    eprintln!("More arguments are needed.");
                    std::process::exit(128);
                }
            }
        }));
        self.args.push(quote!(
            if cnt == #arg_idx {
                arguments.#idx = eclip::Validator::validate(
                    arguments.#idx, eclip::ArgValue::Argument(val), &mut args
                );
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
    arg_helps: Vec<TokenStream>,
    opt_helps: Vec<TokenStream>,
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
        let name = format!("<{}>", name);
        self.arg_helps.push(match &meta.help {
            Some(help) => {
                quote!({
                    let name = #name;
                    if name.len() >= helper.padding {
                        format!("  {}\n  {:padding$} {}", name, "", #help, padding=helper.padding)
                    } else {
                        format!("  {:<padding$} {}", name, #help, padding=helper.padding)
                    }
                })
            }
            None => quote!(format!("  {}", #name)),
        });
    }

    fn add_opt_help(&mut self, name: &str, meta: &OptionMeta, ty: &syn::Type) {
        let mut keys = Vec::new();
        if let Some(short_key) = meta.short_key() {
            keys.push(short_key);
        }
        if let Some(long_key) = meta.long_key(name) {
            keys.push(long_key);
        }
        if keys.is_empty() {
            keys.push(format!("--{}", name));
        }
        if !matches!(
            ty,
            syn::Type::Path(path) if path.path.segments.first().unwrap().ident == "bool"
        ) {
            keys.push(format!("<{}>", name.to_uppercase()));
        }
        let message = keys.join(" ");
        self.opt_helps.push(match &meta.help {
            Some(help) => {
                quote!({
                    let message = #message;
                    if message.len() >= helper.padding {
                        format!("  {}\n  {:padding$} {}", message, "", #help, padding=helper.padding)
                    } else {
                        format!("  {:<padding$} {}", message, #help, padding=helper.padding)
                    }
                })
            }
            None => quote!(format!("  {}", #message)),
        });
    }

    fn add_argument(&mut self, name: String) {
        self.arguments.push(format!("<{}>", name));
    }

    fn build_default() -> TokenStream {
        quote! (
            println!(
                "USAGE:\n  {} [OPTIONS]\n\nOPTIONS:\n{}",
                helper.command(),
                eclip::help_message(helper.padding)
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
            quote!("")
        } else {
            let arg_helps = &self.arg_helps;
            quote!(format!("\n\nARGS:\n{}", [#(#arg_helps),*].join("\n")))
        };
        let opts_help = if self.opt_helps.is_empty() {
            quote!("\n\nOPTIONS:\n")
        } else {
            let opt_helps = &self.opt_helps;
            quote!(format!("\n\nOPTIONS:\n{}\n", [#(#opt_helps),*].join("\n")))
        };
        quote! (
            println!(
                "USAGE:\n  {} [OPTIONS]{}{}{}{}",
                helper.command(),
                #args,
                #args_help,
                #opts_help,
                eclip::help_message(helper.padding)
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
                help_factory.add_opt_help(&name, &meta, &field.ty);
                break;
            } else if attr_ident == "argument" {
                let meta = ArgumentMeta::from(attr);
                new_factory.add_argument(&idx, &name, &field.ty, &meta);
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
                help_factory.add_opt_help(&name, &meta, &field.ty);
                break;
            } else if attr_ident == "argument" {
                let meta = ArgumentMeta::from(attr);
                new_factory.add_argument(&idx, &name, &field.ty, &meta);
                help_factory.add_arg_help(&name, &meta);
                help_factory.add_argument(name);
                break;
            }
        }
        idx.index += 1;
    }

    (new_factory.build_unnamed_fields(), help_factory.build())
}
