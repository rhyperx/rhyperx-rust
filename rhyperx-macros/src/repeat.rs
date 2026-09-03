use std::collections::HashSet;

use proc_macro2::{Ident, Span, TokenStream as TokenStream2, TokenTree};
use quote::{ToTokens, quote};
use syn::visit_mut::VisitMut;
use syn::{
    Field, Fields, ImplItem, Item, LitInt, Token, parenthesized,
    parse::{Parse, ParseStream},
    parse_macro_input,
};

/// Helper to store the parsed arguments of the #[repeat] macro
struct RepeatArgs {
    start: usize,
    end: usize,
    abs_var: String,
    rel_var: String,
    custom_mappings: Vec<(String, String)>,
}

impl Parse for RepeatArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut start: Option<usize> = None;
        let mut end: Option<usize> = None;
        let mut abs_var = "N".to_string();
        let mut rel_var = "I".to_string();
        let mut custom_mappings = Vec::new();

        while !input.is_empty() {
            let name = input.parse::<Ident>()?.to_string();

            if name == "rg" || name == "range" {
                let content;
                parenthesized!(content in input);
                let s: LitInt = content.parse()?;
                content.parse::<Token![..]>()?;
                let e: LitInt = content.parse()?;
                start = Some(s.base10_parse()?);
                end = Some(e.base10_parse()?);
            } else if name == "abs" {
                input.parse::<Token![=]>()?;
                abs_var = input.parse::<syn::LitStr>()?.value();
            } else if name == "rel" {
                input.parse::<Token![=]>()?;
                rel_var = input.parse::<syn::LitStr>()?.value();
            } else {
                input.parse::<Token![=]>()?;
                let val = input.parse::<syn::LitStr>()?.value();
                custom_mappings.push((name, val));
            }

            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
        }

        let start = start.ok_or_else(|| {
            syn::Error::new(Span::call_site(), "missing required range(...) argument")
        })?;
        let end = end.ok_or_else(|| {
            syn::Error::new(Span::call_site(), "missing required range(...) argument")
        })?;

        Ok(RepeatArgs {
            start,
            end,
            abs_var,
            rel_var,
            custom_mappings,
        })
    }
}

pub fn repeat(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let args = parse_macro_input!(attr as RepeatArgs);
    let input_item = parse_macro_input!(item as Item);

    let mut expanded = TokenStream2::new();

    match input_item {
        Item::Struct(mut strct) => {
            let original_fields = strct.fields.clone();

            let expanded_fields = match original_fields {
                Fields::Named(named) => {
                    let mut new_named = syn::FieldsNamed {
                        brace_token: named.brace_token,
                        named: syn::punctuated::Punctuated::new(),
                    };

                    for field in named.named.iter() {
                        if let Some(attr) = field
                            .attrs
                            .iter()
                            .find(|a| a.path().is_ident("repeat_item"))
                        {
                            let local_args = parse_custom_mappings(attr).unwrap().clone();

                            for i in args.start..args.end {
                                let rel = i - args.start;
                                let mut field_clone = field.clone();
                                field_clone
                                    .attrs
                                    .retain(|a| !a.path().is_ident("repeat_item"));

                                let mut mappings = args.custom_mappings.clone();
                                mappings.extend(local_args.clone());

                                field_clone = replace_placeholders_in_field(
                                    field_clone,
                                    i,
                                    rel,
                                    &args,
                                    &mappings,
                                );
                                new_named.named.push(field_clone);
                            }
                        } else {
                            new_named.named.push(field.clone());
                        }
                    }

                    Fields::Named(new_named)
                }
                Fields::Unnamed(unnamed) => {
                    let mut new_unnamed = syn::FieldsUnnamed {
                        paren_token: unnamed.paren_token,
                        unnamed: syn::punctuated::Punctuated::new(),
                    };

                    for field in unnamed.unnamed.iter() {
                        if let Some(attr) = field
                            .attrs
                            .iter()
                            .find(|a| a.path().is_ident("repeat_item"))
                        {
                            let local_args = parse_custom_mappings(attr).unwrap().clone();

                            for i in args.start..args.end {
                                let rel = i - args.start;
                                let mut field_clone = field.clone();
                                field_clone
                                    .attrs
                                    .retain(|a| !a.path().is_ident("repeat_item"));

                                let mut mappings = args.custom_mappings.clone();
                                mappings.extend(local_args.clone());

                                field_clone = replace_placeholders_in_field(
                                    field_clone,
                                    i,
                                    rel,
                                    &args,
                                    &mappings,
                                );
                                new_unnamed.unnamed.push(field_clone);
                            }
                        } else {
                            new_unnamed.unnamed.push(field.clone());
                        }
                    }

                    Fields::Unnamed(new_unnamed)
                }
                Fields::Unit => Fields::Unit,
            };

            strct.fields = expanded_fields;
            expanded.extend(quote!(#strct));
        }
        Item::Impl(mut imp) => {
            let _ = expand_repeat_item_in_impl(&mut imp, &args);

            if let Some(attr) = imp
                .attrs
                .iter()
                .find(|a| a.path().is_ident("repeat_item"))
                .cloned()
            {
                let local_args = match parse_custom_mappings(&attr) {
                    Ok(v) => v,
                    Err(e) => return e.to_compile_error().into(),
                };

                imp.attrs.retain(|a| !a.path().is_ident("repeat_item"));
                let base_impl = strip_repeat_item_attrs_from_impl(imp);

                for i in args.start..args.end {
                    let rel = i - args.start;
                    let mut mappings = args.custom_mappings.clone();
                    mappings.extend(local_args.clone());

                    let processed =
                        replace_tokens(base_impl.to_token_stream(), i, rel, &args, &mappings);
                    let new_impl: syn::ItemImpl = syn::parse2(processed).unwrap();
                    expanded.extend(quote!(#new_impl));
                }
            } else {
                let mut new_items = Vec::new();
                for item in imp.items.iter() {
                    if let ImplItem::Fn(m) = item {
                        // Use #[repeat_item] on methods inside impl to avoid recursive macro expansion.
                        if let Some(attr) =
                            m.attrs.iter().find(|a| a.path().is_ident("repeat_item"))
                        {
                            let local_args = match parse_custom_mappings(attr) {
                                Ok(v) => v,
                                Err(e) => return e.to_compile_error().into(),
                            };

                            for i in args.start..args.end {
                                let rel = i - args.start;
                                let mut method_clone = m.clone();
                                method_clone
                                    .attrs
                                    .retain(|a| !a.path().is_ident("repeat_item"));

                                let mut mappings = args.custom_mappings.clone();
                                mappings.extend(local_args.clone());

                                method_clone = replace_placeholders_in_method(
                                    method_clone,
                                    i,
                                    rel,
                                    &args,
                                    &mappings,
                                );
                                new_items.push(ImplItem::Fn(method_clone));
                            }
                        } else {
                            new_items.push(item.clone());
                        }
                    } else {
                        new_items.push(item.clone());
                    }
                }
                imp.items = new_items;
                expanded.extend(quote!(#imp));
            }
        }
        _ => {
            return syn::Error::new_spanned(input_item, "Repeat only supported on Struct or Impl")
                .to_compile_error()
                .into();
        }
    }

    expanded.into()
}

// Logic to extract key-value pairs from attributes
fn parse_custom_mappings(attr: &syn::Attribute) -> syn::Result<Vec<(String, String)>> {
    if matches!(attr.meta, syn::Meta::Path(_)) {
        return Ok(Vec::new());
    }

    let reserved = HashSet::from(["rg", "range", "abs", "rel"]);
    let mut mappings = Vec::new();
    attr.parse_nested_meta(|meta| {
        if let Some(ident) = meta.path.get_ident()
            && !reserved.contains(&ident.to_string().as_str())
        {
            let name = ident.to_string();
            let val: syn::LitStr = meta.value()?.parse()?;
            mappings.push((name, val.value()));
        }
        Ok(())
    })?;
    Ok(mappings)
}

fn strip_repeat_item_attrs_from_impl(mut imp: syn::ItemImpl) -> syn::ItemImpl {
    for item in imp.items.iter_mut() {
        if let ImplItem::Fn(m) = item {
            m.attrs.retain(|a| !a.path().is_ident("repeat_item"));
        }
    }
    imp
}

fn replace_placeholders_in_field(
    field: Field,
    abs: usize,
    rel: usize,
    args: &RepeatArgs,
    mappings: &[(String, String)],
) -> syn::Field {
    let ts = field.to_token_stream();
    let processed = replace_tokens(ts, abs, rel, args, mappings);

    if field.ident.is_some() {
        let fields: syn::FieldsNamed = syn::parse2(quote!({ #processed })).unwrap();
        fields.named.into_iter().next().unwrap()
    } else {
        let fields: syn::FieldsUnnamed = syn::parse2(quote!(( #processed ))).unwrap();
        fields.unnamed.into_iter().next().unwrap()
    }
}

fn replace_placeholders_in_field_value(
    field: syn::FieldValue,
    abs: usize,
    rel: usize,
    args: &RepeatArgs,
    mappings: &[(String, String)],
) -> syn::FieldValue {
    let ts = field.to_token_stream();
    let processed = replace_tokens(ts, abs, rel, args, mappings);
    syn::parse2(processed).unwrap()
}

fn replace_placeholders_in_method(
    method: syn::ImplItemFn,
    abs: usize,
    rel: usize,
    args: &RepeatArgs,
    mappings: &[(String, String)],
) -> syn::ImplItemFn {
    let ts = method.to_token_stream();
    let processed = replace_tokens(ts, abs, rel, args, mappings);
    syn::parse2(processed).unwrap()
}

fn replace_tokens(
    tokens: TokenStream2,
    abs: usize,
    rel: usize,
    args: &RepeatArgs,
    mappings: &[(String, String)],
) -> TokenStream2 {
    let mut result = Vec::new();
    for token in tokens {
        match token {
            TokenTree::Group(g) => {
                let inner = replace_tokens(g.stream(), abs, rel, args, mappings);
                let mut new_group = proc_macro2::Group::new(g.delimiter(), inner);
                new_group.set_span(g.span());
                result.push(TokenTree::Group(new_group));
            }
            TokenTree::Ident(ident) => {
                let name = ident.to_string();
                let mut replaced = false;

                for (placeholder, template) in mappings {
                    if name == *placeholder {
                        let new_name = template
                            .replace(&format!("${}", args.abs_var), &abs.to_string())
                            .replace(&format!("${}", args.rel_var), &rel.to_string());
                        result.push(TokenTree::Ident(Ident::new(&new_name, ident.span())));
                        replaced = true;
                        break;
                    }
                }

                if !replaced {
                    if name == args.abs_var {
                        result.push(TokenTree::Literal(proc_macro2::Literal::usize_unsuffixed(
                            abs,
                        )));
                    } else if name == args.rel_var {
                        result.push(TokenTree::Literal(proc_macro2::Literal::usize_unsuffixed(
                            rel,
                        )));
                    } else {
                        result.push(TokenTree::Ident(ident));
                    }
                }
            }
            _ => result.push(token),
        }
    }
    result.into_iter().collect()
}

fn expand_repeat_item_in_impl(imp: &mut syn::ItemImpl, args: &RepeatArgs) -> syn::Result<()> {
    let mut expander = RepeatItemExprExpander { args };
    expander.visit_item_impl_mut(imp);
    Ok(())
}

struct RepeatItemExprExpander<'a> {
    args: &'a RepeatArgs,
}

impl<'a> VisitMut for RepeatItemExprExpander<'a> {
    fn visit_expr_struct_mut(&mut self, node: &mut syn::ExprStruct) {
        let mut new_fields = syn::punctuated::Punctuated::new();

        for field in node.fields.iter() {
            if let Some(attr) = field
                .attrs
                .iter()
                .find(|a| a.path().is_ident("repeat_item"))
            {
                let local_args = parse_custom_mappings(attr).unwrap_or_default();

                for i in self.args.start..self.args.end {
                    let rel = i - self.args.start;
                    let mut field_clone = field.clone();
                    field_clone
                        .attrs
                        .retain(|a| !a.path().is_ident("repeat_item"));

                    let mut mappings = self.args.custom_mappings.clone();
                    mappings.extend(local_args.clone());

                    let expanded = replace_placeholders_in_field_value(
                        field_clone,
                        i,
                        rel,
                        self.args,
                        &mappings,
                    );
                    new_fields.push(expanded);
                }
            } else {
                new_fields.push(field.clone());
            }
        }

        node.fields = new_fields;

        syn::visit_mut::visit_expr_struct_mut(self, node);
    }
}
