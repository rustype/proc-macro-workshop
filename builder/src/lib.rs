// extern crate proc_macro;

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(Builder)]
pub fn derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    println!("{:#?}", input);
    let struct_ident_original = input.ident;
    let original_struct_impl = impl_struct(&struct_ident_original, &input.data).unwrap();
    let builder_struct = builder_struct(&struct_ident_original, &input.data).unwrap();
    let builder_impl = builder_impl(&struct_ident_original, &input.data).unwrap();

    quote!(
        #original_struct_impl
        #builder_struct
        #builder_impl
    )
    .into()
}

fn impl_struct(struct_ident: &syn::Ident, data: &syn::Data) -> Option<proc_macro2::TokenStream> {
    if let syn::Data::Struct(syn::DataStruct {
        fields:
            syn::Fields::Named(syn::FieldsNamed {
                named: named_fields,
                ..
            }),
        ..
    }) = data
    {
        let builder_struct_inits = named_fields.iter().map(initialize_field);
        let builder_struct_ident = format_ident!("{}Builder", struct_ident);
        Some(quote! (
            impl #struct_ident {
                pub fn builder() -> #builder_struct_ident {
                    #builder_struct_ident {
                        #(#builder_struct_inits),*
                    }
                }
            }
        ))
    } else {
        None
    }
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

fn builder_impl(struct_ident: &syn::Ident, data: &syn::Data) -> Option<proc_macro2::TokenStream> {
    if let syn::Data::Struct(syn::DataStruct {
        fields:
            syn::Fields::Named(syn::FieldsNamed {
                named: named_fields,
                ..
            }),
        ..
    }) = data
    {
        let builder_impl_functions = named_fields.iter().map(functionize_field);
        let builder_struct_ident = format_ident!("{}Builder", struct_ident);
        Some(quote!(
            impl #builder_struct_ident {
                #(#builder_impl_functions)*
            }
        ))
    } else {
        None
    }
}

fn initialize_field(field: &syn::Field) -> proc_macro2::TokenStream {
    let ref field_name = field.ident;
    quote!(#field_name: None)
}

fn functionize_field(field: &syn::Field) -> proc_macro2::TokenStream {
    let ref field_name = field.ident;
    let ref field_type = field.ty;
    quote!(
        fn #field_name(&mut self, #field_name: #field_type) -> &mut Self {
            self.#field_name = Some(#field_name);
            self
        }
    )
}

fn optionize_field(field: &syn::Field) -> proc_macro2::TokenStream {
    let ref field_name = field.ident;
    let ref field_type = field.ty;
    quote!(#field_name: std::option::Option<#field_type>)
}
