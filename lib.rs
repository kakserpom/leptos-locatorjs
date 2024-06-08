use lazy_static::lazy_static;
use proc_macro2::{Group, TokenStream, TokenTree};
use quote::{quote, ToTokens};
use regex::Regex;
use syn::{parse_macro_input, ItemFn, Stmt};

lazy_static! {
    // Match only few elements
    static ref HTML_REGEX: Regex = Regex::new(r"<(div|h[1-6])\b[^>]*>").unwrap();
    static ref HTML_TAG_REGEX: Regex = Regex::new(r"div|h[1-6]").unwrap();
}

// https://discord.com/channels/1031524867910148188/1247813845414707280
/// This macro adds the data-locatorjs-id attribute to all div, h1, h2, etc. elements in the source code.
/// The attribute is used by the LocatorJS library to uniquely identify HTML elements in automated tests.
///
/// ### Exemple
///
/// ```rust
/// #[component]
/// #[leptos_locatorjs::add_locatorjs_id]
/// pub fn Example() -> impl IntoView {
///     let (count, _) = create_signal(2);
///     // pause 5 seconds
///     let ressource = create_resource(|| (), |_| async move { pray_me().await });
///
///     let hello_word = move || {
///         let my_count = count.get();
///         match my_count {
///             2 => view! {<h2>"Hello, world!"</h2>},
///             _ => view! {<h2>"Burn, world!"</h2>},
///         }
///     };
///
///     let god____where_r_u = move || {
///         let _son_______i_am_everywhere = ressource.get();
///         "Je suis là, mon fils"
///     };
///
///     view! {
///         <div>
///             <div>{hello_word}</div>
///             <Suspense fallback=|| view!{ <div>"Loading..."</div> }>
///                 <ul>
///                     <li>"I like banana."</li>
///                     <li>{god____where_r_u}</li>
///                 </ul>
///             </Suspense>
///         </div>
///     }
/// }
/// ```
///
#[proc_macro_attribute]
pub fn add_locatorjs_id(
    _attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
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

    proc_macro::TokenStream::from(output)
}

/// The add_locatorjs_id_to_block function is used to modify the block of the function.
/// It iterates over each statement in the block and checks if it matches one of the specified patterns.
/// If it does, the add_locatorjs_id_to_expr function is called to modify the expression and add the data-locatorjs-id attribute.
fn add_locatorjs_id_to_block(block: Box<syn::Block>) -> Box<syn::Block> {
    let mut new_block = TokenStream::new();

    for stmt in block.stmts {
        match stmt.clone() {
            Stmt::Macro(stmt) => {
                let modified_expr = add_locatorjs_id_to_expr(stmt);
                new_block.extend(quote! { #modified_expr });
            }
            Stmt::Local(local) => {
                let modified_expr = add_locatorjs_id_to_expr(local);
                new_block.extend(quote! { #modified_expr });
            }
            Stmt::Item(item) => {
                let modified_expr = add_locatorjs_id_to_expr(item);
                new_block.extend(quote! { #modified_expr });
            }
            Stmt::Expr(expr, semi) => {
                let modified_expr = add_locatorjs_id_to_expr(expr);
                new_block.extend(quote! { #modified_expr #semi });
            }
        }
    }

    Box::new(syn::parse_quote!({ #new_block }))
}

/// The add_locatorjs_id_to_expr function is used to modify an expression.
/// It iterates over each token in the expression and checks if it matches the HTML_REGEX pattern.
/// If it does, the add_locatorjs_id_to_macro_tokens function is called to modify the token and add the data-locatorjs-id attribute.
fn add_locatorjs_id_to_expr(expr: impl ToTokens) -> TokenStream {
    let mut new_tokens = TokenStream::new();
    let tokens = expr.into_token_stream();

    for token in tokens {
        let token_str = token.clone().to_string();

        if HTML_REGEX.is_match(&token_str) {
            let macro_tokens = token.clone().into_token_stream();
            let modified_tokens = add_locatorjs_id_to_macro_tokens(macro_tokens);
            new_tokens.extend(modified_tokens);
        } else {
            new_tokens.extend(token.into_token_stream());
        }
    }

    new_tokens
}

/// Modifies HTML tags in a `TokenStream` by adding a `data-locatorjs-id` attribute.
///
/// This function iterates over the tokens in the input `TokenStream`, and for any token that matches the `HTML_TAG_REGEX` pattern,
/// it adds the `data-locatorjs-id` attribute next to the tag.
fn add_locatorjs_id_to_macro_tokens(tokens: TokenStream) -> TokenStream {
    let mut new_tokens = TokenStream::new();

    // Keep track of the last two characters encountered in the TokenStream.
    // This is used to determine if a token is the start of an HTML tag.
    let mut match_token: Vec<String> = vec![];

    for token in tokens.clone() {
        match token {
            TokenTree::Group(group) => {
                let new_group = Group::new(
                    group.delimiter(),
                    add_locatorjs_id_to_macro_tokens(group.stream()),
                );
                new_tokens.extend(TokenTree::Group(new_group).into_token_stream());
            }
            TokenTree::Ident(ident) => {
                let tag = ident.clone().to_string();
                if HTML_TAG_REGEX.is_match(&tag) {
                    if let Some(tag_start) = match_token.last() {
                        // If the last character was a "<", this is the start of an HTML tag.
                        // Clear the match_token vector and add the data-locatorjs-id attribute to the tag.
                        if tag_start.eq("<") {
                            match_token.clear();
                            let attr_locatorjs = format!("{}::{}", "path_to_file", "line_number");
                            new_tokens
                                .extend(quote! { #ident attr:data-locatorjs-id=#attr_locatorjs });
                        } else {
                            // The character was a "/" or others.
                            new_tokens.extend(quote! { #ident });
                        }
                    }
                } else {
                    new_tokens.extend(quote! { #ident });
                }
            }
            TokenTree::Punct(punct) => {
                let char = punct.clone().to_string();
                // If the token is a punctuation "<" or "/" character.
                // If it is, add it to the match_token vector.
                if char.eq("<") | char.eq("/") {
                    match_token.push(char);
                }
                new_tokens.extend(quote! { #punct });
            }
            TokenTree::Literal(literal) => new_tokens.extend(quote! { #literal }),
        }
    }

    new_tokens
}
