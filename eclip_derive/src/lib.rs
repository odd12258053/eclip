use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

const PADDING_SIZE: usize = 30;

mod argument;
mod derive;
mod option;
mod parser;
mod term;

use crate::derive::derive_command;

#[proc_macro_derive(Command, attributes(option, argument))]
pub fn command_macro_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    derive_command(&input).into()
}
