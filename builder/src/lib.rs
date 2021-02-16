
use proc_macro2;
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, DeriveInput};

use std::error::Error;

mod utils;

#[proc_macro_derive(Builder)]
pub fn derive(input: TokenStream) -> TokenStream {
    
    let ast = parse_macro_input!(input as DeriveInput);
    // eprintln!("{:#?}", ast);

    let struct_name = &ast.ident;
    let builder_struct_name = format_ident!("{}Builder", struct_name);

    let field_names:Vec<syn::Ident> = utils::derive_get_struct_fields(&ast).unwrap().iter().map(|field|{
        field.ident.clone().unwrap()
    }).collect();

    let field_types:Vec<syn::Type> = utils::derive_get_struct_fields(&ast).unwrap().iter().map(|field|{
        if let Some(ty) = utils::extract_inner_type(field, "Option".into()){
            ty.clone()
        } else {
            field.ty.clone()
        }
        
    }).collect();


    let builder_struct_def = quote! {
        
        pub struct #builder_struct_name {
            #(#field_names:Option<#field_types>),*
        }

        impl #struct_name {
            pub fn builder() -> #builder_struct_name {
                #builder_struct_name {
                    #(#field_names:None),*
                }
            }
        }
    };


    let builder_methods:Vec<_> = utils::derive_get_struct_fields(&ast).unwrap().iter().map(|field|{
        let field_name = &field.ident;
        let field_type = if utils::is_field_optional(field){
            utils::extract_inner_type(field, "Option".into()).clone().unwrap()
        } else {
            &field.ty
        };
        quote! {
            fn #field_name(&mut self, #field_name: #field_type) -> &mut Self {
                self.#field_name = Some(#field_name);
                self
            }
        }
    }).collect();
    

    let missing_field_check:Vec<_> = utils::derive_get_struct_fields(&ast).unwrap().iter().filter_map(|field|{
        if utils::is_field_optional(field) {
            return None
        }
        let field_name = &field.ident;
        let missing_msg = format!("Field {} is missing", field_name.clone().unwrap());
        return Some(
            quote!(
                if let None = self.#field_name {
                    return Err(#missing_msg.into())
                }
            )
        )
    }).collect();

    let build_final_result_struct_body: Vec<_> = utils::derive_get_struct_fields(&ast).unwrap().iter().map(|field|{
        let field_name = &field.ident;

        if utils::is_field_optional(field) {
            quote!(
                #field_name:self.#field_name.clone()
            )
        } else {
            quote!(
                #field_name:self.#field_name.clone().unwrap()
            )
        }
    }).collect();

    let builder_build_method = quote!(
        fn build(&mut self) -> Result<#struct_name, Box<dyn std::error::Error>> {
            #(#missing_field_check)*
            Ok(
                #struct_name{
                    #(#build_final_result_struct_body),*
                }
            )
        }
    );



    (quote!{
        #builder_struct_def
        impl #builder_struct_name {
            #(#builder_methods)*
            #builder_build_method
        }
    }).into()
}
