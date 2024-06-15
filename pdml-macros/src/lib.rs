extern crate proc_macro;

use proc_macro::TokenStream;
use quote::{format_ident, quote, ToTokens};
use syn::{DeriveInput, Field, FieldsNamed};

fn ignore_if_option(f: &Field) -> proc_macro2::TokenStream {
    let Field {
        attrs,
        vis,
        ident,
        ty,
        ..
    } = f;
    if ty.to_token_stream().to_string().contains("Option") {
        quote! {
            #(#attrs)'\n'*
            #vis #ident: #ty,
        }
    } else {
        quote! {
            #(#attrs)'\n'*
            #vis #ident: std::option::Option<#ty>,
        }
    }
}

fn assignment_fields(f: &Field) -> proc_macro2::TokenStream {
    let Field { ident, ty, .. } = f;
    if ty.to_token_stream().to_string().contains("Option") {
        quote! {
            #ident: self.#ident,
        }
    } else {
        quote! {
            #ident: self.#ident.unwrap(),
        }
    }
}

fn default_fields(f: &Field) -> proc_macro2::TokenStream {
    let Field { ident, .. } = f;
    quote! {
        #ident: None,
    }
}

#[proc_macro_attribute]
pub fn partial(_attr: TokenStream, mut item: TokenStream) -> TokenStream {
    let DeriveInput { ident, data ,.. } = syn::parse(item.clone()).unwrap();

    let partial_name = format_ident!("Partial{}", ident);
    let mut ass_fields: Vec<proc_macro2::TokenStream> = vec![];
    let mut def_fields: Vec<proc_macro2::TokenStream> = vec![];

    let partial_fields = match data {
        syn::Data::Struct(s) => match s.fields {
            syn::Fields::Named(FieldsNamed { named, .. }) => named
                .iter()
                .map(|f| {
                    ass_fields.push(assignment_fields(f));
                    def_fields.push(default_fields(f));
                    ignore_if_option(f)
                })
                .collect::<Vec<proc_macro2::TokenStream>>(),
            _ => todo!(),
        },
        _ => panic!("The macro partial only works for structs"),
    };
    let tokens = TokenStream::from(quote! {
        struct #partial_name {
            #(#partial_fields)*
        }

        impl Into<#ident> for #partial_name {
            fn into(self) -> #ident {
                #ident {
                    #(#ass_fields)*
                }
            }
        }

        impl Default for #partial_name {
            fn default() -> Self {
                Self {
                    #(#def_fields)*
                }
            }
        }
    });

    item.extend(tokens);
    item
}
