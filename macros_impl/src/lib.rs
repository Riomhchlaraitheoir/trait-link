use crate::parse::Parser;
use proc_macro2::{Ident, TokenStream};
use quote::ToTokens;
use syn::{Generics, ItemTrait, PatType, Path, Type, Visibility};

#[cfg(test)]
mod tests;
mod parse;

mod output;

pub fn rpc(_args: TokenStream, input: ItemTrait) -> syn::Result<impl ToTokens> {
    let parser = Parser;
    parser.rpc(input)
}

struct Rpc {
    vis: Visibility,
    generics: Generics,
    name: Ident,
    methods: Vec<Method>,
}

struct Method {
    name: Ident,
    args: Vec<PatType>,
    ret: ReturnType,
}

#[derive(Debug, PartialEq, Eq)]
enum ReturnType {
    Simple(Type),
    Nested { service: Path }
}
