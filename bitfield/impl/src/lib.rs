use proc_macro::TokenStream;
use quote::quote;
use syn::spanned::Spanned;

#[proc_macro_attribute]
pub fn bitfield(args: TokenStream, input: TokenStream) -> TokenStream {
    let _ = args;
    let _ = input;

    unimplemented!()
}

#[proc_macro]
pub fn define_field_width_enums(_: TokenStream) -> TokenStream {
    let mut out_stream = proc_macro2::TokenStream::new();
    for i in 1..=64 {
        let enum_item_ident = syn::Ident::new(format!("B{}", i.to_string()).as_str(), proc_macro2::Span::call_site());
        let s  = quote! {
            pub enum #enum_item_ident {}
            impl Specifier for #enum_item_ident {
                const BITS:i32 = #i;
            }
        };
        out_stream.extend(s);
    }
    out_stream.into()
}