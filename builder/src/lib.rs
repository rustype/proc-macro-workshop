// extern crate proc_macro;

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(Builder, attributes(builder))]
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

fn impl_struct(struct_ident: &syn::Ident, data: &syn::Data) -> std::option::Option<proc_macro2::TokenStream> {
    if let syn::Data::Struct(syn::DataStruct {
        fields: syn::Fields::Named(syn::FieldsNamed { named, .. }),
        ..
    }) = data
    {
        let builder_struct_inits = named.iter().map(initialize_field);
        let builder_struct_ident = format_ident!("{}Builder", struct_ident);
        std::option::Option::Some(quote! (
            impl #struct_ident {
                pub fn builder() -> #builder_struct_ident {
                    #builder_struct_ident {
                        #(#builder_struct_inits),*
                    }
                }
            }
        ))
    } else {
        std::option::Option::None
    }
}

fn builder_struct(struct_ident: &syn::Ident, data: &syn::Data) -> std::option::Option<proc_macro2::TokenStream> {
    if let syn::Data::Struct(syn::DataStruct {
        fields: syn::Fields::Named(syn::FieldsNamed { named, .. }),
        ..
    }) = data
    {
        let builder_struct_fields = named.iter().map(optionize_field);
        let builder_struct_ident = format_ident!("{}Builder", struct_ident);
        std::option::Option::Some(quote!(
            pub struct #builder_struct_ident {
                #(#builder_struct_fields),*
            }
        ))
    } else {
        std::option::Option::None
    }
}

fn builder_impl(struct_ident: &syn::Ident, data: &syn::Data) -> std::option::Option<proc_macro2::TokenStream> {
    if let syn::Data::Struct(syn::DataStruct {
        fields: syn::Fields::Named(syn::FieldsNamed { named, .. }),
        ..
    }) = data
    {
        let builder_impl_functions = named.iter().map(functionize_field);
        let builder_fields = named.iter().map(assign_field);
        let builder_struct_ident = format_ident!("{}Builder", struct_ident);
        std::option::Option::Some(quote!(
            impl #builder_struct_ident {
                pub fn build(&mut self) -> std::result::Result<#struct_ident, std::boxed::Box<dyn std::error::Error>> {
                    std::result::Result::Ok(
                        #struct_ident {
                            #(#builder_fields),*
                        }
                    )
                }
                #(#builder_impl_functions)*
            }
        ))
    } else {
        std::option::Option::None
    }
}

fn initialize_field(field: &syn::Field) -> proc_macro2::TokenStream {
    let ref field_name = field.ident;
    let ref field_type = field.ty;
    if extract_inner_type(field_type, "Vec").is_some() {
        quote!(#field_name: std::option::Option::Some(vec!()))
    } else {
        quote!(#field_name: std::option::Option::None)
    }
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
    let field_name = field.ident.as_ref().unwrap();
    let mut field_type = &field.ty;
    if let std::option::Option::Some(inner_ty) = extract_inner_type(field_type, "Option") {
        field_type = inner_ty;
    }
    let once_setter_tt = once_setter(field_name, field_type);
    if field.attrs.is_empty() {
        return quote!(
           #once_setter_tt
        );
    }
    match extract_each_attr_value(&field.attrs) {
        std::result::Result::Ok(attr_value) => {
            let attr_value = &format_ident!("{}", attr_value);
            let vec_inner_type =
                extract_inner_type(field_type, "Vec").expect("inner type of Vec is <>????");
            let each_setter_tt = each_setter(attr_value, field_name, vec_inner_type);
            if attr_value == field_name {
                quote!(
                    #each_setter_tt
                )
            } else {
                quote! (
                    #once_setter_tt
                    #each_setter_tt
                )
            }
        }
        std::result::Result::Err(err) => err,
    }
}

fn once_setter(field_name: &syn::Ident, field_type: &syn::Type) -> proc_macro2::TokenStream {
    quote!(fn #field_name(&mut self, #field_name: #field_type) -> &mut Self {
        self.#field_name = std::option::Option::Some(#field_name);
        self
    })
}
fn each_setter(
    method_name: &syn::Ident,
    field_name: &syn::Ident,
    field_type: &syn::Type,
) -> proc_macro2::TokenStream {
    quote!(
        fn #method_name(&mut self, #field_name: #field_type) -> &mut Self {
            if let std::option::Option::Some(ref mut v) = self.#field_name {
                v.push(#field_name);
            } else {
                self.#field_name = std::option::Option::Some(vec![#field_name]);
            }
            self
        }
    )
}

fn extract_each_attr_value(
    attrs: &Vec<syn::Attribute>,
) -> Result<String, proc_macro2::TokenStream> {
    if let std::option::Option::Some(attr) = attrs.last() {
        if let std::result::Result::Ok(syn::Meta::List(list)) = attr.parse_meta() {
            if list.path.is_ident("builder") {
                if let std::option::Option::Some(syn::NestedMeta::Meta(syn::Meta::NameValue(syn::MetaNameValue {
                    path,
                    lit: syn::Lit::Str(lit_str),
                    ..
                }))) = list.nested.last()
                {
                    if path.is_ident("each") {
                        return std::result::Result::Ok(lit_str.value());
                    } else {
                        println!("not each");
                        return std::result::Result::Err(syn::Error::new_spanned(list, "expected `builder(each = \"...\")`")
                            .to_compile_error());
                    }
                }
            }
        }
    }
    unreachable!("you should've check for attrs emptyness before")
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
        if let std::option::Option::Some(syn::PathSegment {
            ident,
            arguments:
                syn::PathArguments::AngleBracketed(syn::AngleBracketedGenericArguments { args, .. }),
        }) = segments.last()
        {
            if ident == expected_ident {
                if let std::option::Option::Some(syn::GenericArgument::Type(ty)) = args.last() {
                    return std::option::Option::Some(ty);
                }
            }
        }
    }
    None
}
