use std::str::FromStr;

use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::quote;
use syn::{
    ext::IdentExt, punctuated::Punctuated, Block, Error, FnArg, ImplItem, ItemImpl, Pat, ReturnType, Token, Type,
    TypeReference,
};

use crate::{
    args::{self, ComplexityType, RenameRuleExt, RenameTarget},
    output_type::OutputType,
    utils::{
        extract_input_args, gen_deprecation, generate_default, generate_guards, get_cfg_attrs, get_crate_name,
        get_rustdoc, get_type_path_and_name, parse_complexity_expr, parse_graphql_attrs, remove_graphql_attrs,
        visible_fn, GeneratorResult,
    },
};

pub fn generate(object_args: &args::Object, item_impl: &mut ItemImpl) -> GeneratorResult<TokenStream> {
    let crate_name = get_crate_name(object_args.internal);
    let (self_ty, self_name) = get_type_path_and_name(item_impl.self_ty.as_ref())?;
    let (impl_generics, _, where_clause) = item_impl.generics.split_for_impl();
    let extends = object_args.extends;
    let gql_typename = object_args
        .name
        .clone()
        .unwrap_or_else(|| RenameTarget::Type.rename(self_name.clone()));

    let desc = if object_args.use_type_description {
        quote! { ::std::option::Option::Some(::std::borrow::ToOwned::to_owned(<Self as #crate_name::Description>::description())) }
    } else {
        get_rustdoc(&item_impl.attrs)
            .map(|s| quote!(::std::option::Option::Some(::std::borrow::ToOwned::to_owned(#s))))
            .unwrap_or_else(|| quote!(::std::option::Option::None))
    };

    let mut resolvers = Vec::new();
    let mut schema_fields = Vec::new();
    let mut find_entities = Vec::new();
    let mut add_keys = Vec::new();
    let mut create_entity_types = Vec::new();

    // Computation of the derivated fields
    let mut derived_impls = vec![];
    for item in &mut item_impl.items {
        if let ImplItem::Fn(method) = item {
            let method_args: args::ObjectField = parse_graphql_attrs(&method.attrs)?.unwrap_or_default();

            for derived in method_args.derived {
                if derived.name.is_some() && derived.into.is_some() {
                    let base_function_name = &method.sig.ident;
                    let name = derived.name.unwrap();
                    let with = derived.with;
                    let into = Type::Verbatim(proc_macro2::TokenStream::from_str(&derived.into.unwrap()).unwrap());

                    let mut new_impl = method.clone();
                    new_impl.sig.ident = name;
                    new_impl.sig.output = syn::parse2::<ReturnType>(quote! { -> #crate_name::Result<#into> })
                        .expect("invalid result type");

                    let should_create_context = new_impl
                        .sig
                        .inputs
                        .iter()
                        .nth(1)
                        .map(|x| {
                            if let FnArg::Typed(pat) = x {
                                if let Type::Reference(TypeReference { elem, .. }) = &*pat.ty {
                                    if let Type::Path(path) = elem.as_ref() {
                                        return path.path.segments.last().unwrap().ident != "Context";
                                    }
                                }
                            };
                            true
                        })
                        .unwrap_or(true);

                    if should_create_context {
                        let arg_ctx = syn::parse2::<FnArg>(quote! { ctx: &Context<'_> }).expect("invalid arg type");
                        new_impl.sig.inputs.insert(1, arg_ctx);
                    }

                    let other_atts: Punctuated<Ident, Token![,]> = new_impl
                        .sig
                        .inputs
                        .iter()
                        .filter_map(|x| match x {
                            FnArg::Typed(pat) => match &*pat.pat {
                                Pat::Ident(ident) => Some(Ok(ident.ident.clone())),
                                _ => Some(Err(Error::new_spanned(pat, "Must be a simple argument"))),
                            },
                            FnArg::Receiver(_) => None,
                        })
                        .collect::<Result<Vec<Ident>, Error>>()?
                        .into_iter()
                        .collect();

                    let new_block = match with {
                        Some(with) => quote!({
                            ::std::result::Result::Ok(#with(#self_ty::#base_function_name(&self, #other_atts).await?))
                        }),
                        None => quote!({
                            {
                                ::std::result::Result::Ok(#self_ty::#base_function_name(&self, #other_atts).await?.into())
                            }
                        }),
                    };

                    new_impl.block = syn::parse2::<Block>(new_block).expect("invalid block");

                    derived_impls.push(ImplItem::Fn(new_impl));
                }
            }
        }
    }
    item_impl.items.append(&mut derived_impls);

    for item in &mut item_impl.items {
        if let ImplItem::Fn(method) = item {
            let method_args: args::ObjectField = parse_graphql_attrs(&method.attrs)?.unwrap_or_default();

            if method_args.entity {
                let cfg_attrs = get_cfg_attrs(&method.attrs);

                if method.sig.asyncness.is_none() {
                    return Err(Error::new_spanned(&method, "Must be asynchronous").into());
                }

                let args = extract_input_args(&crate_name, method)?;

                let ty = match &method.sig.output {
                    ReturnType::Type(_, ty) => OutputType::parse(ty)?,
                    ReturnType::Default => {
                        return Err(Error::new_spanned(&method.sig.output, "Resolver must have a return type").into())
                    }
                };

                let entity_type = ty.value_type();
                let mut key_pat = Vec::new();
                let mut key_getter = Vec::new();
                let mut use_keys = Vec::new();
                let mut get_federation_key = Vec::new();
                let mut requires_getter = Vec::new();
                let all_key = args.iter().all(|(_, _, arg)| !arg.key);

                if args.is_empty() {
                    return Err(Error::new_spanned(method, "Entity need to have at least one key.").into());
                }

                for (ident, ty, args::Argument { name, key, .. }) in &args {
                    let is_key = all_key || *key;
                    let name = name.clone().unwrap_or_else(|| {
                        object_args
                            .rename_args
                            .rename(ident.ident.unraw().to_string(), RenameTarget::Argument)
                    });

                    if is_key {
                        get_federation_key.push(quote! {
                            if let Some(fields) = <#ty as #crate_name::LegacyInputType>::federation_fields() {
                                key_str.push(format!("{} {}", #name, fields));
                            } else {
                                key_str.push(#name.to_string());
                            }
                        });

                        key_pat.push(quote! {
                            ::std::option::Option::Some(#ident)
                        });
                        key_getter.push(quote! {
                            params.get(#name).and_then(|value| {
                                let value: ::std::option::Option<#ty> = #crate_name::LegacyInputType::parse(::std::option::Option::Some(::std::clone::Clone::clone(&value))).ok();
                                value
                            })
                        });
                    } else {
                        // requires
                        requires_getter.push(quote! {
                            let #ident: #ty = #crate_name::LegacyInputType::parse(params.get(#name).cloned()).
                                map_err(|err| err.into_server_error(ctx.item.pos))?;
                        });
                    }
                    use_keys.push(ident);
                }

                add_keys.push(quote! {
                    {
                        let mut key_str = Vec::new();
                        #(#get_federation_key)*
                        registry.add_keys(&<#entity_type as #crate_name::LegacyOutputType>::type_name(), &key_str.join(" "));
                    }
                });
                create_entity_types
                    .push(quote! { <#entity_type as #crate_name::LegacyOutputType>::create_type_info(registry); });

                let field_ident = &method.sig.ident;
                if let OutputType::Value(inner_ty) = &ty {
                    let block = &method.block;
                    let new_block = quote!({
                        {
                            let value:#inner_ty = async move #block.await;
                            ::std::result::Result::Ok(value)
                        }
                    });
                    method.block = syn::parse2::<Block>(new_block).expect("invalid block");
                    method.sig.output = syn::parse2::<ReturnType>(quote! { -> #crate_name::Result<#inner_ty> })
                        .expect("invalid result type");
                }
                let do_find = quote! {
                    self.#field_ident(ctx, #(#use_keys),*)
                        .await.map_err(|err| ::std::convert::Into::<#crate_name::Error>::into(err)
                        .into_server_error(ctx.item.pos))
                };

                find_entities.push((
                    args.len(),
                    quote! {
                        #(#cfg_attrs)*
                        if typename == &<#entity_type as #crate_name::LegacyOutputType>::type_name() {
                            if let (#(#key_pat),*) = (#(#key_getter),*) {
                                use #crate_name::context::ContextExt;
                                let f = async move {
                                    #(#requires_getter)*
                                    #do_find
                                };
                                let obj = f.await.map_err(|err| ctx.set_error_path(err))?;
                                let ctx_obj = ctx.with_selection_set(&ctx.item.node.selection_set);
                                return #crate_name::LegacyOutputType::resolve(&obj, &ctx_obj, ctx.item).await.map(::std::option::Option::Some);
                            }
                        }
                    },
                ));
            } else if !method_args.skip {
                if method.sig.asyncness.is_none() {
                    return Err(Error::new_spanned(&method, "Must be asynchronous").into());
                }
                let cfg_attrs = get_cfg_attrs(&method.attrs);

                if method_args.flatten {
                    // Only used to inject the context placeholder if required.
                    extract_input_args(&crate_name, method)?;

                    let ty = match &method.sig.output {
                        ReturnType::Type(_, ty) => OutputType::parse(ty)?,
                        ReturnType::Default => {
                            return Err(Error::new_spanned(
                                &method.sig.output,
                                "Flatten resolver must have a return type",
                            )
                            .into())
                        }
                    };
                    let ty = ty.value_type();
                    let ident = &method.sig.ident;

                    schema_fields.push(quote! {
                        #crate_name::static_assertions::assert_impl_one!(#ty: #crate_name::ObjectType);
                        <#ty>::create_type_info(registry);
                        if let #crate_name::registry::MetaType::Object { fields: obj_fields, .. } =
                            registry.create_fake_output_type::<#ty>() {
                            fields.extend(obj_fields);
                        }
                    });

                    resolvers.push(quote! {
                        #(#cfg_attrs)*
                        if let ::std::option::Option::Some(value) = #crate_name::ContainerType::resolve_field(&self.#ident(ctx).await, ctx).await? {
                            return ::std::result::Result::Ok(std::option::Option::Some(value));
                        }
                    });

                    remove_graphql_attrs(&mut method.attrs);
                    continue;
                }

                let field_name = method_args.name.clone().unwrap_or_else(|| {
                    object_args
                        .rename_fields
                        .rename(method.sig.ident.unraw().to_string(), RenameTarget::Field)
                });
                let field_desc = get_rustdoc(&method.attrs)
                    .map(|s| quote! { ::std::option::Option::Some(::std::borrow::ToOwned::to_owned(#s)) })
                    .unwrap_or_else(|| quote! {::std::option::Option::None});
                let field_deprecation = gen_deprecation(&method_args.deprecation, &crate_name);
                let external = method_args.external;
                let requires = match &method_args.requires {
                    Some(requires) => quote! { ::std::option::Option::Some(#requires) },
                    None => quote! { ::std::option::Option::None },
                };
                let provides = match &method_args.provides {
                    Some(provides) => quote! { ::std::option::Option::Some(#provides) },
                    None => quote! { ::std::option::Option::None },
                };
                let cache_control = {
                    let public = method_args.cache_control.is_public();
                    let max_age = method_args.cache_control.max_age;
                    let stale_while_revalidate = method_args.cache_control.stale_while_revalidate;
                    quote! {
                        #crate_name::CacheControl {
                            public: #public,
                            max_age: #max_age,
                            stale_while_revalidate: #stale_while_revalidate,
                            invalidation_policy: None,
                            access_scopes: None,
                        }
                    }
                };

                let args = extract_input_args(&crate_name, method)?;
                let mut schema_args = Vec::new();
                let mut use_params = Vec::new();
                let mut get_params = Vec::new();
                let ty = match &method.sig.output {
                    ReturnType::Type(_, ty) => OutputType::parse(ty)?,
                    ReturnType::Default => {
                        return Err(Error::new_spanned(&method.sig.output, "Resolver must have a return type").into())
                    }
                };

                for (
                    ident,
                    ty,
                    args::Argument {
                        name,
                        desc,
                        default,
                        default_with,
                        validator,
                        visible,
                        secret,
                        ..
                    },
                ) in &args
                {
                    let name = name.clone().unwrap_or_else(|| {
                        object_args
                            .rename_args
                            .rename(ident.ident.unraw().to_string(), RenameTarget::Argument)
                    });
                    let desc = desc
                        .as_ref()
                        .map(|s| quote! {::std::option::Option::Some(::std::borrow::ToOwned::to_owned(#s))})
                        .unwrap_or_else(|| quote! {::std::option::Option::None});
                    let default = generate_default(default, default_with)?;
                    let schema_default = default
                        .as_ref()
                        .map(|value| {
                            quote! {
                                ::std::option::Option::Some(<#ty as #crate_name::LegacyInputType>::to_value(&#value))
                            }
                        })
                        .unwrap_or_else(|| quote! {::std::option::Option::None});

                    let visible = visible_fn(visible);
                    schema_args.push(quote! {
                        args.insert(::std::borrow::ToOwned::to_owned(#name), #crate_name::registry::MetaInputValue {
                            name: ::std::borrow::ToOwned::to_owned(#name),
                            description: #desc,
                            ty: <#ty as #crate_name::LegacyInputType>::create_type_info(registry),
                            default_value: #schema_default,
                            validators: None,
                            visible: #visible,
                            is_secret: #secret,
                            rename: None
                        });
                    });

                    let param_ident = &ident.ident;
                    use_params.push(quote! { #param_ident });

                    let default = match default {
                        Some(default) => {
                            quote! { ::std::option::Option::Some(|| -> #ty { #default }) }
                        }
                        None => quote! { ::std::option::Option::None },
                    };

                    let validators = validator.clone().unwrap_or_default().create_validators(
                        &crate_name,
                        quote!(&#ident),
                        quote!(#ty),
                        Some(quote!(.map_err(|err| err.into_server_error(__pos)))),
                    )?;

                    get_params.push(quote! {
                        #[allow(non_snake_case, unused_variables)]
                        let (__pos, #ident) = ctx.param_value::<#ty>(#name, #default)?;
                        #validators
                    });
                }

                let schema_ty = ty.value_type();
                let visible = visible_fn(&method_args.visible);

                let complexity = if let Some(complexity) = &method_args.complexity {
                    match complexity {
                        ComplexityType::Const(n) => {
                            quote! { ::std::option::Option::Some(#crate_name::registry::ComplexityType::Const(#n)) }
                        }
                        ComplexityType::Fn(s) => {
                            let (variables, expr) = parse_complexity_expr(s)?;
                            let mut parse_args = Vec::new();
                            for variable in variables {
                                if let Some((
                                    ident,
                                    ty,
                                    args::Argument {
                                        name,
                                        default,
                                        default_with,
                                        ..
                                    },
                                )) = args.iter().find(|(pat_ident, _, _)| pat_ident.ident == variable)
                                {
                                    let default = match generate_default(default, default_with)? {
                                        Some(default) => {
                                            quote! { ::std::option::Option::Some(|| -> #ty { #default }) }
                                        }
                                        None => quote! { ::std::option::Option::None },
                                    };
                                    let name = name.clone().unwrap_or_else(|| {
                                        object_args
                                            .rename_args
                                            .rename(ident.ident.unraw().to_string(), RenameTarget::Argument)
                                    });
                                    parse_args.push(quote! {
                                        let #ident: #ty = __ctx.param_value(__variables_definition, __field, #name, #default)?;
                                    });
                                }
                            }
                            quote! {
                                Some(#crate_name::registry::ComplexityType::Fn(|__ctx, __variables_definition, __field, child_complexity| {
                                    #(#parse_args)*
                                    Ok(#expr)
                                }))
                            }
                        }
                    }
                } else {
                    quote! { ::std::option::Option::None }
                };

                schema_fields.push(quote! {
                    #(#cfg_attrs)*
                    fields.insert(::std::borrow::ToOwned::to_owned(#field_name), #crate_name::registry::MetaField {
                        name: ::std::borrow::ToOwned::to_owned(#field_name),
                        mapped_name: None,
                        description: #field_desc,
                        args: {
                            let mut args = #crate_name::indexmap::IndexMap::new();
                            #(#schema_args)*
                            args
                        },
                        ty: <#schema_ty as #crate_name::LegacyOutputType>::create_type_info(registry),
                        deprecation: #field_deprecation,
                        cache_control: #cache_control,
                        external: #external,
                        provides: #provides,
                        requires: #requires,
                        visible: #visible,
                        compute_complexity: #complexity,
                        edges: Vec::new(),
                        relation: None,
                        resolver: #crate_name::registry::resolvers::Resolver::Parent,
                        required_operation: None,
                        auth: None,
                    });
                });

                let field_ident = &method.sig.ident;
                if let OutputType::Value(inner_ty) = &ty {
                    let block = &method.block;
                    let new_block = quote!({
                        {
                            ::std::result::Result::Ok(async move {
                                let value:#inner_ty = #block;
                                value
                            }.await)
                        }
                    });
                    method.block = syn::parse2::<Block>(new_block).expect("invalid block");
                    method.sig.output = syn::parse2::<ReturnType>(quote! { -> #crate_name::Result<#inner_ty> })
                        .expect("invalid result type");
                }

                let resolve_obj = quote! {
                    {
                        let res = self.#field_ident(ctx, #(#use_params),*).await;
                        res.map_err(|err| ::std::convert::Into::<#crate_name::Error>::into(err).into_server_error(ctx.item.pos))
                    }
                };

                let guard_map_err = quote! {
                    .map_err(|err| err.into_server_error(ctx.item.pos))
                };
                let guard = match &method_args.guard {
                    Some(code) => Some(generate_guards(&crate_name, code, guard_map_err)?),
                    None => None,
                };

                resolvers.push(quote! {
                    #(#cfg_attrs)*
                    if ctx.item.node.name.node == #field_name {
                        use #crate_name::context::ContextExt;
                        let f = async move {
                            #(#get_params)*
                            #guard
                            #resolve_obj
                        };
                        let obj = f.await.map_err(|err| ctx.set_error_path(err))?;
                        let ctx_obj = ctx.with_selection_set(&ctx.item.node.selection_set);
                        return #crate_name::LegacyOutputType::resolve(&obj, &ctx_obj, ctx.item).await.map(::std::option::Option::Some);
                    }
                });
            }

            remove_graphql_attrs(&mut method.attrs);
        }
    }

    let cache_control = {
        let public = object_args.cache_control.is_public();
        let max_age = object_args.cache_control.max_age;
        let stale_while_revalidate = object_args.cache_control.stale_while_revalidate;
        quote! {
            #crate_name::CacheControl {
                public: #public,
                max_age: #max_age,
                stale_while_revalidate: #stale_while_revalidate,
                invalidation_policy: None,
                access_scopes: None,
            }
        }
    };

    find_entities.sort_by(|(a, _), (b, _)| b.cmp(a));
    let find_entities_iter = find_entities.iter().map(|(_, code)| code);

    if resolvers.is_empty() && create_entity_types.is_empty() {
        return Err(Error::new_spanned(self_ty, "A GraphQL Object type must define one or more fields.").into());
    }

    let visible = visible_fn(&object_args.visible);
    // TODO: Ok, so this thing is doing a recurse...
    let resolve_container = if object_args.serial {
        quote! { #crate_name::resolver_utils::resolve_container_serial_native(ctx, self).await }
    } else {
        quote! { #crate_name::resolver_utils::resolve_container_native(ctx, self).await }
    };

    let expanded = if object_args.concretes.is_empty() {
        quote! {
            #item_impl

            #[allow(clippy::all, clippy::pedantic, clippy::suspicious_else_formatting)]
            #[allow(unused_braces, unused_variables, unused_parens, unused_mut)]
            #[#crate_name::async_trait::async_trait]
            impl #impl_generics #crate_name::resolver_utils::ContainerType for #self_ty #where_clause {
                async fn resolve_field(&self, ctx: &#crate_name::Context<'_>) -> #crate_name::ServerResult<::std::option::Option<#crate_name::ResponseNodeId>> {
                    #(#resolvers)*
                    ::std::result::Result::Ok(::std::option::Option::None)
                }

                async fn find_entity(&self, ctx: &#crate_name::Context<'_>, params: &#crate_name::Value) -> #crate_name::ServerResult<::std::option::Option<#crate_name::Value>> {
                    let params = match params {
                        #crate_name::Value::Object(params) => params,
                        _ => return ::std::result::Result::Ok(::std::option::Option::None),
                    };
                    let typename = if let ::std::option::Option::Some(#crate_name::Value::String(typename)) = params.get("__typename") {
                        typename
                    } else {
                        return ::std::result::Result::Err(
                            #crate_name::ServerError::new(r#""__typename" must be an existing string."#, ::std::option::Option::Some(ctx.item.pos))
                        );
                    };
                    #(#find_entities_iter)*
                    ::std::result::Result::Ok(::std::option::Option::None)
                }
            }

            #[allow(clippy::all, clippy::pedantic)]
            #[#crate_name::async_trait::async_trait]
            impl #impl_generics #crate_name::LegacyOutputType for #self_ty #where_clause {
                fn type_name() -> ::std::borrow::Cow<'static, ::std::primitive::str> {
                    ::std::borrow::Cow::Borrowed(#gql_typename)
                }

                fn create_type_info(registry: &mut #crate_name::registry::Registry) -> #crate_name::registry::MetaFieldType {
                    let ty = registry.create_output_type::<Self, _>(|registry|
                        #crate_name::registry::MetaType::Object(#crate_name::registry::ObjectType {
                            name: ::std::borrow::ToOwned::to_owned(#gql_typename),
                            description: #desc,
                            fields: {
                                let mut fields = #crate_name::indexmap::IndexMap::new();
                                #(#schema_fields)*
                                fields
                            },
                            cache_control: #cache_control,
                            extends: #extends,
                            keys: ::std::option::Option::None,
                            is_node: false,
                            visible: #visible,
                            is_subscription: false,
                            rust_typename: ::std::borrow::ToOwned::to_owned(::std::any::type_name::<Self>()),
                            constraints: vec![],
                        })
                    );
                    #(#create_entity_types)*
                    #(#add_keys)*
                    ty
                }

                async fn resolve(
                    &self,
                    ctx: &#crate_name::ContextSelectionSet<'_>,
                    _field: &#crate_name::Positioned<#crate_name::parser::types::Field>
                ) -> #crate_name::ServerResult<#crate_name::ResponseNodeId> {
                    #resolve_container
                }
            }

            impl #impl_generics #crate_name::ObjectType for #self_ty #where_clause {}
        }
    } else {
        let mut codes = Vec::new();

        codes.push(quote! {
            #item_impl

            impl #impl_generics #self_ty #where_clause {
                fn __internal_create_type_info(registry: &mut #crate_name::registry::Registry, name: &str) -> #crate_name::registry::InputValueType  where Self: #crate_name::LegacyOutputType {
                    let ty = registry.create_output_type::<Self, _>(|registry|
                        #crate_name::registry::MetaType::Object(#crate_name::registry::ObjectType {
                            name: ::std::borrow::ToOwned::to_owned(name),
                            description: #desc,
                            fields: {
                                let mut fields = #crate_name::indexmap::IndexMap::new();
                                #(#schema_fields)*
                                fields
                            },
                            cache_control: #cache_control,
                            extends: #extends,
                            keys: ::std::option::Option::None,
                            visible: #visible,
                            is_node: false,
                            is_subscription: false,
                            rust_typename: ::std::borrow::ToOwned::to_owned(::std::any::type_name::<Self>()),
                            constraints: vec![],
                        })
                    );
                    #(#create_entity_types)*
                    #(#add_keys)*
                    ty
                }

                async fn __internal_resolve_field(&self, ctx: &#crate_name::Context<'_>) -> #crate_name::ServerResult<::std::option::Option<#crate_name::ResponseNodeId>> where Self: #crate_name::ContainerType {
                    #(#resolvers)*
                    ::std::result::Result::Ok(::std::option::Option::None)
                }

                async fn __internal_find_entity(&self, ctx: &#crate_name::Context<'_>, params: &#crate_name::Value) -> #crate_name::ServerResult<::std::option::Option<#crate_name::Value>> {
                    let params = match params {
                        #crate_name::Value::Object(params) => params,
                        _ => return ::std::result::Result::Ok(::std::option::Option::None),
                    };
                    let typename = if let ::std::option::Option::Some(#crate_name::Value::String(typename)) = params.get("__typename") {
                        typename
                    } else {
                        return ::std::result::Result::Err(
                            #crate_name::ServerError::new(r#""__typename" must be an existing string."#, ::std::option::Option::Some(ctx.item.pos))
                        );
                    };
                    #(#find_entities_iter)*
                    ::std::result::Result::Ok(::std::option::Option::None)
                }
            }
        });

        for concrete in &object_args.concretes {
            let gql_typename = &concrete.name;
            let params = &concrete.params.0;
            let ty = {
                let s = quote!(#self_ty).to_string();
                match s.rfind('<') {
                    Some(pos) => syn::parse_str(&s[..pos]).unwrap(),
                    None => self_ty.clone(),
                }
            };
            let concrete_type = quote! { #ty<#(#params),*> };

            codes.push(quote! {
                #[#crate_name::async_trait::async_trait]
                impl #crate_name::resolver_utils::ContainerType for #concrete_type {
                    async fn resolve_field(&self, ctx: &#crate_name::Context<'_>) -> #crate_name::ServerResult<::std::option::Option<#crate_name::ResponseNodeId>> {
                        self.__internal_resolve_field(ctx).await
                    }

                    async fn find_entity(&self, ctx: &#crate_name::Context<'_>, params: &#crate_name::Value) -> #crate_name::ServerResult<::std::option::Option<#crate_name::Value>> {
                        self.__internal_find_entity(ctx, params).await
                    }
                }

                #[#crate_name::async_trait::async_trait]
                impl #crate_name::LegacyOutputType for #concrete_type {
                    fn type_name() -> ::std::borrow::Cow<'static, ::std::primitive::str> {
                        ::std::borrow::Cow::Borrowed(#gql_typename)
                    }

                    fn create_type_info(registry: &mut #crate_name::registry::Registry) -> #crate_name::registry::MetaFieldType {
                        Self::__internal_create_type_info(registry, #gql_typename)
                    }

                    async fn resolve(
                        &self,
                        ctx: &#crate_name::ContextSelectionSet<'_>,
                        _field: &#crate_name::Positioned<#crate_name::parser::types::Field>
                    ) -> #crate_name::ServerResult<#crate_name::ResponseNodeId> {
                        #resolve_container
                    }
                }

                impl #crate_name::ObjectType for #concrete_type {}
            });
        }

        quote!(#(#codes)*)
    };

    Ok(expanded.into())
}
