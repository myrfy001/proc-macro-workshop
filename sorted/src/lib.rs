use proc_macro::TokenStream;

use quote::ToTokens;
use syn::{self};

#[proc_macro_attribute]
pub fn sorted(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = syn::parse_macro_input!(args as syn::AttributeArgs);
    let st = syn::parse_macro_input!(input as syn::Item);

    match do_sort_check(&args, &st) {
        Ok(_) => st.into_token_stream().into(),
        Err(e) => {
            // let mut new_stream = st.into_token_stream();
            // new_stream.extend(e.to_compile_error());
            // new_stream.into()

            let mut new_stream =e.to_compile_error();
            new_stream.extend(st.to_token_stream());
            new_stream.into()
        }

    }
    
}

fn do_sort_check(args: &Vec<syn::NestedMeta>,st_root: &syn::Item) -> syn::Result<()> {
    let _ = args;
    if let syn::Item::Enum(e) = st_root {
        check_sort(e)?;
    }
    else {
        return Err(syn::Error::new(proc_macro2::Span::call_site(), "expected enum or match expression"))
    };
    Ok(())
}

fn check_sort(enum_body: &syn::ItemEnum) -> syn::Result<()> {
    if enum_body.variants.len() <= 1 {
        return Ok(())
    }

    let origin_name_order:Vec<(String, &syn::Ident)> = enum_body.variants.iter().map(|f|(f.ident.to_string(), &f.ident)).collect();
    let mut sorted_name = origin_name_order.clone();
    sorted_name.sort();

    for i in 0..enum_body.variants.len() {
        if origin_name_order[i].0 != sorted_name[i].0 {
           let err_msg = format!("{} should sort before {}", sorted_name[i].0, origin_name_order[i].0);
            return Err(syn::Error::new(sorted_name[i].1.span(), err_msg))
        }
    }
    Ok(())
}