
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
        field.ty.clone()
    }).collect();

    let field_mising_err_msg:Vec<String> =  utils::derive_get_struct_fields(&ast).unwrap().iter().map(|field|{
        format!("Field {} is missing",field.ident.clone().unwrap())
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
        let field_type = &field.ty;
        quote! {
            fn #field_name(&mut self, #field_name: #field_type) -> &mut Self {
                self.#field_name = Some(#field_name);
                self
            }
        }
    }).collect();
    

    let builder_build_method = quote!(
        fn build(&mut self) -> Result<#struct_name, Box<dyn std::error::Error>> {
            #(
                if let None = self.#field_names {
                    return Err(#field_mising_err_msg.into())
                }
            )*
            Ok(
                #struct_name{
                    #(#field_names:self.#field_names.clone().unwrap()),*
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
