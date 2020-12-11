extern crate proc_macro;

use proc_macro2::TokenStream;
use quote::quote;
use syn::parse_macro_input;

#[proc_macro_attribute]
pub fn sorted(
    args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let _ = args;
    let item = parse_macro_input!(input as syn::Item);

    match parse_sorted(item) {
        Ok(tt) => tt,
        Err(err) => err.to_compile_error(),
    }
    .into()
}

fn parse_sorted(item: syn::Item) -> syn::Result<TokenStream> {
    println!("{:#?}", item);
    if let syn::Item::Enum(item_enum) = item {
        parse_sorted_enum(item_enum)
    } else {
        syn::Result::Err(syn::Error::new(
            proc_macro2::Span::call_site(),
            "expected enum or match expression",
        ))
    }
}

fn parse_sorted_enum(item_enum: syn::ItemEnum) -> syn::Result<TokenStream> {
    is_sorted(item_enum.variants.iter().map(|v| &v.ident))?;
    syn::Result::Ok(quote!(#item_enum))
}

fn is_sorted<'a, I>(iter: I) -> syn::Result<()>
where
    I: Iterator<Item = &'a syn::Ident>,
{
    let mut acc = vec![];
    for ident in iter {
        for acc_ident in &acc {
            if *acc_ident > ident {
                return syn::Result::Err(syn::Error::new_spanned(
                    ident,
                    format!("{} should sort before {}", ident, acc_ident),
                ));
            }
        }
        acc.push(ident);
    }
    syn::Result::Ok(())
}
