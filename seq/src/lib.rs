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
    start: syn::LitInt,
    end: syn::LitInt,
    body: proc_macro2::TokenStream,
    inclusive: bool,
}

impl Seq {
    fn expand(&self) -> syn::Result<proc_macro2::TokenStream> {
        let start: usize = self.start.base10_parse()?;
        let end: usize = self.end.base10_parse()?;
        let mut output = proc_macro2::TokenStream::new();
        if self.inclusive {
            for value in start..=end {
                output.extend(self.inner_expand(value, self.body.clone()));
            }
        } else {
            for value in start..end {
                output.extend(self.inner_expand(value, self.body.clone()));
            }
        }
        println!("{:#?}", output);
        syn::Result::Ok(output)
    }

    fn inner_expand(
        &self,
        value: usize,
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
            if let proc_macro2::TokenTree::Punct(punct) = &mut tokens[i] {
                if punct.as_char() != '#' {
                    i += 1;
                    continue;
                }
                match (&tokens[i - 1], &tokens[i + 1]) {
                    (proc_macro2::TokenTree::Ident(prefix), proc_macro2::TokenTree::Ident(var))
                        if self.var.to_string() == var.to_string() =>
                    {
                        let ident = proc_macro2::Ident::new(
                            &format!("{}{}", prefix.to_string(), value),
                            prefix.span(),
                        )
                        .into();
                        tokens.splice(i - 1..=i + 1, std::iter::once(ident));
                        i += 2;
                        continue;
                    }
                    _ => {
                        i += 1;
                        continue;
                    }
                }
            }

            if let proc_macro2::TokenTree::Ident(ident) = &mut tokens[i] {
                if *ident == self.var {
                    let mut lit = proc_macro2::Literal::usize_unsuffixed(value);
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
            var,
            start,
            end,
            body,
            inclusive,
        })
    }
}
