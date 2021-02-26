use proc_macro::TokenStream;

use quote::ToTokens;
use syn::{self, visit_mut::VisitMut};

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

            let mut new_stream = e.to_compile_error();
            new_stream.extend(st.to_token_stream());
            new_stream.into()
        }
    }
}

fn do_sort_check(args: &Vec<syn::NestedMeta>, st_root: &syn::Item) -> syn::Result<()> {
    let _ = args;
    if let syn::Item::Enum(e) = st_root {
        check_sort(e)?;
    } else {
        return Err(syn::Error::new(
            proc_macro2::Span::call_site(),
            "expected enum or match expression",
        ));
    };
    Ok(())
}

fn check_sort(enum_body: &syn::ItemEnum) -> syn::Result<()> {
    if enum_body.variants.len() <= 1 {
        return Ok(());
    }

    let origin_name_order: Vec<(String, &syn::Ident)> = enum_body
        .variants
        .iter()
        .map(|f| (f.ident.to_string(), &f.ident))
        .collect();
    let mut sorted_name = origin_name_order.clone();
    sorted_name.sort();

    for i in 0..enum_body.variants.len() {
        if origin_name_order[i].0 != sorted_name[i].0 {
            let err_msg = format!(
                "{} should sort before {}",
                sorted_name[i].0, origin_name_order[i].0
            );
            return Err(syn::Error::new(sorted_name[i].1.span(), err_msg));
        }
    }
    Ok(())
}

#[proc_macro_attribute]
pub fn check(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = syn::parse_macro_input!(args as syn::AttributeArgs);
    let mut st = syn::parse_macro_input!(input as syn::ItemFn);

    match do_check_check(&args, &mut st) {
        Ok(_) => st.into_token_stream().into(),
        Err(e) => {
            let mut new_stream = e.to_compile_error();
            new_stream.extend(st.to_token_stream());
            new_stream.into()
        }
    }
}

struct FnVisitor {
    err: syn::Result<()>,
}

impl syn::visit_mut::VisitMut for FnVisitor {
    fn visit_expr_match_mut(&mut self, node: &mut syn::ExprMatch) {
        let maybe_idx = node.attrs.iter().position(|a| {
            if let Some(p) = a.path.segments.last() {
                if p.ident.to_string() == "sorted" {
                    return true;
                }
            }
            return false;
        });
        if let Some(idx) = maybe_idx {
            node.attrs.remove(idx);
        }

        let mut not_supported_arm:  Option<&syn::Arm> = None;
        let org_order: Vec<(String, &syn::Path)> = node
            .arms
            .iter()
            .filter_map(|a| {
                match a.pat {
                    syn::Pat::TupleStruct(ref pts) => {
                        return Some((join_path_segments(&pts.path), &pts.path ));
                    }
                    syn::Pat::Path(ref pp) => {
                        return Some((join_path_segments(&pp.path), &pp.path ));
                    }
                    syn::Pat::Struct(ref ps) => {
                        return Some((join_path_segments(&ps.path), &ps.path));
                    }
                    _ => {if not_supported_arm.is_none() {not_supported_arm = Some(a)}  ;return None},
                }
            })
            .collect();

        if let Some(a) = not_supported_arm {
            self.err = Err(syn::Error::new_spanned(&a.pat, "unsupported by #[sorted]"));
                return;
        }

        let mut sorted_order = org_order.clone();
        sorted_order.sort_by(|a, b| a.0.cmp(&b.0));

        for i in 0..sorted_order.len() {
            if org_order[i].0 != sorted_order[i].0 {
                let err_msg = format!(
                    "{} should sort before {}",
                    sorted_order[i].0, org_order[i].0
                );

                self.err = Err(syn::Error::new_spanned(sorted_order[i].1, err_msg));
                return;
            }
        }

        syn::visit_mut::visit_expr_match_mut(self, node)
    }
}

fn do_check_check(args: &Vec<syn::NestedMeta>, st_root: &mut syn::ItemFn) -> syn::Result<()> {
    let _ = args;
    let mut visitor = FnVisitor { err: Ok(()) };
    visitor.visit_item_fn_mut(st_root);
    return visitor.err;
}


fn join_path_segments(path: &syn::Path) -> String {
    let mut ret = String::new();
    for ref seg in path.segments.pairs(){
        match seg{
            syn::punctuated::Pair::Punctuated( a,_) => {
                ret += &a.ident.to_string();
                ret += "::";
            }
            syn::punctuated::Pair::End(a) => {
                ret += &a.ident.to_string();
            }
        }
    }
    ret
}