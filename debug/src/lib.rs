extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, parse_quote, DeriveInput};

#[proc_macro_derive(CustomDebug, attributes(debug))]
pub fn derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let output = parse_derive_input(&input);
    match output {
        syn::Result::Ok(tt) => tt,
        syn::Result::Err(err) => err.to_compile_error(),
    }
    .into()
}

fn parse_derive_input(input: &syn::DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    println!("{:#?}", &input);
    let struct_ident = &input.ident;
    let struct_data = parse_data(&input.data)?;
    let struct_fields = &struct_data.fields;
    let generics = add_debug_bound(struct_fields, input.generics.clone());
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let struct_ident_str = format!("{}", struct_ident);
    let debug_fields = match struct_fields {
        syn::Fields::Named(fields_named) => handle_named_fields(fields_named)?,
        syn::Fields::Unnamed(fields_unnamed) => {
            let field_indexes = (0..fields_unnamed.unnamed.len()).map(syn::Index::from);
            let field_indexes_str = (0..fields_unnamed.unnamed.len()).map(|idx| format!("{}", idx));
            quote!(#( .field(#field_indexes_str, &self.#field_indexes) )*)
        }
        syn::Fields::Unit => quote!(),
    };

    syn::Result::Ok(quote!(
        impl #impl_generics std::fmt::Debug for #struct_ident #ty_generics #where_clause {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.debug_struct(#struct_ident_str)
                #debug_fields
                .finish()
            }
        }
    ))
}

fn handle_named_fields(fields: &syn::FieldsNamed) -> syn::Result<proc_macro2::TokenStream> {
    fields.named.iter().map(parse_named_field).collect()
}

fn parse_named_field(field: &syn::Field) -> syn::Result<proc_macro2::TokenStream> {
    let ident = field.ident.as_ref().unwrap();
    let ident_str = format!("{}", ident);
    if field.attrs.is_empty() {
        syn::Result::Ok(quote!(.field(#ident_str, &self.#ident)))
    } else {
        parse_named_field_attrs(field)
    }
}

fn parse_named_field_attrs(field: &syn::Field) -> syn::Result<proc_macro2::TokenStream> {
    let ident = field.ident.as_ref().unwrap();
    let ident_str = format!("{}", ident);
    let attr = field.attrs.last().unwrap();
    if !attr.path.is_ident("debug") {
        return syn::Result::Err(syn::Error::new_spanned(
            &attr.path,
            "value must be \"debug\"",
        ));
    }
    let attr_meta = &attr.parse_meta();
    match attr_meta {
        Ok(syn::Meta::NameValue(syn::MetaNameValue { lit, .. })) => {
            let debug_assign_value = lit;
            syn::Result::Ok(quote!(
                .field(#ident_str, &format_args!(#debug_assign_value, &self.#ident))
            ))
        }
        Ok(meta) => syn::Result::Err(syn::Error::new_spanned(meta, "expected meta name value")),
        Err(err) => syn::Result::Err(err.clone()),
    }
}

fn parse_data(data: &syn::Data) -> syn::Result<&syn::DataStruct> {
    match data {
        syn::Data::Struct(data_struct) => syn::Result::Ok(data_struct),
        syn::Data::Enum(syn::DataEnum { enum_token, .. }) => syn::Result::Err(
            syn::Error::new_spanned(enum_token, "CustomDebug is not implemented for enums"),
        ),
        syn::Data::Union(syn::DataUnion { union_token, .. }) => syn::Result::Err(
            syn::Error::new_spanned(union_token, "CustomDebug is not implemented for unions"),
        ),
    }
}

fn add_debug_bound(fields: &syn::Fields, mut generics: syn::Generics) -> syn::Generics {
    let mut phantom_ty_idents = std::collections::HashSet::new();
    let mut non_phantom_ty_idents = std::collections::HashSet::new();
    for (ident, opt_iter) in fields
        .iter()
        .flat_map(extract_ty_path)
        .map(extract_ty_idents)
    {
        if ident == "PhantomData" {
            // If the field type ident is `PhantomData`
            // add the generic parameters into the phantom idents collection
            if let std::option::Option::Some(args) = opt_iter {
                for arg in args {
                    phantom_ty_idents.insert(arg);
                }
            }
        } else {
            // Else, add the type and existing generic parameters into the non-phantom idents collection
            non_phantom_ty_idents.insert(ident);
            if let std::option::Option::Some(args) = opt_iter {
                for arg in args {
                    non_phantom_ty_idents.insert(arg);
                }
            }
        }
    }
    // Find the difference between the phantom idents and non-phantom idents
    // Collect them into an hash set for O(1) lookup
    let non_debug_fields = phantom_ty_idents
        .difference(&non_phantom_ty_idents)
        .collect::<std::collections::HashSet<_>>();
    // Iterate generic params and if their ident is NOT in the phantom fields
    // do not add the generic bound
    for param in generics.type_params_mut() {
        // this is kinda shady, hoping it works
        if !non_debug_fields.contains(&&param.ident) {
            param.bounds.push(parse_quote!(std::fmt::Debug));
        }
    }
    generics
}

/// Extract the path from the type path in a field.
fn extract_ty_path(field: &syn::Field) -> std::option::Option<&syn::Path> {
    if let syn::Type::Path(syn::TypePath { path, .. }) = &field.ty {
        std::option::Option::Some(&path)
    } else {
        std::option::Option::None
    }
}

/// From a `syn::Path` extract both the type ident and an iterator over generic type arguments.
fn extract_ty_idents(
    path: &syn::Path,
) -> (
    &syn::Ident,
    std::option::Option<impl Iterator<Item = &syn::Ident>>,
) {
    let ty_segment = path.segments.last().unwrap();
    let ty_ident = &ty_segment.ident;
    if let syn::PathArguments::AngleBracketed(syn::AngleBracketedGenericArguments {
        args, ..
    }) = &ty_segment.arguments
    {
        let ident_iter = args.iter().flat_map(|gen_arg| {
            if let syn::GenericArgument::Type(syn::Type::Path(syn::TypePath { path, .. })) = gen_arg
            {
                path.get_ident()
            } else {
                std::option::Option::None
            }
        });
        (ty_ident, std::option::Option::Some(ident_iter))
    } else {
        (ty_ident, std::option::Option::None)
    }
}

fn add_trait_bounds(mut generics: syn::Generics) -> syn::Generics {
    for param in &mut generics.params {
        if let syn::GenericParam::Type(ref mut type_param) = *param {
            type_param.bounds.push(parse_quote!(std::fmt::Debug));
        }
    }
    generics
}
