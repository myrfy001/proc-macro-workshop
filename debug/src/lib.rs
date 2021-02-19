use proc_macro::TokenStream;

use quote::{quote};
use syn::{DeriveInput, Result, parse_macro_input, parse_quote};

#[proc_macro_derive(CustomDebug, attributes(debug))]
pub fn derive(input: TokenStream) -> TokenStream {
    let st = parse_macro_input!(input as DeriveInput);
    // eprintln!("{:#?}", st);
    do_derive(&st).unwrap_or_else(syn::Error::into_compile_error).into()
}


fn get_all_struct_fields(st: &DeriveInput) -> Option<&syn::punctuated::Punctuated<syn::Field, syn::Token![,]>>{
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
    ) = st.data {
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

fn get_struct_escape_hatch(st: &DeriveInput) -> Option<String> {
    if let Some(inert_attr)  = st.attrs.last(){
        if let Ok(syn::Meta::List(syn::MetaList{nested, ..})) = inert_attr.parse_meta(){
            if let Some(syn::NestedMeta::Meta(syn::Meta::NameValue(path_value))) = nested.last() {
                if path_value.path.is_ident("bound") {
                    if let syn::Lit::Str(ref lit) = path_value.lit{
                        return Some(lit.value())
                    }
                }
            }
        }
    }
    None
}

fn inject_trait_bound(st: &DeriveInput) -> syn::Generics{
    let mut g = st.generics.clone();

    let all_phantomdata_generic_type_ident:Vec<_> = get_all_struct_fields(st).as_ref().unwrap().iter().filter_map(|field|{
        if let syn::Type::Path(syn::TypePath{ref path,..}) = field.ty {
            if let Some(ps) = path.segments.last() {
                if ps.ident == "PhantomData" {
                    if let syn::PathArguments::AngleBracketed(syn::AngleBracketedGenericArguments{ref args, ..}) = ps.arguments {
                        if let Some(syn::GenericArgument::Type(syn::Type::Path(syn::TypePath{path:syn::Path{segments, ..},..}))) = args.last() {
                            if let Some(path_seg) = segments.last() {
                                return Some(&path_seg.ident);
                            }
                        }
                    }
                }
            }
        }
        None
    }).collect();

    let all_associated_types:Vec<_> = get_all_struct_fields(st).as_ref().unwrap().iter().filter_map(|field|{
        if let syn::Type::Path(syn::TypePath{ref path,..}) = field.ty {
            if let Some(ps) = path.segments.last() {
                if let syn::PathArguments::AngleBracketed(syn::AngleBracketedGenericArguments{ref args, ..}) = ps.arguments {
                    if let Some(syn::GenericArgument::Type(t)) = args.last() {
                        if let syn::Type::Path(syn::TypePath{path:syn::Path{segments, ..},..}) = t {
                            if segments.len() >= 2 {
                                return Some((&segments[0].ident, t));
                            }
                        }
                    }
                }
            }
        }
        None
    }).collect();

    

    for gp in &mut g.params{
        if let syn::GenericParam::Type(syn::TypeParam{ident, ..}) = gp {
            if all_phantomdata_generic_type_ident.iter().any(|f| *f == ident) {
                continue
            }

            if all_associated_types.iter().any(|&(idt, _)| idt == ident) {
                continue
            }
        }

        if let syn::GenericParam::Type(syn::TypeParam{ref mut bounds,..}) = gp {
            bounds.push(parse_quote!(std::fmt::Debug));
        }
    }

    let wc = g.make_where_clause();

    for (_, associate_type) in all_associated_types {
        wc.predicates.push(parse_quote!(#associate_type:std::fmt::Debug))
    }
    g
}

fn inject_escape_hatch(st: &DeriveInput, hatch_lit: String) -> syn::Generics{
    let mut g = st.generics.clone();
    let wc = g.make_where_clause();
    let st_node = syn::parse_str(hatch_lit.as_str()).unwrap();
    wc.predicates.push(st_node);
    g
}

fn do_derive(st: &DeriveInput) -> Result<proc_macro2::TokenStream> {
    let target_struct_name_ident = &st.ident;
    let target_struct_name_literal = target_struct_name_ident.to_string();

    let default_fmt_body:Vec<_> = get_all_struct_fields(st).unwrap().iter().map(|field|{
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



    let expanded_generics;
    if let Some(hatch_lit) = get_struct_escape_hatch(st) {
        expanded_generics = inject_escape_hatch(st, hatch_lit);
    } else {
        expanded_generics = inject_trait_bound(st);
    }

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

