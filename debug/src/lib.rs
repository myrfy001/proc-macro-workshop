use proc_macro::TokenStream;

use quote::{quote};
use syn::{DeriveInput, Result, parse_macro_input, parse_quote};

#[proc_macro_derive(CustomDebug, attributes(debug))]
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

fn get_inert_attribute_of_field(field:&syn::Field) -> Option<String> {
    if let Some(inert_attr)  = field.attrs.last(){
        if let Ok(syn::Meta::NameValue(inert_path_value)) = inert_attr.parse_meta(){
            if inert_path_value.path.is_ident("debug") {
                if let syn::Lit::Str(lit) = inert_path_value.lit{
                    return Some(lit.value())
                }
            }
        }
    }
    None
}

fn inject_trait_bound(generics: &syn::Generics) -> syn::Generics{
    let mut g = generics.clone();
    for gp in &mut g.params{
        if let syn::GenericParam::Type(syn::TypeParam{ref mut bounds,..}) = gp {
            bounds.push(parse_quote!(std::fmt::Debug));
        }
    }
    g
}

fn do_derive(ast: &DeriveInput) -> Result<proc_macro2::TokenStream> {
    let target_struct_name_ident = &ast.ident;
    let target_struct_name_literal = target_struct_name_ident.to_string();

    let default_fmt_body:Vec<_> = get_all_struct_fields(ast).unwrap().iter().map(|field|{
        let field_name_ident = &field.ident;
        let field_name_literal = field_name_ident.as_ref().unwrap().to_string();

        if let Some(format_literal) = get_inert_attribute_of_field(field) {
            quote!(
                .field(#field_name_literal, &format_args!(#format_literal ,&self.#field_name_ident))
            )
        } else {
            quote!(
                .field(#field_name_literal ,&self.#field_name_ident)
            )
        }
        
    }).collect();

    let expanded_generics = inject_trait_bound(&ast.generics);

    let (impl_generics, ty_generics, where_clause) = expanded_generics.split_for_impl();

    

    return Ok(quote! {
        impl #impl_generics std::fmt::Debug for #target_struct_name_ident #ty_generics #where_clause {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.debug_struct(#target_struct_name_literal)
                #(#default_fmt_body)*                    
                 .finish()
            }
        }
    });
}

