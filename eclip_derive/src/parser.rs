use quote::quote;

use crate::argument::ArgumentMeta;
use crate::option::OptionMeta;
use crate::PADDING_SIZE;

pub fn parse_named_fields(
    fields: &syn::FieldsNamed,
) -> (proc_macro2::TokenStream, proc_macro2::TokenStream) {
    if fields.named.is_empty() {
        return (
            quote!(Self {}),
            quote! (
                println!(
                    "Usage:\n  {} [OPTIONS]\n\nOptions:\n{}",
                    helper.command(),
                    eclip::help_message(eclip::PADDING_SIZE)
                );
            ),
        );
    }

    let mut arguments: Vec<String> = Vec::new();
    let mut argument_helps: Vec<String> = Vec::new();
    let mut option_helps: Vec<String> = Vec::new();

    let mut inits = Vec::new();
    let mut setter = Vec::new();
    let mut arg = Vec::new();
    let mut opts = Vec::new();
    let mut arg_idx = syn::Index::from(0);
    let mut idx = syn::Index::from(0);

    for field in fields.named.iter() {
        let ident = &field.ident;
        let ty = &field.ty;
        let name = field.ident.as_ref().unwrap().to_string();
        for attr in &field.attrs {
            let attr_ident = &attr.path.segments.first().unwrap().ident;
            if attr_ident == "option" {
                let meta = if attr.tokens.is_empty() {
                    OptionMeta::new()
                } else {
                    attr.parse_args().unwrap()
                };
                if let Some(default) = &meta.default {
                    inits.push(quote!(#default));
                } else {
                    inits.push(quote!(#ty ::default()));
                }
                setter.push(quote!(#ident: arguments.#idx));

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
                    syn::Type::Path(path)
                        if path.path.segments.first().unwrap().ident == "bool" =>
                    {
                        opts.push(quote!(
                            if #(#conditions)||* { arguments.#idx = true; }
                        ))
                    }
                    _ => opts.push(quote!(
                        if #(#conditions)||* {
                            arguments.#idx = args.next().unwrap().parse().unwrap();
                        }
                    )),
                }
                option_helps.push(meta.help_message(&name, PADDING_SIZE));
                break;
            } else if attr_ident == "argument" {
                arguments.push(name.to_string());
                let meta = if attr.tokens.is_empty() {
                    ArgumentMeta::new()
                } else {
                    attr.parse_args().unwrap()
                };
                inits.push(quote!(None));
                setter.push(quote!(#ident: arguments.#idx.unwrap()));
                arg.push(quote!(
                    if cnt == #arg_idx {
                        arguments.#idx = Some(val.parse().unwrap());
                        cnt += 1;
                    }
                ));
                arg_idx.index += 1;
                argument_helps.push(meta.help_message(&name, PADDING_SIZE));
                break;
            }
        }
        idx.index += 1;
    }
    let args = if arguments.is_empty() {
        "".to_string()
    } else {
        format!(" {}", arguments.join(" "))
    };

    let args_help = if argument_helps.is_empty() {
        "".to_string()
    } else {
        format!("\n\nArguments:\n{}", argument_helps.join("\n"))
    };
    let opts_help = format!("\n\nOptions:\n{}", option_helps.join("\n"))
        .trim_end()
        .to_string()
        + "\n";

    let cond = if !opts.is_empty() && !arg.is_empty() {
        quote!(#(#opts) else * else { #(#arg) else * })
    } else if opts.is_empty() {
        quote!(#(#arg) else * )
    } else {
        quote!(#(#opts) else *)
    };

    (
        quote! (
            let mut arguments = ( #(#inits),*, );
            let mut cnt = 0;
            while let Some(val) = args.next() { #cond }
            Self { #(#setter),* }
        ),
        quote! (
            println!(
                "Usage:\n  {} [OPTIONS]{}{}{}{}",
                helper.command(),
                #args,
                #args_help,
                #opts_help,
                eclip::help_message(eclip::PADDING_SIZE)
            );
        ),
    )
}

pub fn parse_unit() -> (proc_macro2::TokenStream, proc_macro2::TokenStream) {
    (
        quote! {
            Self
        },
        quote! {
            println!(
                "Usage:\n  {} [OPTIONS]\n\nOptions:\n{}",
                helper.command(),
                eclip::help_message(eclip::PADDING_SIZE)
            );
        },
    )
}

pub fn parse_unnamed_fields(
    fields: &syn::FieldsUnnamed,
) -> (proc_macro2::TokenStream, proc_macro2::TokenStream) {
    if fields.unnamed.is_empty() {
        return (
            quote!(Self()),
            quote! (
                println!(
                    "Usage:\n  {} [OPTIONS]\n\nOptions:\n{}",
                    helper.command(),
                    eclip::help_message(eclip::PADDING_SIZE)
                );
            ),
        );
    }
    let mut arguments: Vec<String> = Vec::new();
    let mut argument_helps: Vec<String> = Vec::new();
    let mut option_helps: Vec<String> = Vec::new();

    let mut inits = Vec::new();
    let mut setter = Vec::new();
    let mut arg = Vec::new();
    let mut opts = Vec::new();
    let mut arg_idx = syn::Index::from(0);
    let mut idx = syn::Index::from(0);

    for field in fields.unnamed.iter() {
        let ty = &field.ty;
        let name = format!("{}", idx.index);
        for attr in &field.attrs {
            let attr_ident = &attr.path.segments.first().unwrap().ident;
            if attr_ident == "option" {
                let meta = if attr.tokens.is_empty() {
                    OptionMeta::new()
                } else {
                    attr.parse_args().unwrap()
                };
                if let Some(default) = &meta.default {
                    inits.push(quote!(#default));
                } else {
                    inits.push(quote!(#ty ::default()));
                }
                setter.push(quote!(arguments.#idx));

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
                    syn::Type::Path(path)
                        if path.path.segments.first().unwrap().ident == "bool" =>
                    {
                        opts.push(quote!(
                            if #(#conditions)||* { arguments.#idx = true; }
                        ))
                    }
                    _ => opts.push(quote!(
                        if #(#conditions)||* {
                            arguments.#idx = args.next().unwrap().parse().unwrap();
                        }
                    )),
                }
                option_helps.push(meta.help_message(&name, PADDING_SIZE));
                break;
            } else if attr_ident == "argument" {
                arguments.push(name.to_string());
                let meta = if attr.tokens.is_empty() {
                    ArgumentMeta::new()
                } else {
                    attr.parse_args().unwrap()
                };
                inits.push(quote!(None));
                setter.push(quote!(arguments.#idx.unwrap()));
                arg.push(quote!(
                    if cnt == #arg_idx {
                        arguments.#idx = Some(val.parse().unwrap());
                        cnt += 1;
                    }
                ));
                arg_idx.index += 1;
                argument_helps.push(meta.help_message(&name, PADDING_SIZE));
                break;
            }
        }
        idx.index += 1;
    }
    let args = if arguments.is_empty() {
        "".to_string()
    } else {
        format!(" {}", arguments.join(" "))
    };

    let args_help = if argument_helps.is_empty() {
        "".to_string()
    } else {
        format!("\n\nArguments:\n{}", argument_helps.join("\n"))
    };

    let opts_help = format!("\n\nOptions:\n{}", option_helps.join("\n"))
        .trim_end()
        .to_string()
        + "\n";
    (
        quote! (
            let mut arguments = ( #(#inits),*, );
            let mut cnt = 0;
            while let Some(val) = args.next() {
                #(#opts) else * else { #(#arg) else * }
            }
            Self ( #(#setter),* )
        ),
        quote! (
            println!(
                "Usage:\n  {} [OPTIONS]{}{}{}\n{}",
                helper.command(),
                #args,
                #args_help,
                #opts_help,
                eclip::help_message(eclip::PADDING_SIZE)
            );
        ),
    )
}
