use proc_macro::TokenStream;

use quote::ToTokens;
use syn;

#[proc_macro_attribute]
pub fn sorted(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = syn::parse_macro_input!(args as syn::AttributeArgs);
    let st = syn::parse_macro_input!(input as syn::Item);

    match do_sort(&args, &st) {
        Ok(r) => r.into(),
        Err(e) => e.to_compile_error().into()
    }
    
}

fn do_sort(args: &Vec<syn::NestedMeta>,st_root: &syn::Item) -> syn::Result<proc_macro2::TokenStream> {
    let _ = args;
    if let syn::Item::Enum(_) = st_root {}
    else {
        return Err(syn::Error::new(proc_macro2::Span::call_site(), "expected enum or match expression"))
    };
    Ok(st_root.to_token_stream())
}