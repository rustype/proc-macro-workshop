extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

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
    let struct_ident = &input.ident;
    let struct_data = parse_data(&input.data)?;
    let struct_fields = &struct_data.fields;

    let struct_ident_str = format!("{}", struct_ident);
    let debug_fields = match struct_fields {
        syn::Fields::Named(fields_named) => {
            let field_idents = fields_named.named.iter().map(|named_field| {
                let named_field_ident = named_field.ident.as_ref().unwrap();
                let named_field_ident_str = format!("{}", named_field_ident);
                if named_field.attrs.is_empty() {
                    quote!(.field(#named_field_ident_str, &self.#named_field_ident))
                } else {
                    let attr = named_field.attrs.last().unwrap();
                    if attr.path.is_ident("debug") {
                        let attr_meta = &attr.parse_meta();
                        match attr_meta {
                            Ok(syn::Meta::NameValue(syn::MetaNameValue {lit, ..})) => {
                                let debug_assign_value = lit;
                                quote!(
                                    .field(#named_field_ident_str, &format!("{}", format_args!(#debug_assign_value, &self.#named_field_ident)))
                                )
                            }
                            Ok(meta) => syn::Error::new_spanned(meta, "expected meta name value").to_compile_error(),
                            Err(err) => err.to_compile_error(),
                        }
                    } else {
                        syn::Error::new_spanned(&attr.path, "value must be \"debug\"")
                            .to_compile_error()
                    }
                }
            });
            quote!(#( #field_idents )*)
        }
        syn::Fields::Unnamed(fields_unnamed) => {
            let field_indexes = (0..fields_unnamed.unnamed.len()).map(syn::Index::from);
            let field_indexes_str = (0..fields_unnamed.unnamed.len()).map(|idx| format!("{}", idx));
            quote!(#( .field(#field_indexes_str, &self.#field_indexes) )*)
        }
        syn::Fields::Unit => quote!(),
    };

    syn::Result::Ok(quote!(
        impl std::fmt::Debug for #struct_ident {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.debug_struct(#struct_ident_str)
                #debug_fields
                .finish()
            }
        }
    ))
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
