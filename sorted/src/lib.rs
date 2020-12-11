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
    if let syn::Item::Enum(_) = item {
        syn::Result::Ok(quote!(#item))
    } else {
        syn::Result::Err(syn::Error::new(
            proc_macro2::Span::call_site(),
            "expected enum or match expression",
        ))
    }
}
