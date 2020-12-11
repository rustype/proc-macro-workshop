extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{braced, parse::Parse, parse_macro_input, Token};

#[derive(Debug)]
struct Seq {
    name: syn::Ident,
    start: syn::LitInt,
    end: syn::LitInt,
    body: proc_macro2::TokenStream,
    inclusive: bool,
}

impl Parse for Seq {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let name = input.parse()?;
        input.parse::<Token![in]>()?;
        let start = input.parse()?;
        let inclusive = if input.peek(Token![..=]) {
            input.parse::<Token![..=]>()?;
            true
        } else {
            input.parse::<Token![..]>()?;
            false
        };
        let end = input.parse()?;
        let content;
        braced!(content in input);
        let body = content.parse()?;
        Ok(Seq {
            name,
            start,
            end,
            body,
            inclusive,
        })
    }
}

impl Into<proc_macro2::TokenStream> for Seq {
    fn into(self) -> proc_macro2::TokenStream {
        self.expand()
    }
}

impl Seq {
    fn expand(&self) -> proc_macro2::TokenStream {
        self.replace_stream(self.body.clone())
    }

    fn replace_stream(&self, stream: proc_macro2::TokenStream) -> proc_macro2::TokenStream {
        // println!("{:#?}", stream.into());
        quote!()
    }
}

#[proc_macro]
pub fn seq(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as Seq);
    let output: proc_macro2::TokenStream = input.into();
    // output.into()
    quote!().into()
}
