use proc_macro::TokenStream;

use quote::quote;
use syn::{parse_macro_input, DeriveInput, Result};

#[proc_macro_derive(CustomDebug)]
pub fn derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    do_derive(&ast).unwrap_or_else(syn::Error::into_compile_error).into()
}


fn get_all_struct_fields(ast: &DeriveInput) -> Option<&syn::punctuated::Punctuated<syn::Field, syn::Token![,]>>{
    if let syn::Data::Struct(
        syn::DataStruct{
            fields: syn::Fields::Named(
                syn::FieldsNamed{
                    ref named,
                    ..
                },
            ),
            ..
        }
    ) = ast.data {
        return Some(named)
    }
    None
}

fn do_derive(ast: &DeriveInput) -> Result<proc_macro2::TokenStream> {

    let target_struct_name_ident = &ast.ident;
    let target_struct_name_literal = target_struct_name_ident.to_string();

    let default_fmt_body:Vec<_> = get_all_struct_fields(ast).unwrap().iter().map(|field|{
        let field_name_ident = &field.ident;
        let field_name_literal = field_name_ident.as_ref().unwrap().to_string();
        quote!(
            .field(#field_name_literal ,&self.#field_name_ident)
        )
    }).collect();



    return Ok(quote! {
        impl std::fmt::Debug for #target_struct_name_ident {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.debug_struct(#target_struct_name_literal)
                #(#default_fmt_body)*                    
                 .finish()
            }
        }
    });
}

