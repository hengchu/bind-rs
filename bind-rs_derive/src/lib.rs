use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{
    parenthesized,
    parse::{Parse, Parser},
    parse_macro_input,
    spanned::Spanned,
    Data, DeriveInput, Error, FnArg,
    FnArg::{Receiver, Typed},
    Ident, ImplItemMethod, Pat, Token, Type,
};

use syn::parse::ParseStream;

#[proc_macro_derive(Monad, attributes(monad))]
pub fn derive_monad(input: TokenStream) -> TokenStream {
    let input: DeriveInput = parse_macro_input!(input as DeriveInput);
    let test_type: Type = syn::parse_str("T0Repr<'a, Env, T>").unwrap();
    println!("{:#?}", test_type);
    println!("{:#?}", input);
    panic!("not yet implemented")
}
