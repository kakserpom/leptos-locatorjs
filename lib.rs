extern crate proc_macro;
extern crate quote;
extern crate syn;

use proc_macro::TokenStream;
use proc_macro2::{Group, TokenTree};
use quote::{quote, ToTokens};
use syn::{parse_macro_input, Expr, ExprMacro, ItemFn};

#[proc_macro_attribute]
pub fn add_locatorjs_id(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);
    let ItemFn {
        attrs,
        vis,
        sig,
        block,
    } = input;

    // Modify the block to add the `data-locatorjs-id` attribute
    let modified_block = add_locatorjs_id_to_block(block);

    let output = quote! {
        #(#attrs)*
        #vis #sig
        #modified_block
    };

    TokenStream::from(output)
}

fn add_locatorjs_id_to_block(block: Box<syn::Block>) -> Box<syn::Block> {
    let mut new_block = proc_macro2::TokenStream::new();

    for stmt in block.stmts {
        match stmt {
            syn::Stmt::Expr(expr, semi) => {
                let modified_expr = add_locatorjs_id_to_expr(expr);
                new_block.extend(quote! { #modified_expr #semi });
            }
            _ => new_block.extend(stmt.into_token_stream()),
        }
    }

    Box::new(syn::parse_quote!({ #new_block }))
}

fn add_locatorjs_id_to_expr(expr: Expr) -> proc_macro2::TokenStream {
    match expr {
        Expr::Macro(ExprMacro {
            ref attrs, ref mac, ..
        }) => {
            let macro_path = mac.path.to_token_stream().to_string();
            if macro_path == "view" {
                let macro_tokens = mac.tokens.clone();
                let modified_tokens = add_locatorjs_id_to_macro_tokens(macro_tokens);

                quote! {
                    #(#attrs)*
                    view! { #modified_tokens }
                }
            } else {
                expr.into_token_stream()
            }
        }
        _ => expr.into_token_stream(),
    }
}

fn add_locatorjs_id_to_macro_tokens(tokens: proc_macro2::TokenStream) -> proc_macro2::TokenStream {
    let mut new_tokens = proc_macro2::TokenStream::new();

    for token in tokens {
        match token {
            TokenTree::Group(group) => {
                let new_group = Group::new(
                    group.delimiter(),
                    add_locatorjs_id_to_macro_tokens(group.stream()),
                );
                new_tokens.extend(TokenTree::Group(new_group).into_token_stream());
            }
            TokenTree::Ident(ref ident) => {
                let ident_str = ident.to_string();
                if ident_str == "div"
                    || ident_str == "h1"
                    || ident_str.starts_with('h')
                        && ident_str.len() == 2
                        && ident_str.chars().nth(1).unwrap().is_numeric()
                {
                    new_tokens.extend(quote! {
                        #ident data-locatorjs-id={concat!(file!(), "::", line!())}
                    });
                } else {
                    new_tokens.extend(token.into_token_stream());
                }
            }
            _ => new_tokens.extend(token.into_token_stream()),
        }
    }

    new_tokens
}
