use proc_macro::TokenStream;
use quote::{ToTokens, quote};
use syn;

#[proc_macro_attribute]
pub fn bitfield(args: TokenStream, input: TokenStream) -> TokenStream {
    let _ = args;

    let st = syn::parse_macro_input!(input as syn::ItemStruct);

    eprintln!("==========\n{:#?}", st);
    st.to_token_stream().into()
}


fn get_struct_fields(st: &syn::ItemStruct) -> Result<Vec<&syn::Field>, String>{
    if let syn::Fields::Named(ref named_fields) = st.fields {
        let t: Vec<_> = named_fields.named.iter().map(|f|f).collect();
        return Ok(t);
    }
    return Err("".to_string())
}

// fn calc_total_bit_width(st: &syn::ItemStruct) -> Result<i32, String>{
//     let mut total_bit_width = 0;
//     for field in get_struct_fields(st)? {
//         field
//     }
// }


#[proc_macro]
pub fn define_field_width_enums(_: TokenStream) -> TokenStream {
    let mut out_stream = proc_macro2::TokenStream::new();
    for i in 1..=64usize {  // 这里要指定usize，否则宏展开的时候类型不对
        let enum_item_ident = syn::Ident::new(format!("B{}", i.to_string()).as_str(), proc_macro2::Span::call_site());
        let s  = quote! {
            pub enum #enum_item_ident {}
            impl Specifier for #enum_item_ident {
                const BITS:usize = #i;
            }
        };
        out_stream.extend(s);
    }
    out_stream.into()
}
