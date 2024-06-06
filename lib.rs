extern crate proc_macro;
extern crate quote;
extern crate syn;

use std::str::FromStr;

use lazy_static::lazy_static;
use regex::{Captures, Regex};

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, ToTokens};
use syn::{parse_macro_input, ItemFn, Stmt::*};

lazy_static! {
    // Match only few elements
    static ref HTML_TAG_REGEX: Regex = Regex::new(r"<(div|h[1-6])\b[^>]*>").unwrap();
}

/// This macro adds the data-locatorjs-id attribute to all div, h1, h2, etc. elements in the source code.
/// The attribute is used by the LocatorJS library to uniquely identify HTML elements in automated tests.
/// The add_locatorjs_id function takes two arguments: _attr and item.
/// The _attr argument is ignored, and the item argument is expected to be a function item.
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

/// The add_locatorjs_id_to_block function is used to modify the block of the function.
/// It iterates over each statement in the block and checks if it matches one of the specified patterns.
/// If it does, the add_locatorjs_id_to_expr function is called to modify the expression and add the data-locatorjs-id attribute.
fn add_locatorjs_id_to_block(block: Box<syn::Block>) -> Box<syn::Block> {
    let mut new_block = TokenStream2::new();

    for stmt in block.stmts {
        match stmt.clone() {
            Macro(stmt) => {
                let modified_expr = add_locatorjs_id_to_expr(stmt);
                new_block.extend(quote! { #modified_expr });
            }
            Local(local) => {
                let modified_expr = add_locatorjs_id_to_expr(local);
                new_block.extend(quote! { #modified_expr });
            }
            Item(item) => {
                let modified_expr = add_locatorjs_id_to_expr(item);
                new_block.extend(quote! { #modified_expr });
            }
            Expr(expr, semi) => {
                let modified_expr = add_locatorjs_id_to_expr(expr);
                new_block.extend(quote! { #modified_expr #semi });
            }
        }
    }

    Box::new(syn::parse_quote!({ #new_block }))
}

/// The add_locatorjs_id_to_expr function is used to modify an expression.
/// It iterates over each token in the expression and checks if it matches the HTML_TAG_REGEX pattern.
/// If it does, the add_locatorjs_id_to_macro_tokens function is called to modify the token and add the data-locatorjs-id attribute.
fn add_locatorjs_id_to_expr(expr: impl ToTokens) -> TokenStream2 {
    let mut new_tokens = TokenStream2::new();
    let tokens = expr.into_token_stream();

    for token in tokens {
        let token_str = token.clone().to_string();

        if HTML_TAG_REGEX.is_match(&token_str) {
            let macro_tokens = token.clone().into_token_stream();
            let modified_tokens = add_locatorjs_id_to_macro_tokens(macro_tokens);
            new_tokens.extend(modified_tokens);
        } else {
            new_tokens.extend(token.into_token_stream());
        }
    }

    new_tokens
}

/// The add_locatorjs_id_to_macro_tokens function is used to modify a token that matches the HTML_TAG_REGEX pattern.
/// It replaces the closing angle bracket > with the data-locatorjs-id attribute and the closing angle bracket.
fn add_locatorjs_id_to_macro_tokens(tokens: TokenStream2) -> TokenStream2 {
    let mut new_tokens = TokenStream2::new();

    for token in tokens {
        let tokens_block = token.clone().into_token_stream();
        let tokens_a = tokens_block.to_string();
        let locatorjs = HTML_TAG_REGEX.replace_all(&tokens_a, |captures: &Captures| {
            let tag_elm = &captures[0];
            let locatorjs = format!(
                " attr:data-locatorjs-id=\"{}::{}\">",
                "source_file", "file_line"
            );
            return tag_elm.replace(">", &locatorjs);
        });

        if let Ok(token_locatorjs) = TokenStream2::from_str(&locatorjs) {
            new_tokens.extend(quote! { #token_locatorjs });
        }
    }

    new_tokens
}
