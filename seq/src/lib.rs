extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{Token, braced, parse::Parse, parse_macro_input};

struct Seq {
    name: syn::Ident,
    start: syn::Lit,
    end: syn::Lit,
}

impl Parse for Seq {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let content;
        let name = input.parse()?;
        input.parse::<Token![in]>()?;
        let start = input.parse()?;
        input.parse::<Token![..]>()?;
        let end = input.parse()?;
        braced!(content in input);
        Ok(Seq { name, start, end})
    }
}

#[proc_macro]
pub fn seq(input: TokenStream) -> TokenStream {
    let Seq { name, start, end } = parse_macro_input!(input as Seq);

    quote!().into()
}
