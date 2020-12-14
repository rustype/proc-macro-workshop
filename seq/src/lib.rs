extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro2::Group;
use quote::quote;
use std::convert::From;
use std::iter::FromIterator;
use syn::{braced, parse::Parse, parse_macro_input, Token};

#[proc_macro]
pub fn seq(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as Seq);
    // println!("{:#?}", input);
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
        if self.body.is_empty() {
            syn::Result::Ok(quote!())
        } else {
            let (mut output, expanded) = self.expand_repetitions(self.body.clone())?;
            if !expanded {
                output = self.expand_range(output.clone())?;
            }
            // println!("{:#?}", output);
            syn::Result::Ok(output)
        }
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

    fn expand_repetitions(
        &self,
        tokens: proc_macro2::TokenStream,
    ) -> syn::Result<(proc_macro2::TokenStream, bool)> {
        let mut expanded = false;
        let mut tokens = Vec::from_iter(tokens);
        let mut i = 0;
        while i < tokens.len() {
            if let proc_macro2::TokenTree::Group(group) = &mut tokens[i] {
                let (content, was_expanded) = self.expand_repetitions(group.stream())?;
                let original_span = group.span();
                *group = proc_macro2::Group::new(group.delimiter(), content);
                group.set_span(original_span);
                expanded = was_expanded;
                i += 1;
                continue;
            }
            if i + 3 > tokens.len() {
                i += 1;
                continue;
            }
            let body = match enter_repetition(&tokens[i..i + 3]) {
                Some(body) => body,
                None => {
                    i += 1;
                    continue;
                }
            };
            // This is kinda hacky but sidesteps having to go around and replace N nodes n'friends
            let group = proc_macro2::TokenTree::Group(Group::new(proc_macro2::Delimiter::None, self.expand_range(body)?));
            tokens.splice(i..i + 3, std::iter::once(group));
            i += 1;
            expanded = true;
        }
        syn::Result::Ok((quote!(#(#tokens)*), expanded))
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

            if let proc_macro2::TokenTree::Ident(ident) = &mut tokens[i] {
                if *ident == self.var {
                    let mut lit = proc_macro2::Literal::i64_unsuffixed(value);
                    lit.set_span(ident.span());
                    tokens[i] = proc_macro2::TokenTree::Literal(lit);
                    i += 1;
                    continue;
                }
                if i + 2 >= tokens.len() {
                    i += 1;
                    continue;
                }
                if let proc_macro2::TokenTree::Punct(punct) = &tokens[i + 1] {
                    if punct.as_char() != '#' {
                        i += 1;
                        continue;
                    }
                    if let (
                        proc_macro2::TokenTree::Ident(prefix),
                        proc_macro2::TokenTree::Ident(var),
                    ) = (&tokens[i], &tokens[i + 2])
                    {
                        if self.var.to_string() == var.to_string() {
                            let ident = proc_macro2::Ident::new(
                                &format!("{}{}", prefix.to_string(), value),
                                prefix.span(),
                            )
                            .into();
                            tokens.splice(i..i + 3, std::iter::once(ident));
                            i += 2;
                            continue;
                        }
                    }
                }
            }
            i += 1;
        }
        // it is ironic that we are implementing a mechanism
        // while using the same kind of mechanism
        syn::Result::Ok(quote!(#(#tokens)*))
    }
}

fn enter_repetition(tokens: &[proc_macro2::TokenTree]) -> Option<proc_macro2::TokenStream> {
    assert!(tokens.len() == 3);
    match &tokens[0] {
        proc_macro2::TokenTree::Punct(punct) if punct.as_char() == '#' => {}
        _ => return None,
    }
    match &tokens[2] {
        proc_macro2::TokenTree::Punct(punct) if punct.as_char() == '*' => {}
        _ => return None,
    }
    match &tokens[1] {
        proc_macro2::TokenTree::Group(group)
            if group.delimiter() == proc_macro2::Delimiter::Parenthesis =>
        {
            Some(group.stream())
        }
        _ => None,
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
