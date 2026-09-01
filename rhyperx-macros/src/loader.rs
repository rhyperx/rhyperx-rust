use proc_macro::TokenStream;
use proc_macro2::{Group, TokenStream as TokenStream2, TokenTree};
use quote::{ToTokens, format_ident, quote}; // Corrected duplicates
use std::collections::{HashMap, HashSet};
use syn::parse::{Parse, ParseStream};
use syn::{Field, Ident, Item, ItemMod, Visibility, parse_macro_input};

// Helper struct to keep track of parsed loader metadata
struct StructInfo {
    ident: Ident,
    vis: Visibility,
    attrs: Vec<syn::Attribute>,
    impl_attrs: Vec<syn::Attribute>,
    parent: Option<Ident>,
    custom_method_name: Option<Ident>, // Stores the custom method name if provided
    local_fields: Vec<Field>,
    children: Vec<Ident>,
}

// Custom parser to handle `#[loaders::sub(Parent)]` or `#[loaders::sub(Parent, custom_name)]`
struct SubArgs {
    parent: Ident,
    custom_name: Option<Ident>,
}

impl Parse for SubArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let parent: Ident = input.parse()?;
        let mut custom_name = None;
        if input.peek(syn::Token![,]) {
            let _: syn::Token![,] = input.parse()?;
            custom_name = Some(input.parse()?);
        }
        Ok(SubArgs {
            parent,
            custom_name,
        })
    }
}

// Struct to store parsed dynamic leaf functions
struct LeafFunctionDef {
    target_root: Option<Ident>,
    func_tokens: TokenStream2,
}

// Recursively replaces occurrences of `struct_ident` with the target struct name
fn replace_struct_ident(stream: TokenStream2, target: &Ident) -> TokenStream2 {
    let mut out = TokenStream2::new();
    for tt in stream.into_iter() {
        match tt {
            TokenTree::Ident(ref id) if id == "struct_ident" => {
                let mut new_id = target.clone();
                new_id.set_span(id.span());
                out.extend(std::iter::once(TokenTree::Ident(new_id)));
            }
            TokenTree::Group(g) => {
                let new_stream = replace_struct_ident(g.stream(), target);
                let mut new_group = Group::new(g.delimiter(), new_stream);
                new_group.set_span(g.span());
                out.extend(std::iter::once(TokenTree::Group(new_group)));
            }
            _ => {
                out.extend(std::iter::once(tt));
            }
        }
    }
    out
}

// Recursive helper to check if a struct is in the subtree of a target root
fn is_descendant(child: &Ident, target_root: &Ident, structs: &HashMap<Ident, StructInfo>) -> bool {
    if child == target_root {
        return true;
    }
    if let Some(info) = structs.get(child)
        && let Some(ref parent) = info.parent
    {
        return is_descendant(parent, target_root, structs);
    }
    false
}

fn to_snake_case(s: &str) -> String {
    let mut snake = String::new();
    for (i, ch) in s.chars().enumerate() {
        if ch.is_uppercase() {
            if i > 0 {
                snake.push('_');
            }
            snake.push(ch.to_ascii_lowercase());
        } else {
            snake.push(ch);
        }
    }
    snake.replace("_d_b", "_db")
}

// Helper to check for and strip `#[loaders::for_each]` from any AST item type
fn strip_and_check_for_each(item: &mut Item) -> bool {
    let attrs = match item {
        Item::Const(i) => &mut i.attrs,
        Item::Enum(i) => &mut i.attrs,
        Item::ExternCrate(i) => &mut i.attrs,
        Item::Fn(i) => &mut i.attrs,
        Item::ForeignMod(i) => &mut i.attrs,
        Item::Impl(i) => &mut i.attrs,
        Item::Macro(i) => &mut i.attrs,
        Item::Mod(i) => &mut i.attrs,
        Item::Static(i) => &mut i.attrs,
        Item::Struct(i) => &mut i.attrs,
        Item::Trait(i) => &mut i.attrs,
        Item::TraitAlias(i) => &mut i.attrs,
        Item::Type(i) => &mut i.attrs,
        Item::Union(i) => &mut i.attrs,
        Item::Use(i) => &mut i.attrs,
        _ => return false,
    };

    let mut found = false;
    attrs.retain(|attr| {
        let path = attr.path();
        let is_for_each = path.is_ident("for_each")
            || (path.segments.len() == 2
                && path.segments[0].ident == "loaders"
                && path.segments[1].ident == "for_each");
        if is_for_each {
            found = true;
            false // Remove attribute after finding
        } else {
            true // Keep attribute
        }
    });
    found
}

pub fn loaders_suite(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut item_mod = parse_macro_input!(item as ItemMod);

    // 1. Process module-level attributes for global injection
    let mut global_struct_attrs: Vec<syn::Attribute> = Vec::new();
    let mut global_impl_attrs: Vec<syn::Attribute> = Vec::new();
    let mut retained_mod_attrs = Vec::new();

    for attr in item_mod.attrs {
        let path = attr.path();
        let is_struct_attr = path.is_ident("struct_attr")
            || (path.segments.len() == 2
                && path.segments[0].ident == "loaders"
                && path.segments[1].ident == "struct_attr");
        let is_impl_attr = path.is_ident("impl_attr")
            || (path.segments.len() == 2
                && path.segments[0].ident == "loaders"
                && path.segments[1].ident == "impl_attr");

        if is_struct_attr {
            let parser = syn::punctuated::Punctuated::<syn::Meta, syn::Token![,]>::parse_terminated;
            if let Ok(metas) = attr.parse_args_with(parser) {
                for meta in metas {
                    global_struct_attrs.push(syn::parse_quote! { #[#meta] });
                }
            }
        } else if is_impl_attr {
            let parser = syn::punctuated::Punctuated::<syn::Meta, syn::Token![,]>::parse_terminated;
            if let Ok(metas) = attr.parse_args_with(parser) {
                for meta in metas {
                    global_impl_attrs.push(syn::parse_quote! { #[#meta] });
                }
            }
        } else {
            retained_mod_attrs.push(attr);
        }
    }
    item_mod.attrs = retained_mod_attrs;

    let Some((brace, items)) = item_mod.content.take() else {
        return item_mod.into_token_stream().into();
    };

    // Pre-pass: Identify all managed loader structs to properly capture their explicit impl blocks
    let mut loader_idents = HashSet::new();
    for item in &items {
        if let Item::Struct(strct) = item {
            let is_loader = strct.attrs.iter().any(|attr| {
                let path = attr.path();
                path.is_ident("primary")
                    || path.is_ident("sub")
                    || (path.segments.len() == 2
                        && path.segments[0].ident == "loaders"
                        && (path.segments[1].ident == "primary" || path.segments[1].ident == "sub"))
            });
            if is_loader {
                loader_idents.insert(strct.ident.clone());
            }
        }
    }

    let mut managed_structs = HashMap::new();
    let mut explicit_impls = HashMap::new();
    let mut leaf_functions = Vec::new();
    let mut for_each_items = Vec::new();
    let mut other_items = Vec::new();

    // 2. Main separation pass: Collect structures, explicit impls, leaf functions, for_each items, and other items
    for mut item in items {
        // Intercept `#[loaders::for_each]` across any matching node type (macros, fns, impls, etc)
        if strip_and_check_for_each(&mut item) {
            for_each_items.push(item.into_token_stream());
            continue;
        }

        match item {
            Item::Struct(mut strct) if loader_idents.contains(&strct.ident) => {
                let mut parent = None;
                let mut custom_method_name = None;
                let mut retained_attrs = Vec::new();
                let mut impl_attrs = Vec::new();

                for attr in strct.attrs.iter() {
                    let path = attr.path();

                    let is_primary = path.is_ident("primary")
                        || (path.segments.len() == 2
                            && path.segments[0].ident == "loaders"
                            && path.segments[1].ident == "primary");

                    let is_sub = path.is_ident("sub")
                        || (path.segments.len() == 2
                            && path.segments[0].ident == "loaders"
                            && path.segments[1].ident == "sub");

                    if is_primary {
                        // Handled
                    } else if is_sub {
                        if let Ok(args) = attr.parse_args::<SubArgs>() {
                            parent = Some(args.parent);
                            custom_method_name = args.custom_name;
                        }
                    } else if path.is_ident("impl_attr") {
                        if let syn::Meta::List(meta_list) = &attr.meta {
                            let inner_tokens = &meta_list.tokens;
                            let forwarded_attr: syn::Attribute =
                                syn::parse_quote! { #[#inner_tokens] };
                            impl_attrs.push(forwarded_attr);
                        }
                    } else {
                        retained_attrs.push(attr.clone());
                    }
                }

                strct.attrs = retained_attrs;
                let local_fields = match strct.fields {
                    syn::Fields::Named(fields) => fields.named.into_iter().collect(),
                    _ => Vec::new(),
                };

                managed_structs.insert(
                    strct.ident.clone(),
                    StructInfo {
                        ident: strct.ident,
                        vis: strct.vis,
                        attrs: strct.attrs,
                        impl_attrs,
                        parent,
                        custom_method_name,
                        local_fields,
                        children: Vec::new(),
                    },
                );
            }
            Item::Fn(mut func) => {
                let mut is_leaf_func = false;
                let mut target_root = None;
                let mut retained_attrs = Vec::new();

                for attr in func.attrs.iter().cloned() {
                    let path = attr.path();
                    let is_leaf_attr = path.is_ident("leaf_function")
                        || (path.segments.len() == 2
                            && path.segments[0].ident == "loaders"
                            && path.segments[1].ident == "leaf_function");

                    if is_leaf_attr {
                        is_leaf_func = true;
                        if let syn::Meta::List(list) = &attr.meta
                            && let Ok(ident) = syn::parse2::<Ident>(list.tokens.clone())
                        {
                            target_root = Some(ident);
                        }
                    } else {
                        retained_attrs.push(attr);
                    }
                }

                if is_leaf_func {
                    func.attrs = retained_attrs;
                    leaf_functions.push(LeafFunctionDef {
                        target_root,
                        func_tokens: func.into_token_stream(),
                    });
                } else {
                    other_items.push(Item::Fn(func));
                }
            }
            Item::Impl(imp) if imp.trait_.is_none() => {
                let mut is_managed_impl = false;
                if let syn::Type::Path(type_path) = &*imp.self_ty
                    && type_path.qself.is_none()
                    && type_path.path.leading_colon.is_none()
                    && type_path.path.segments.len() == 1
                {
                    let struct_ident = &type_path.path.segments[0].ident;
                    if loader_idents.contains(struct_ident) {
                        is_managed_impl = true;
                        explicit_impls
                            .entry(struct_ident.clone())
                            .or_insert_with(Vec::new)
                            .extend(imp.items.clone());
                    }
                }
                if !is_managed_impl {
                    other_items.push(Item::Impl(imp));
                }
            }
            _ => {
                other_items.push(item);
            }
        }
    }

    // 3. Map out parent-to-child relationships
    let keys: Vec<Ident> = managed_structs.keys().cloned().collect();
    for key in &keys {
        if let Some(parent_ident) = managed_structs.get(key).unwrap().parent.clone()
            && let Some(parent_info) = managed_structs.get_mut(&parent_ident)
        {
            parent_info.children.push(key.clone());
        }
    }

    // Recursive helper to resolve field inheritance across the hierarchy
    fn resolve_all_fields(
        ident: &Ident,
        structs: &HashMap<Ident, StructInfo>,
        cache: &mut HashMap<Ident, Vec<Field>>,
    ) -> Vec<Field> {
        if let Some(fields) = cache.get(ident) {
            return fields.clone();
        }

        let info = &structs[ident];
        let mut all_fields = Vec::new();

        if let Some(ref parent_ident) = info.parent
            && structs.contains_key(parent_ident)
        {
            let parent_fields = resolve_all_fields(parent_ident, structs, cache);
            all_fields.extend(parent_fields);
        }

        for local_f in &info.local_fields {
            let local_ident = local_f.ident.as_ref();
            if let Some(existing_idx) = all_fields
                .iter()
                .position(|f| f.ident.as_ref() == local_ident)
            {
                all_fields[existing_idx] = local_f.clone();
            } else {
                all_fields.push(local_f.clone());
            }
        }

        cache.insert(ident.clone(), all_fields.clone());
        all_fields
    }

    let mut fields_cache = HashMap::new();
    let struct_keys: Vec<Ident> = managed_structs.keys().cloned().collect();
    for key in &struct_keys {
        resolve_all_fields(key, &managed_structs, &mut fields_cache);
    }

    let mut generated_items = Vec::<TokenStream2>::new();

    // 4. Code Generation Phase
    for info in managed_structs.values() {
        let struct_ident = &info.ident;
        let struct_vis = &info.vis;
        let struct_attrs = &info.attrs;
        let impl_attrs = &info.impl_attrs;

        let all_fields = fields_cache.get(struct_ident).unwrap();

        let explicit_method_names: HashSet<Ident> = explicit_impls
            .get(struct_ident)
            .map(|items| {
                items
                    .iter()
                    .filter_map(|item| {
                        if let syn::ImplItem::Fn(f) = item {
                            Some(f.sig.ident.clone())
                        } else {
                            None
                        }
                    })
                    .collect()
            })
            .unwrap_or_default();

        // Generate Struct Definition
        let fields_tokens = all_fields.iter().map(|f| {
            let f_vis = &f.vis;
            let f_ident = &f.ident;
            let f_ty = &f.ty;

            let f_attrs = f.attrs.iter().filter(|attr| {
                let path = attr.path();
                let is_builder_attr = path.is_ident("builder")
                    || path.segments.iter().any(|seg| seg.ident == "builder");
                !is_builder_attr
            });

            quote! { #(#f_attrs)* #f_vis #f_ident: #f_ty }
        });

        let global_struct_attrs_ref = &global_struct_attrs;
        generated_items.push(quote! {
            #(#global_struct_attrs_ref)*
            #(#struct_attrs)*
            #struct_vis struct #struct_ident {
                #(#fields_tokens,)*
            }
        });

        let mut final_methods = Vec::<TokenStream2>::new();

        // 4b. Field setters generation
        for f in all_fields {
            let f_ident = &f.ident;
            let f_ty = &f.ty;

            let should_skip = f.attrs.iter().any(|attr| {
                let segments = &attr.path().segments;
                let is_builder = (segments.len() == 1 && segments[0].ident == "builder")
                    || (segments.len() == 2
                        && segments[0].ident == "loaders"
                        && segments[1].ident == "builder");

                if is_builder {
                    if let syn::Meta::List(meta_list) = &attr.meta {
                        meta_list.tokens.to_string() == "skip"
                    } else {
                        false
                    }
                } else {
                    false
                }
            });

            if !should_skip
                && let Some(f_id) = f_ident.clone()
                && !explicit_method_names.contains(&f_id)
            {
                final_methods.push(quote! {
                    pub fn #f_id(&mut self, #f_id: #f_ty) -> Self {
                        self.#f_id = #f_id;
                        self.clone()
                    }
                });
            }
        }

        // 4c. Context Transitions / End Node Action generation
        if !info.children.is_empty() {
            for child_ident in &info.children {
                let child_info = &managed_structs[child_ident];
                let child_all_fields = fields_cache.get(child_ident).unwrap();

                let method_name = if let Some(ref custom_override) = child_info.custom_method_name {
                    custom_override.clone()
                } else {
                    format_ident!("{}", to_snake_case(&child_ident.to_string()))
                };

                if !explicit_method_names.contains(&method_name) {
                    let copy_fields = child_all_fields.iter().filter_map(|f| {
                        let f_ident = &f.ident;
                        let is_redefined = child_info
                            .local_fields
                            .iter()
                            .any(|lf| lf.ident == *f_ident);
                        let parent_has_field = all_fields.iter().any(|pf| pf.ident == *f_ident);

                        if parent_has_field && !is_redefined {
                            Some(quote! { rv.#f_ident = self.#f_ident.clone(); })
                        } else {
                            None
                        }
                    });

                    final_methods.push(quote! {
                        pub fn #method_name(&self) -> #child_ident {
                            let mut rv = #child_ident::default();
                            #(#copy_fields)*
                            rv
                        }
                    });
                }
            }
        } else {
            // Generate custom leaf functions dynamically
            for leaf_func in &leaf_functions {
                let applies = match &leaf_func.target_root {
                    Some(root) => is_descendant(struct_ident, root, &managed_structs),
                    None => true,
                };

                if applies {
                    let replaced_tokens =
                        replace_struct_ident(leaf_func.func_tokens.clone(), struct_ident);
                    if let Ok(replaced_fn) = syn::parse2::<syn::ItemFn>(replaced_tokens.clone()) {
                        let fn_ident = &replaced_fn.sig.ident;
                        if !explicit_method_names.contains(fn_ident) {
                            final_methods.push(replaced_tokens);
                        }
                    }
                }
            }
        }

        let user_items = explicit_impls.remove(struct_ident).unwrap_or_default();
        let global_impl_attrs_ref = &global_impl_attrs;

        generated_items.push(quote! {
            #(#global_impl_attrs_ref)*
            #(#impl_attrs)*
            impl #struct_ident {
                #(#user_items)*
                #(#final_methods)*
            }
        });

        // 4d. Trait and Getter Generation (LoaderNameAttr)
        let trait_ident = format_ident!("{}Attr", struct_ident);

        // 1. Define the Trait
        let trait_methods = info.local_fields.iter().map(|f| {
            let f_ident = format_ident!("get_{}", &f.ident.as_ref().unwrap());
            let f_ty = &f.ty;
            quote! { fn #f_ident(&self) -> &#f_ty; }
        });

        // Push the trait definition first
        generated_items.push(quote! {
            pub trait #trait_ident {
                #(#trait_methods)*
            }
        });

        // 2. Implement this trait for the current struct AND all of its descendants
        for desc_ident in managed_structs.keys() {
            if is_descendant(desc_ident, struct_ident, &managed_structs) {
                // We need the fields for the descendant to implement the trait
                // let desc_all_fields = fields_cache.get(desc_ident).unwrap();

                let impl_methods = info.local_fields.iter().map(|f| {
                    let f_ident = &f.ident.as_ref().unwrap();
                    let getter_ident = format_ident!("get_{}", f_ident);
                    let f_ty = &f.ty;

                    // Ensure the field exists on the descendant
                    quote! {
                        fn #getter_ident(&self) -> &#f_ty {
                            &self.#f_ident
                        }
                    }
                });

                generated_items.push(quote! {
                    impl #trait_ident for #desc_ident {
                        #(#impl_methods)*
                    }
                });
            }
        }
    }

    // 5. Expand `#[loaders::for_each]` items
    let mut expanded_for_each_tokens = Vec::new();
    for target_ident in managed_structs.keys() {
        for item_tokens in &for_each_items {
            let duplicated_item = replace_struct_ident(item_tokens.clone(), target_ident);
            expanded_for_each_tokens.push(duplicated_item);
        }
    }

    let final_tokens = quote! {
        #(#other_items)*
        #(#generated_items)*
        #(#expanded_for_each_tokens)*
    };

    item_mod.content = Some((brace, vec![Item::Verbatim(final_tokens)]));
    item_mod.into_token_stream().into()
}
