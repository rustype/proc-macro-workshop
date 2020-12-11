mod visitor;

use quote::quote;
use syn::parse_macro_input;

#[proc_macro_attribute]
pub fn sorted(
    _: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    // since the sorted attribute only checks for sorted enums
    // the output is the same as the input
    let mut output = input.clone();
    let item = parse_macro_input!(input as syn::Item);

    // with the possible addition of an error to the out stream
    if let syn::Result::Err(err) = parse_sorted(item) {
        output.extend(proc_macro::TokenStream::from(err.to_compile_error()))
    }

    output
}

/// Parses the `syn::Item` first checking if the item is an `Enum` variant.
fn parse_sorted(item: syn::Item) -> syn::Result<()> {
    if let syn::Item::Enum(item_enum) = item {
        parse_sorted_enum(item_enum)
    } else {
        syn::Result::Err(syn::Error::new(
            proc_macro2::Span::call_site(),
            "expected enum or match expression",
        ))
    }
}

/// Parses the `syn::ItemEnum` and
fn parse_sorted_enum(item_enum: syn::ItemEnum) -> syn::Result<()> {
    is_sorted(item_enum.variants.iter().map(|v| &v.ident))?;
    syn::Result::Ok(())
}

/// Checks if the iterator of `syn::Ident`'s yields a sorted stream.
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

#[proc_macro_attribute]
pub fn check(
    _: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let mut item_fn = parse_macro_input!(input as syn::ItemFn);
    // println!("{:#?}", item_fn);
    let mut mv = crate::visitor::MatchVisitor::new();
    syn::visit_mut::VisitMut::visit_item_fn_mut(&mut mv, &mut item_fn);
    let mut output = quote!(#item_fn);
    if !mv.errors.is_empty() {
        output.extend(
            mv.errors
                .iter()
                .map(|err| proc_macro2::TokenStream::from(err.to_compile_error())),
        );
    }
    output.into()
}
