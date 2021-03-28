use proc_macro::TokenStream;
use quote::{ToTokens, quote};
use syn;

#[proc_macro_attribute]
pub fn bitfield(args: TokenStream, input: TokenStream) -> TokenStream {
    let _ = args;

    let st = syn::parse_macro_input!(input as syn::ItemStruct);
    return generate_real_struct(&st).unwrap_or_else(syn::Error::into_compile_error).into();
 
}


fn get_struct_fields(st: &syn::ItemStruct) -> syn::Result<Vec<&syn::Field>>{
    if let syn::Fields::Named(ref named_fields) = st.fields {
        let t: Vec<_> = named_fields.named.iter().map(|f|f).collect();
        return Ok(t);
    }
    return Err(syn::Error::new_spanned(st,""))
}

fn generate_total_bitwidth_devide_by_8_expression(st: &syn::ItemStruct) -> syn::Result<proc_macro2::TokenStream>{
    let mut plus_part = proc_macro2::TokenStream::new();

    let fields = get_struct_fields(st)?;

    for (idx, field) in fields.iter().enumerate() {
        let field_type = &field.ty;
        if idx == fields.len() - 1  {
            plus_part.extend(quote!(<#field_type as Specifier>::BITS));
        } else {
            plus_part.extend(quote!(<#field_type as Specifier>::BITS + ));
        }
    }

    let ret = quote!((#plus_part) / 8); 
    return Ok(ret)
}

fn generate_real_struct(st: &syn::ItemStruct) -> syn::Result<proc_macro2::TokenStream> {
    let length_expression = generate_total_bitwidth_devide_by_8_expression(st)?;
    let mut st_to_mut = st.clone();
    
    let t:syn::FieldsNamed = syn::parse_quote!({data: [u8; #length_expression],});
    st_to_mut.fields = syn::Fields::Named(t);
    Ok(st_to_mut.to_token_stream())
}





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
