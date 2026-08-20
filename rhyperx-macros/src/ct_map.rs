use proc_macro2::{Span, TokenStream};
use quote::{ToTokens, format_ident, quote};
use syn::{
    Expr, ExprRange, Ident, ItemStruct, ItemTrait, Path, TraitItem, TraitItemFn, Type,
    parenthesized, parse::Parse, parse::ParseStream, parse_macro_input, spanned::Spanned,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SkipKind {
    New,
    InternalGetters,
    TotalSizeFn,
    All,
}

impl SkipKind {
    fn parse(ident: &Ident) -> syn::Result<Self> {
        match ident.to_string().as_str() {
            "new" => Ok(SkipKind::New),
            "internal_getters" => Ok(SkipKind::InternalGetters),
            "total_size_fn" => Ok(SkipKind::TotalSizeFn),
            "all" => Ok(SkipKind::All),
            _ => Err(syn::Error::new(ident.span(), "unknown skip option")),
        }
    }
}

struct GenFlags {
    new: bool,
    internal_getters: bool,
    total_size_fn: bool,
}
impl Default for GenFlags {
    fn default() -> Self {
        Self {
            new: true,
            internal_getters: true,
            total_size_fn: true,
        }
    }
}

struct Args {
    ty: Type,
    range: ExprRange,
    default_expr: Option<Expr>,
    allocator: Option<Type>,
    skip: Vec<SkipKind>,
}

struct ArgsFinalized {
    ty: Type,
    range_start: usize,
    range_end: usize,
    default_expr: Expr,
    allocator: Type,
    gen_flags: GenFlags,
}

impl Parse for Args {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut ty = None;
        let mut range = None;
        let mut default_expr = None;
        let mut allocator = None;
        let mut skip = Vec::new();

        while !input.is_empty() {
            let ident: Ident = input.parse()?;
            let content;
            parenthesized!(content in input);

            match ident.to_string().as_str() {
                "ty" => ty = Some(content.parse()?),
                "rg" => range = Some(content.parse()?),
                "default" => default_expr = Some(content.parse()?),
                "allocator" => allocator = Some(content.parse()?),
                "skip" => {
                    let list =
                        syn::punctuated::Punctuated::<Ident, syn::Token![,]>::parse_terminated(
                            &content,
                        )?;
                    if list.is_empty() {
                        return Err(syn::Error::new(
                            ident.span(),
                            "skip(...) requires at least one option",
                        ));
                    }
                    for item in list {
                        skip.push(SkipKind::parse(&item)?);
                    }
                }
                _ => return Err(syn::Error::new(ident.span(), "unknown ct_map option")),
            }
            if input.peek(syn::Token![,]) {
                input.parse::<syn::Token![,]>()?;
            }
        }

        Ok(Self {
            ty: ty.ok_or_else(|| syn::Error::new(Span::call_site(), "missing ty(...)"))?,
            range: range.ok_or_else(|| syn::Error::new(Span::call_site(), "missing rg(...)"))?,
            default_expr,
            allocator,
            skip,
        })
    }
}

impl Args {
    fn finalize(self) -> syn::Result<ArgsFinalized> {
        let start_expr = self
            .range
            .start
            .as_ref()
            .ok_or_else(|| syn::Error::new(self.range.span(), "range must have start"))?;
        let end_expr = self
            .range
            .end
            .as_ref()
            .ok_or_else(|| syn::Error::new(self.range.span(), "range must have end"))?;

        let range_start = expr_to_usize(start_expr)?;
        let range_end = expr_to_usize(end_expr)?;

        let ty = self.ty;
        let allocator = self
            .allocator
            .unwrap_or_else(|| syn::parse_quote! { Vec<T> });

        let ty_stream = ty.to_token_stream();
        let allocator_stream = allocator.to_token_stream();
        let allocator_with_ty_stream = replace_ident_with_stream(allocator_stream, "T", ty_stream);

        let default_expr = match self.default_expr {
            Some(expr) => expr,
            None => syn::parse2(quote! {
                <#allocator_with_ty_stream as ::core::default::Default>::default()
            })?,
        };

        let mut gen_flags = GenFlags::default();
        if self.skip.contains(&SkipKind::All) {
            gen_flags.new = false;
            gen_flags.internal_getters = false;
        } else {
            if self.skip.contains(&SkipKind::New) {
                gen_flags.new = false;
            }
            if self.skip.contains(&SkipKind::InternalGetters) {
                gen_flags.internal_getters = false;
            }

            if self.skip.contains(&SkipKind::TotalSizeFn) {
                gen_flags.total_size_fn = false;
            }
        }

        Ok(ArgsFinalized {
            ty,
            range_start,
            range_end,
            default_expr,
            allocator,
            gen_flags,
        })
    }
}

fn expr_to_usize(expr: &Expr) -> syn::Result<usize> {
    match expr {
        Expr::Lit(e) => match &e.lit {
            syn::Lit::Int(l) => l.base10_parse(),
            _ => Err(syn::Error::new(expr.span(), "expected int")),
        },
        _ => Err(syn::Error::new(expr.span(), "expected int")),
    }
}

fn replace_ident_with_stream(
    stream: TokenStream,
    target: &str,
    replacement: TokenStream,
) -> TokenStream {
    use proc_macro2::{Group, TokenTree};

    let mut out = TokenStream::new();

    for tt in stream {
        match tt {
            TokenTree::Ident(ref ident) if ident == target => {
                out.extend(replacement.clone());
            }
            TokenTree::Group(group) => {
                let mut new_group = Group::new(
                    group.delimiter(),
                    replace_ident_with_stream(group.stream(), target, replacement.clone()),
                );
                new_group.set_span(group.span());
                out.extend([TokenTree::Group(new_group)]);
            }
            other => out.extend([other]),
        }
    }

    out
}

fn replace_idents(stream: TokenStream, n_val: i64, i_val: i64) -> TokenStream {
    use proc_macro2::{Group, Literal, TokenTree};

    stream
        .into_iter()
        .map(|tt| match tt {
            TokenTree::Ident(ref ident) if ident == "N" => {
                TokenTree::Literal(Literal::i64_unsuffixed(n_val))
            }
            TokenTree::Ident(ref ident) if ident == "I" => {
                TokenTree::Literal(Literal::i64_unsuffixed(i_val))
            }
            TokenTree::Group(group) => {
                let mut new_group = Group::new(
                    group.delimiter(),
                    replace_idents(group.stream(), n_val, i_val),
                );
                new_group.set_span(group.span());
                TokenTree::Group(new_group)
            }
            other => other,
        })
        .collect()
}

fn replace_idents_with_metavars(stream: TokenStream, n_var: &str, i_var: &str) -> TokenStream {
    use proc_macro2::{Group, Punct, Spacing, TokenTree};

    let mut out = TokenStream::new();

    for tt in stream {
        match tt {
            TokenTree::Ident(ref ident) if ident == "N" => {
                out.extend([
                    TokenTree::Punct(Punct::new('$', Spacing::Alone)),
                    TokenTree::Ident(Ident::new(n_var, Span::call_site())),
                ]);
            }
            TokenTree::Ident(ref ident) if ident == "I" => {
                out.extend([
                    TokenTree::Punct(Punct::new('$', Spacing::Alone)),
                    TokenTree::Ident(Ident::new(i_var, Span::call_site())),
                ]);
            }
            TokenTree::Group(group) => {
                let mut new_group = Group::new(
                    group.delimiter(),
                    replace_idents_with_metavars(group.stream(), n_var, i_var),
                );
                new_group.set_span(group.span());
                out.extend([TokenTree::Group(new_group)]);
            }
            other => out.extend([other]),
        }
    }

    out
}

fn expand_ct_map(args: ArgsFinalized, input: ItemStruct) -> TokenStream {
    let struct_name = &input.ident;
    let generics = &input.generics;

    let vis = &input.vis;
    let attrs = &input.attrs; // Captures #[derive(...)] and other attributes

    let min = args.range_start;
    let max = args.range_end;
    let count = max - min;

    let ty_stream = args.ty.to_token_stream();
    let default_stream = args.default_expr.to_token_stream();
    let allocator_stream = args.allocator.to_token_stream();

    let fields = (min..max).enumerate().map(|(i, n)| {
        let specialized_type = replace_idents(ty_stream.clone(), n as i64, i as i64);
        let allocator_with_specialized =
            replace_ident_with_stream(allocator_stream.clone(), "T", specialized_type);
        quote! { #allocator_with_specialized }
    });

    let new_fn = if args.gen_flags.new {
        let defaults = (min..max).enumerate().map(|(i, n)| {
            let expr = replace_idents(default_stream.clone(), n as i64, i as i64);
            quote! { #expr }
        });

        quote! {
            pub fn new() -> Self {
                Self {
                    buckets: (#(#defaults),*)
                }
            }
        }
    } else {
        quote! {}
    };

    let total_size_fn = if args.gen_flags.total_size_fn {
        let mut rv = vec![quote! {0}];
        for (i, _) in (min..max).enumerate() {
            let i_idx = syn::Index::from(i); // Required for tuple indexing like .0, .1
            rv.push(quote! { + self.buckets.#i_idx.len() })
        }

        quote! {
            pub fn tot_size(&self) -> usize {
                #(#rv)*
            }
        }
    } else {
        quote! {}
    };

    let internal_getters = if args.gen_flags.internal_getters {
        let mut getters = Vec::new();
        for (i, n) in (min..max).enumerate() {
            let specialized_ty = replace_idents(ty_stream.clone(), n as i64, i as i64);
            let full_ty = replace_ident_with_stream(allocator_stream.clone(), "T", specialized_ty);
            let immutable_function_name = format_ident!("get_bucket_{}", n);
            let mutable_function_name = format_ident!("get_bucket_{}_mut", n);
            let take_function_name = format_ident!("take_bucket_{}", n);

            let i_idx = syn::Index::from(i); // Required for tuple indexing like .0, .1

            getters.push(quote! {
                #[inline(always)]
                pub fn #immutable_function_name(&self) -> &#full_ty {
                    &self.buckets.#i_idx
                }

                #[inline(always)]
                pub fn #mutable_function_name(&mut self) -> &mut #full_ty {
                    &mut self.buckets.#i_idx
                }

                #[inline(always)]
                pub fn #take_function_name(&mut self) -> #full_ty {
                    std::mem::take(&mut self.buckets.#i_idx)
                }
            });
        }
        quote! { #(#getters)* }
    } else {
        quote! {}
    };

    let helper_macro_name = format_ident!("__ct_map_for_{}", struct_name);

    let helper_expansions = (min..max).enumerate().map(|(i, n)| {
        let n_lit = syn::LitInt::new(&n.to_string(), Span::call_site());
        let i_lit = syn::LitInt::new(&i.to_string(), Span::call_site());
        quote! { $m!(#n_lit, #i_lit); }
    });

    quote! {
        #(#attrs)* // Re-emits the derives and other attributes here
        #vis struct #struct_name #generics{
            pub buckets: (#(#fields),*)
        }

        impl #generics #struct_name #generics{
            pub const START: usize = #min;
            pub const END: usize = #max;
            pub const SIZE: usize = #count;
            #new_fn
            #total_size_fn
            #internal_getters
        }

        #[doc(hidden)]
        #[macro_export]
        macro_rules! #helper_macro_name {
            ($m:ident) => {
                #(#helper_expansions)*
            };
        }
    }
}

pub fn ct_map(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let args = parse_macro_input!(attr as Args);
    let finalized = match args.finalize() {
        Ok(v) => v,
        Err(e) => return e.into_compile_error().into(),
    };
    let input_struct = parse_macro_input!(item as ItemStruct);
    expand_ct_map(finalized, input_struct).into()
}

struct AccessorArgs {
    target: Path,
}

impl Parse for AccessorArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut target = None;

        while !input.is_empty() {
            let ident: Ident = input.parse()?;
            let content;
            parenthesized!(content in input);

            match ident.to_string().as_str() {
                "target" => target = Some(content.parse()?),
                _ => {
                    return Err(syn::Error::new(
                        ident.span(),
                        "unknown ct_map_accessor option",
                    ));
                }
            }

            if input.peek(syn::Token![,]) {
                input.parse::<syn::Token![,]>()?;
            }
        }

        Ok(Self {
            target: target
                .ok_or_else(|| syn::Error::new(Span::call_site(), "missing target(...)"))?,
        })
    }
}

fn extract_accessor_attr(attrs: &mut Vec<syn::Attribute>) -> syn::Result<TokenStream> {
    let mut found = None;

    attrs.retain(|attr| {
        if attr.path().is_ident("accessor") {
            found = Some(attr.clone());
            false
        } else {
            true
        }
    });

    match found {
        Some(attr) => attr.parse_args(),
        None => Err(syn::Error::new(
            Span::call_site(),
            "missing #[accessor(...)] on method",
        )),
    }
}

pub fn ct_map_accessor(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let args = parse_macro_input!(attr as AccessorArgs);
    let mut input_trait = parse_macro_input!(item as ItemTrait);

    let trait_ident = input_trait.ident.clone();
    let target = args.target;

    let target_ident = target
        .segments
        .last()
        .map(|seg| seg.ident.clone())
        .unwrap_or_else(|| Ident::new("Target", Span::call_site()));

    let helper_macro_name = format_ident!("__ct_map_for_{}", target_ident);

    let mut method_tokens = Vec::new();

    for item in &mut input_trait.items {
        if let TraitItem::Fn(TraitItemFn { attrs, sig, .. }) = item {
            let body_tokens = match extract_accessor_attr(attrs) {
                Ok(tokens) => tokens,
                Err(err) => return err.into_compile_error().into(),
            };

            let sig_tokens = replace_idents_with_metavars(sig.to_token_stream(), "n", "i");
            let body_tokens = replace_idents_with_metavars(body_tokens, "n", "i");

            method_tokens.push(quote! {
                #(#attrs)*
                #sig_tokens {
                    #body_tokens
                }
            });
        }
    }

    let impl_macro = quote! {
        macro_rules! __ct_map_accessor_impls {
            ($n:literal, $i:tt) => {
                impl #trait_ident<$n> for #target {
                    #(#method_tokens)*
                }
            };
        }
    };

    let helper_call = quote! {
        #helper_macro_name!(__ct_map_accessor_impls);
    };

    quote! {
        #input_trait
        #impl_macro
        #helper_call
    }
    .into()
}
