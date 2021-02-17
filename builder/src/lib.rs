
use proc_macro2;
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, DeriveInput};

use std::{borrow::BorrowMut, mem::ManuallyDrop};

use std::error::Error;

mod utils;

#[proc_macro_derive(Builder, attributes(builder))]
pub fn derive(input: TokenStream) -> TokenStream {
    
    let ast = parse_macro_input!(input as DeriveInput);
    // eprintln!("{:#?}", ast);

    let struct_name = &ast.ident;
    let builder_struct_name = format_ident!("{}Builder", struct_name);

    let builder_struct_items:Vec<_> = utils::derive_get_struct_fields(&ast).unwrap().iter().map(|field|{
        let field_name = field.ident.as_ref();
        let field_type =  &field.ty;
        if utils::get_each_attr_name(field).is_some() {
            quote!(
                #field_name:#field_type
            )
        } else if utils::extract_inner_type(field, "Option".into()).is_some(){
            quote!(
                #field_name:#field_type
            )
        } else {
            quote!(
                #field_name:Option<#field_type>
            )
        }
    }).collect();

    let builder_factory_body_items:Vec<_> = utils::derive_get_struct_fields(&ast).unwrap().iter().map(|field|{
        let field_name = field.ident.as_ref();
        let field_type =  &field.ty;
        if utils::get_each_attr_name(field).is_some() {
            quote!(
                #field_name:<#field_type>::new()
            )
        } else {
            quote!(
                #field_name:None
            )
        }
    }).collect();


    let builder_struct_def = quote! {
        
        pub struct #builder_struct_name {
            #(#builder_struct_items),*
        }

        impl #struct_name {
            pub fn builder() -> #builder_struct_name {
                #builder_struct_name {
                    #(#builder_factory_body_items),*
                }
            }
        }
    };


    let builder_methods:Vec<_> = utils::derive_get_struct_fields(&ast).unwrap().iter().map(|field|{
        let field_name = field.ident.as_ref().unwrap();
        let field_each_name = utils::get_each_attr_name(field);

        let t;
        let field_setter_name = if let Some(setter_name) = field_each_name.as_ref(){
            t = format_ident!("{}",setter_name, span=field_name.span());
            &t
        } else {
            field_name
        };


        let field_type = if utils::is_field_optional(field){ 
            utils::extract_inner_type(field, "Option".into()).as_ref().unwrap()
        } else if field_each_name.is_some() {
            utils::extract_inner_type(field, "Vec".into()).as_ref().unwrap()
        } else {
            &field.ty
        };

        if field_each_name.is_some() {
            quote! {
                fn #field_setter_name(&mut self, #field_setter_name: #field_type) -> &mut Self {
                    self.#field_name.push(#field_setter_name);
                    self
                }
            }
        } else {
            quote! {
                fn #field_name(&mut self, #field_name: #field_type) -> &mut Self {
                    self.#field_name = Some(#field_name);
                    self
                }
            }
        }

    }).collect();
    

    let missing_field_check:Vec<_> = utils::derive_get_struct_fields(&ast).unwrap().iter().filter_map(|field|{
        if utils::is_field_optional(field) || utils::get_each_attr_name(field).is_some() {
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
        } else if utils::get_each_attr_name(field).is_some() {
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
