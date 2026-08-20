use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use regex::Regex;
use syn::{
    Attribute, Expr, ExprLit, Fields, Item, Lit,
    parse::{Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
    token::Comma,
};

// ---------------------------------------------------------------------------
// Argument parsing
// ---------------------------------------------------------------------------

struct RemoveAttrArgs {
    patterns: Vec<String>,
}

impl Parse for RemoveAttrArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let exprs = Punctuated::<Expr, Comma>::parse_terminated(input)?;
        let mut patterns = Vec::new();

        for expr in exprs {
            match &expr {
                Expr::Lit(ExprLit {
                    lit: Lit::Str(s), ..
                }) => {
                    patterns.push(s.value());
                }
                _ => {
                    return Err(syn::Error::new_spanned(
                        expr,
                        "expected a string literal regex pattern",
                    ));
                }
            }
        }

        if patterns.is_empty() {
            return Err(input.error("expected at least one regex pattern"));
        }

        Ok(RemoveAttrArgs { patterns })
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Returns true if the attribute's path matches any of the compiled regexes.
fn attr_matches(attr: &Attribute, regexes: &[Regex]) -> bool {
    let path_str = attr
        .path()
        .segments
        .iter()
        .map(|s| s.ident.to_string())
        .collect::<Vec<_>>()
        .join("::");

    regexes.iter().any(|re| re.is_match(&path_str))
}

/// Filter a `Vec<Attribute>`, removing those that match any regex.
fn filter_attrs(attrs: Vec<Attribute>, regexes: &[Regex]) -> Vec<Attribute> {
    attrs
        .into_iter()
        .filter(|a| !attr_matches(a, regexes))
        .collect()
}

/// Recursively strip matching attributes from every field / variant / item
/// inside the top-level item, then return the rewritten `TokenStream2`.
fn strip_item(mut item: Item, regexes: &[Regex]) -> TokenStream2 {
    match &mut item {
        // ------------------------------------------------------------------ struct
        Item::Struct(s) => {
            s.attrs = filter_attrs(std::mem::take(&mut s.attrs), regexes);
            strip_fields(&mut s.fields, regexes);
            quote!(#s)
        }
        // ------------------------------------------------------------------ enum
        Item::Enum(e) => {
            e.attrs = filter_attrs(std::mem::take(&mut e.attrs), regexes);
            for variant in &mut e.variants {
                variant.attrs = filter_attrs(std::mem::take(&mut variant.attrs), regexes);
                strip_fields(&mut variant.fields, regexes);
            }
            quote!(#e)
        }
        // ------------------------------------------------------------------ union
        Item::Union(u) => {
            u.attrs = filter_attrs(std::mem::take(&mut u.attrs), regexes);
            for field in u.fields.named.iter_mut() {
                field.attrs = filter_attrs(std::mem::take(&mut field.attrs), regexes);
            }
            quote!(#u)
        }
        // ------------------------------------------------------------------ fn
        Item::Fn(f) => {
            f.attrs = filter_attrs(std::mem::take(&mut f.attrs), regexes);
            quote!(#f)
        }
        // ------------------------------------------------------------------ impl
        Item::Impl(i) => {
            i.attrs = filter_attrs(std::mem::take(&mut i.attrs), regexes);
            for impl_item in &mut i.items {
                strip_impl_item(impl_item, regexes);
            }
            quote!(#i)
        }
        // ------------------------------------------------------------------ trait
        Item::Trait(t) => {
            t.attrs = filter_attrs(std::mem::take(&mut t.attrs), regexes);
            for trait_item in &mut t.items {
                strip_trait_item(trait_item, regexes);
            }
            quote!(#t)
        }
        // ------------------------------------------------------------------ type alias
        Item::Type(t) => {
            t.attrs = filter_attrs(std::mem::take(&mut t.attrs), regexes);
            quote!(#t)
        }
        // ------------------------------------------------------------------ const / static
        Item::Const(c) => {
            c.attrs = filter_attrs(std::mem::take(&mut c.attrs), regexes);
            quote!(#c)
        }
        Item::Static(s) => {
            s.attrs = filter_attrs(std::mem::take(&mut s.attrs), regexes);
            quote!(#s)
        }
        // ------------------------------------------------------------------ mod
        Item::Mod(m) => {
            m.attrs = filter_attrs(std::mem::take(&mut m.attrs), regexes);
            if let Some((brace, items)) = &mut m.content {
                let new_items: Vec<_> = std::mem::take(items)
                    .into_iter()
                    .map(|i| {
                        // parse back from token stream so we get an Item again
                        let ts = strip_item(i, regexes);
                        syn::parse2(ts).unwrap_or_else(|_| {
                            // fallback: return whatever we had (already rewritten)
                            syn::parse2(quote!()).unwrap()
                        })
                    })
                    .collect();
                *items = new_items;
                let _ = brace; // suppress unused warning
            }
            quote!(#m)
        }
        // ------------------------------------------------------------------ use / extern / macro / verbatim – pass through
        other => quote!(#other),
    }
}

fn strip_fields(fields: &mut Fields, regexes: &[Regex]) {
    match fields {
        Fields::Named(f) => {
            for field in f.named.iter_mut() {
                field.attrs = filter_attrs(std::mem::take(&mut field.attrs), regexes);
            }
        }
        Fields::Unnamed(f) => {
            for field in f.unnamed.iter_mut() {
                field.attrs = filter_attrs(std::mem::take(&mut field.attrs), regexes);
            }
        }
        Fields::Unit => {}
    }
}

fn strip_impl_item(item: &mut syn::ImplItem, regexes: &[Regex]) {
    match item {
        syn::ImplItem::Fn(f) => {
            f.attrs = filter_attrs(std::mem::take(&mut f.attrs), regexes);
        }
        syn::ImplItem::Const(c) => {
            c.attrs = filter_attrs(std::mem::take(&mut c.attrs), regexes);
        }
        syn::ImplItem::Type(t) => {
            t.attrs = filter_attrs(std::mem::take(&mut t.attrs), regexes);
        }
        _ => {}
    }
}

fn strip_trait_item(item: &mut syn::TraitItem, regexes: &[Regex]) {
    match item {
        syn::TraitItem::Fn(f) => {
            f.attrs = filter_attrs(std::mem::take(&mut f.attrs), regexes);
        }
        syn::TraitItem::Const(c) => {
            c.attrs = filter_attrs(std::mem::take(&mut c.attrs), regexes);
        }
        syn::TraitItem::Type(t) => {
            t.attrs = filter_attrs(std::mem::take(&mut t.attrs), regexes);
        }
        _ => {}
    }
}

// ---------------------------------------------------------------------------
// The macro entry-point
// ---------------------------------------------------------------------------

/// Remove helper attributes whose path matches any of the given regex patterns.
///
/// # Example
/// ```rust
/// #[remove_attr("serde.*", "some_other_attr")]
/// #[derive(Debug)]
/// struct Foo {
///     #[serde(rename = "bar")]
///     #[doc = "kept"]
///     field: u32,
/// }
/// // expands to:
/// #[derive(Debug)]
/// struct Foo {
///     #[doc = "kept"]
///     field: u32,
/// }
/// ```
pub fn remove_attr(args: TokenStream, input: TokenStream) -> TokenStream {
    // 1. Parse the regex pattern arguments.
    let RemoveAttrArgs { patterns } = parse_macro_input!(args as RemoveAttrArgs);

    // 2. Compile all regexes up-front (full-match anchored).
    let regexes: Vec<Regex> = patterns
        .iter()
        .map(|p| {
            // Anchor automatically so "serde.*" matches the whole path string.
            let anchored = format!("^(?:{})$", p);
            Regex::new(&anchored).unwrap_or_else(|e| panic!("invalid regex `{}`: {}", p, e))
        })
        .collect();

    // 3. Parse the annotated item.
    let item = parse_macro_input!(input as Item);

    // 4. Strip matching attributes recursively.
    strip_item(item, &regexes).into()
}
