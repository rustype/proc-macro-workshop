// extern crate proc_macro;

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(Builder)]
pub fn derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
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
        fields: syn::Fields::Named(syn::FieldsNamed { named, .. }),
        ..
    }) = data
    {
        let builder_struct_inits = named.iter().map(initialize_field);
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
        fields: syn::Fields::Named(syn::FieldsNamed { named, .. }),
        ..
    }) = data
    {
        let builder_struct_fields = named.iter().map(optionize_field);
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
        fields: syn::Fields::Named(syn::FieldsNamed { named, .. }),
        ..
    }) = data
    {
        let builder_impl_functions = named.iter().map(functionize_field);
        let builder_fields = named.iter().map(assign_field);
        let builder_struct_ident = format_ident!("{}Builder", struct_ident);
        Some(quote!(
            impl #builder_struct_ident {
                pub fn build(&mut self) -> std::result::Result<#struct_ident, Box<dyn std::error::Error>> {
                    Ok(
                        #struct_ident {
                            #(#builder_fields),*
                        }
                    )
                }
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

fn assign_field(field: &syn::Field) -> proc_macro2::TokenStream {
    let ref field_name = field.ident;
    let ref field_type = field.ty;
    if extract_inner_type(field_type, "Option").is_some() {
        quote!(#field_name: self.#field_name.clone())
    } else {
        quote!(#field_name: self.#field_name.clone().ok_or("field was not set")?)
    }
}

fn functionize_field(field: &syn::Field) -> proc_macro2::TokenStream {
    let ref field_name = field.ident;
    let mut field_type = &field.ty;
    if let Some(inner_ty) = extract_inner_type(field_type, "Option") {
        field_type = inner_ty;
    }
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
    if extract_inner_type(field_type, "Option").is_some() {
        quote!(#field_name: #field_type)
    } else {
        quote!(#field_name: std::option::Option<#field_type>)
    }
}

fn extract_inner_type<'t>(ty: &'t syn::Type, expected_ident: &str) -> Option<&'t syn::Type> {
    if let syn::Type::Path(syn::TypePath {
        path: syn::Path { segments, .. },
        ..
    }) = ty
    {
        if let Some(syn::PathSegment {
            ident,
            arguments:
                syn::PathArguments::AngleBracketed(syn::AngleBracketedGenericArguments { args, .. }),
        }) = segments.last()
        {
            if ident == expected_ident {
                if let Some(syn::GenericArgument::Type(ty)) = args.last() {
                    return Some(ty);
                }
            }
        }
    }
    None
}
