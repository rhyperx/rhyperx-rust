use proc_macro::TokenStream;
use quote::quote;
use syn::{
    ItemMod, Meta, Token,
    parse::{Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
};

struct Args {
    extra_attrs: Vec<syn::Attribute>,
}

impl Parse for Args {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let vars = Punctuated::<Meta, Token![,]>::parse_terminated(input)?;
        let mut extra_attrs = Vec::new();

        for meta in vars {
            if meta.path().is_ident("attr") {
                // Extract what's inside the parentheses of attr(...)
                if let Meta::List(list) = meta {
                    let inner_tokens = &list.tokens;
                    extra_attrs.push(syn::parse_quote!(#[#inner_tokens]));
                }
            }
        }
        Ok(Args { extra_attrs })
    }
}

pub fn hoist_mod(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemMod);

    let args = parse_macro_input!(attr as Args);
    let items = input
        .content
        .expect("module must be inline")
        .1
        .into_iter()
        .map(|item| {
            let extra_attrs = &args.extra_attrs;
            let x = quote! {
                #(#extra_attrs)*
                #item
            };
            x
        });

    TokenStream::from(quote! {
        #(#items)*
    })
}
