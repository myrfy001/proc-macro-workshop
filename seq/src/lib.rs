use std::{iter::FromIterator};

use proc_macro::TokenStream;
use proc_macro2;
use quote::{self, ToTokens};
use syn::{self};
#[proc_macro]
pub fn seq(input: TokenStream) -> TokenStream {
    let ref st = syn::parse_macro_input!(input as SeqDef);
    let mut r = proc_macro2::TokenStream::new();
    
    for idx in st.start_num .. st.stop_num {
        let new_part = expand(&st.body, &st.ident, idx );
        r.extend(new_part);
    }
    
    // eprintln!("----===={:#?}", r);

    r.into()
}


fn expand(ts: &proc_macro2::TokenStream, target_ident: &proc_macro2::Ident, i: i64) -> proc_macro2::TokenStream {
    let new_stream = proc_macro2::TokenStream::from_iter(
     ts.into_token_stream().into_iter().map(|ref tree|{
        match tree {
            proc_macro2::TokenTree::Group(g) => {
                proc_macro2::Group::new(
                    g.delimiter(), 
                    expand(&g.stream(), target_ident, i)
                ).to_token_stream()
            },
            proc_macro2::TokenTree::Ident(idt) => {
                if idt.to_string() == target_ident.to_string() {
                    proc_macro2::Literal::i64_unsuffixed(i).to_token_stream()
                } else {
                    tree.to_token_stream()
                }
            },
            _ => {tree.to_token_stream()}
        }
    }));
    new_stream
    
}


struct SeqDef {
    ident: syn::Ident,
    start_num: i64,
    stop_num: i64,
    body: proc_macro2::TokenStream,
    // body: syn::Block,
}

impl syn::parse::Parse for SeqDef {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self>{
        let ident: syn::Ident = input.parse()?;
        input.parse::<syn::Token![in]>()?;
        let start_num_lit:syn::LitInt = input.parse()?;
        input.parse::<syn::Token![..]>()?;
        let stop_num_lit:syn::LitInt = input.parse()?;

        let body;
        syn::braced!(body in input);
        let body: proc_macro2::TokenStream = body.parse()?; 
        // 上面的可以替换下面的吗？注意区别
        // let body: syn::Block = body.parse()?; 
        
        // eprintln!("===={:#?}", body);

        return Ok(SeqDef{
            ident,
            start_num:start_num_lit.base10_parse::<i64>()?,
            stop_num:stop_num_lit.base10_parse::<i64>()?,
            body,
        })
    }
}