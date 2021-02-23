use std::iter::FromIterator;

use proc_macro::TokenStream;
use proc_macro2::{self};
use quote::{self, ToTokens, TokenStreamExt};
use syn::{self};
#[proc_macro]
pub fn seq(input: TokenStream) -> TokenStream {
    let ref st = syn::parse_macro_input!(input as SeqDef);

    // eprintln!("----===={:#?}", st);
    
    let (mut ts,expanded) = st.travel_and_expand_blocks(&st.body);
    if !expanded {
        ts = st.expand_single_seq_block(&st.body);
    }

    // eprintln!("----===={:#?}", ts);

    ts.into()
}

impl SeqDef {


    fn travel_and_expand_blocks(&self, ts: &proc_macro2::TokenStream ) -> (proc_macro2::TokenStream, bool) {
        let mut expanded = false;
        let mut new_stream = proc_macro2::TokenStream::new();
        let tokens = Vec::from_iter(ts.clone().into_iter());
        let mut token_idx = 0;
        while token_idx < tokens.len() {
            let tree = &tokens[token_idx];
            // eprintln!("----===={:#?}", tree);
            let new_stream_part = match tree {
                proc_macro2::TokenTree::Punct(pstart) if pstart.to_string() == "#" => {
                    if let proc_macro2::TokenTree::Group(ref g) = &tokens[token_idx+1] {
                        if let proc_macro2::TokenTree::Punct(pend) = &tokens[token_idx+2] {
                            if pend.to_string() == "*" {
                                token_idx +=2;
                                expanded = true;
                                
                                self.expand_single_seq_block(&g.stream())
                            } else {tree.to_token_stream()}
                        } else {tree.to_token_stream()}
                    } else {tree.to_token_stream()}
                },
                proc_macro2::TokenTree::Group(g) => {
                    let (r, e )= self.travel_and_expand_blocks(&g.stream());
                    expanded = e;
                    proc_macro2::Group::new(g.delimiter(), r).to_token_stream()
                }
                _ => tree.to_token_stream(),
            };
            

            new_stream.append_all(new_stream_part);
            token_idx += 1;
        };
        (new_stream, expanded)
    }

    fn expand_single_seq_block(&self, ts: &proc_macro2::TokenStream) -> proc_macro2::TokenStream {
        let mut r = proc_macro2::TokenStream::new();
        for idx in self.start_num..self.stop_num {
            let new_part = self.replace_ident(ts, idx, 0);
            r.extend(new_part);
        }
        // eprintln!("{:#?}", r);
        r
    }

    fn replace_ident(
        &self,
        ts: &proc_macro2::TokenStream,
        i: i64,
        depth: i64,
    ) -> proc_macro2::TokenStream {
        let mut new_stream = proc_macro2::TokenStream::new();

        let tokens = Vec::from_iter(ts.clone().into_iter());
        let mut token_idx = 0;
        while token_idx < tokens.len() {
            let tree = &tokens[token_idx];
            let new_stream_part = match tree {
                proc_macro2::TokenTree::Group(g) => proc_macro2::Group::new(
                    g.delimiter(),
                    self.replace_ident(&g.stream(), i, depth + 1),
                )
                .to_token_stream(),
                proc_macro2::TokenTree::Ident(_) => {
                    if let Some(t) = SeqDef::try_get_seq_ident(
                        &mut token_idx,
                        &tokens,
                        self.ident.to_string(),
                        i,
                    ) {
                        t
                    } else {
                        tree.to_token_stream()
                    }
                }
                _ => tree.to_token_stream(),
            };
            new_stream.append_all(new_stream_part);
            token_idx += 1;
        }
        new_stream
    }

    fn try_get_seq_ident(
        token_idx: &mut usize,
        tokens: &Vec<proc_macro2::TokenTree>,
        target_ident_str: String,
        i: i64,
    ) -> Option<proc_macro2::TokenStream> {
        if let proc_macro2::TokenTree::Ident(ref ident_1) = tokens[*token_idx] {
            if ident_1.to_string() == target_ident_str {
                return Some(proc_macro2::Literal::i64_unsuffixed(i).to_token_stream());
            }
        }

        if *token_idx + 3 > tokens.len() {
            return None;
        }

        if let proc_macro2::TokenTree::Ident(ref ident_1) = tokens[*token_idx] {
            if let proc_macro2::TokenTree::Punct(ref p) = tokens[*token_idx + 1] {
                if p.as_char() != char::from('#') {
                    return None;
                }
                if let proc_macro2::TokenTree::Ident(ref ident_2) = tokens[*token_idx + 2] {
                    if ident_2.to_string() != target_ident_str {
                        return None;
                    }
                    if let proc_macro2::TokenTree::Punct(ref p) = tokens[*token_idx + 3] {
                        if p.as_char() == char::from('#') {
                            if *token_idx + 4 <= tokens.len() {
                                if let proc_macro2::TokenTree::Ident(ref ident_3) =
                                    tokens[*token_idx + 4]
                                {
                                    if ident_3.span().start() == p.span().end() {
                                        *token_idx += 4;
                                        let xx = Some(
                                            proc_macro2::Ident::new(
                                                format!(
                                                    "{}{}{}",
                                                    ident_1.to_string(),
                                                    i.to_string(),
                                                    ident_3.to_string()
                                                )
                                                .as_str(),
                                                p.span(),
                                            )
                                            .to_token_stream(),
                                        );
                                        return xx;
                                    }
                                }
                            }
                            *token_idx += 3;
                            return Some(
                                proc_macro2::Ident::new(
                                    format!("{}{}", ident_1.to_string(), i.to_string()).as_str(),
                                    p.span(),
                                )
                                .to_token_stream(),
                            );
                        } else {
                            *token_idx += 2;
                            return Some(
                                proc_macro2::Ident::new(
                                    format!(
                                        "{}{}",
                                        ident_1.to_string(),
                                        i.to_string(),
                                    )
                                    .as_str(),
                                    p.span(),
                                )
                                .to_token_stream()
                            )
                        }
                        
                    } else {
                        *token_idx += 2;
                        return Some(
                            proc_macro2::Ident::new(
                                format!("{}{}", ident_1.to_string(), i.to_string()).as_str(),
                                p.span(),
                            )
                            .to_token_stream(),
                        );
                    }

                }
            }
        };
        None
    }
}

#[derive(Debug)]
struct SeqDef {
    ident: syn::Ident,
    start_num: i64,
    stop_num: i64,
    body: proc_macro2::TokenStream,
    // body: syn::Block,
}

impl syn::parse::Parse for SeqDef {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut inclusive = false;

        let ident: syn::Ident = input.parse()?;
        input.parse::<syn::Token![in]>()?;
        let start_num_lit: syn::LitInt = input.parse()?;
        input.parse::<syn::Token![..]>()?;

        
        if input.peek(syn::Token!(=)) {
            inclusive = true;
            input.parse::<syn::Token![=]>()?;
        }

        let stop_num_lit: syn::LitInt = input.parse()?;

        let body;
        syn::braced!(body in input);
        let body: proc_macro2::TokenStream = body.parse()?;
        // 上面的可以替换下面的吗？注意区别
        // let body: syn::Block = body.parse()?;

        // eprintln!("===={:#?}", body);

        let mut ret = SeqDef {
            ident,
            start_num: start_num_lit.base10_parse::<i64>()?,
            stop_num: stop_num_lit.base10_parse::<i64>()?,
            body,
        };
        if inclusive {
            ret.stop_num += 1;
        }

        Ok(ret)
    }
}
