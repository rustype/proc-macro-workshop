// extern crate proc_macro;

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(Builder)]
pub fn derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    println!("{:#?}", input);
    let struct_ident_original = input.ident;
    let original_struct_impl = impl_struct(&struct_ident_original);
    let builder_struct = builder_struct(&struct_ident_original, &input.data).unwrap();

    quote!(
        #original_struct_impl
        #builder_struct
    )
    .into()
}

fn impl_struct(struct_ident: &syn::Ident) -> proc_macro2::TokenStream {
    quote! (
        impl #struct_ident {
            pub fn builder() {}
        }
    )
}

fn builder_struct(struct_ident: &syn::Ident, data: &syn::Data) -> Option<proc_macro2::TokenStream> {
    if let syn::Data::Struct(syn::DataStruct {
        fields:
            syn::Fields::Named(syn::FieldsNamed {
                named: named_fields,
                ..
            }),
        ..
    }) = data
    {
        println!("{:#?}", named_fields);
        let builder_struct_fields = named_fields.iter().map(optionize_field);
        let builder_struct_ident = format_ident!("{}Builder", struct_ident);
        Some(quote!(
            pub struct #builder_struct_ident {
                #(#builder_struct_fields),*
            }
        ))
    } else {
        None
    }
}

fn optionize_field(field: &syn::Field) -> proc_macro2::TokenStream {
    let ref field_name = field.ident;
    let ref field_type = field.ty;
    quote!(#field_name: std::option::Option<#field_type>)
}
