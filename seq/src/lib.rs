extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{braced, parse::Parse, parse_macro_input, Token};

#[proc_macro]
pub fn seq(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as Seq);
    println!("{:#?}", input);
    let output = input.expand();
    match output {
        Ok(ts) => ts.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

#[derive(Debug)]
struct Seq {
    var: syn::Ident,
    start: i64,
    end: i64,
    body: proc_macro2::TokenStream,
    inclusive: bool,
}

impl Seq {
    fn expand(&self) -> syn::Result<proc_macro2::TokenStream> {
        let output = self.expand_range(self.body.clone())?;
        println!("{:#?}", output);
        syn::Result::Ok(output)
    }


    fn expand_range(
        &self,
        tokens: proc_macro2::TokenStream,
    ) -> syn::Result<proc_macro2::TokenStream> {
        let mut output = proc_macro2::TokenStream::new();
        if self.inclusive {
            for value in self.start..=self.end {
                output.extend(self.inner_expand(value, tokens.clone()));
            }
        } else {
            for value in self.start..self.end {
                output.extend(self.inner_expand(value, tokens.clone()));
            }
        }
        syn::Result::Ok(output)
    }

    #[allow(dead_code)]
    fn expand_repetitions(&self) -> syn::Result<proc_macro2::TokenStream> {
        let mut tokens: Vec<proc_macro2::TokenTree> = self.body.clone().into_iter().collect();
        let mut i = 0;
        while i < tokens.len() - 2 {
            match &mut tokens[i..i + 3] {
                // check for a repetition group
                // #(...)*
                [proc_macro2::TokenTree::Punct(pound), proc_macro2::TokenTree::Group(group), proc_macro2::TokenTree::Punct(star)]
                    if pound.as_char() == '#' && star.as_char() == '*' =>
                {
                    if let proc_macro2::Delimiter::Parenthesis = group.delimiter() {
                        let original_span = group.span();
                        *group = proc_macro2::Group::new(
                            group.delimiter(),
                            self.expand_range(group.stream())?,
                        );
                        group.set_span(original_span);
                        i += 3;
                        continue;
                    }
                }
                _ => {
                    i += 1;
                    continue;
                }
            }
        }
        syn::Result::Ok(quote!(#(#tokens)*))
    }

    fn inner_expand(
        &self,
        value: i64,
        tokens: proc_macro2::TokenStream,
    ) -> syn::Result<proc_macro2::TokenStream> {
        let mut tokens: Vec<proc_macro2::TokenTree> = tokens.into_iter().collect();
        let mut i = 0;
        while i < tokens.len() {
            if let proc_macro2::TokenTree::Group(group) = &mut tokens[i] {
                // process the group as a standalone stream
                let content = self.inner_expand(value, group.stream())?;
                // save the original
                let original_span = group.span();
                // replace the group which yeilded the recursive call
                // with a new group built from the previous one
                *group = proc_macro2::Group::new(group.delimiter(), content);
                // since group is now the new group
                // replace the span with the original one
                group.set_span(original_span);
                // advance the iteration for the next token
                i += 1;
                // skip processing of other possibilities
                continue;
            }
            // detected some form of punctuation
            if let proc_macro2::TokenTree::Punct(punct) = &mut tokens[i] {
                // if it is not # we don't care for it
                // so we just keep going
                if punct.as_char() != '#' {
                    i += 1;
                    continue;
                }

                // try to match the previous and next tokens
                // if both tokens are idents
                // and the next token is an equal ident to the seq var
                // create a new token and replace the three tokens with the new ones
                if let (proc_macro2::TokenTree::Ident(prefix), proc_macro2::TokenTree::Ident(var)) =
                    (&tokens[i - 1], &tokens[i + 1])
                {
                    if self.var.to_string() == var.to_string() {
                        let ident = proc_macro2::Ident::new(
                            &format!("{}{}", prefix.to_string(), value),
                            prefix.span(),
                        )
                        .into();
                        tokens.splice(i - 1..=i + 1, std::iter::once(ident));
                        i += 2;
                        continue;
                    }
                }
            }

            if let proc_macro2::TokenTree::Ident(ident) = &mut tokens[i] {
                if *ident == self.var {
                    let mut lit = proc_macro2::Literal::i64_unsuffixed(value);
                    lit.set_span(ident.span());
                    tokens[i] = proc_macro2::TokenTree::Literal(lit);
                }
                i += 1;
                continue;
            }
            i += 1;
        }
        // it is ironic that we are implementing a mechanism
        // while using the same kind of mechanism
        syn::Result::Ok(quote!(#(#tokens)*))
    }
}

impl Parse for Seq {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let var = input.parse()?;
        input.parse::<Token![in]>()?;
        let start: syn::LitInt = input.parse()?;
        let start: i64 = start.base10_parse()?;
        let inclusive = if input.peek(Token![..=]) {
            input.parse::<Token![..=]>()?;
            true
        } else {
            input.parse::<Token![..]>()?;
            false
        };
        let end: syn::LitInt = input.parse()?;
        let end: i64 = end.base10_parse()?;
        let content;
        braced!(content in input);
        let body = content.parse()?;
        Ok(Seq {
            var,
            start,
            end,
            body,
            inclusive,
        })
    }
}
