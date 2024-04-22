#![allow(clippy::option_if_let_else)]
#![allow(clippy::useless_let_if_seq)]
use std::str::FromStr;

use darling::ast::Data;
use proc_macro::TokenStream;
use quote::quote;
use syn::{ext::IdentExt, visit::Visit, Error, Ident, Lifetime, Path, Type};

use crate::{
    args::{self, RenameRuleExt, RenameTarget, SimpleObjectField},
    utils::{gen_deprecation, generate_guards, get_crate_name, get_rustdoc, visible_fn, GeneratorResult},
};

#[derive(Debug)]
struct DerivedFieldMetadata {
    ident: Ident,
    into: Type,
    owned: Option<bool>,
    with: Option<Path>,
}

struct SimpleObjectFieldGenerator<'a> {
    field: &'a SimpleObjectField,
    derived: Option<DerivedFieldMetadata>,
}

pub fn generate(object_args: &args::SimpleObject) -> GeneratorResult<TokenStream> {
    let crate_name = get_crate_name(object_args.internal);
    let ident = &object_args.ident;
    let (impl_generics, ty_generics, where_clause) = object_args.generics.split_for_impl();
    let extends = object_args.extends;
    let gql_typename = object_args
        .name
        .clone()
        .unwrap_or_else(|| RenameTarget::Type.rename(ident.to_string()));

    let desc = get_rustdoc(&object_args.attrs)
        .map(|s| quote! { ::std::option::Option::Some(::std::borrow::ToOwned::to_owned(#s)) })
        .unwrap_or_else(|| quote! {::std::option::Option::None});

    let s = match &object_args.data {
        Data::Struct(e) => e,
        _ => return Err(Error::new_spanned(ident, "SimpleObject can only be applied to an struct.").into()),
    };
    let mut getters = Vec::new();
    let mut resolvers = Vec::new();
    let mut schema_fields = Vec::new();

    let mut processed_fields: Vec<SimpleObjectFieldGenerator<'_>> = vec![];

    // Before processing the fields, we generate the derivated fields
    for field in &s.fields {
        processed_fields.push(SimpleObjectFieldGenerator { field, derived: None });

        for derived in &field.derived {
            if derived.name.is_some() && derived.into.is_some() {
                let name = derived.name.clone().unwrap();
                let into = match syn::parse2::<Type>(
                    proc_macro2::TokenStream::from_str(&derived.into.clone().unwrap()).unwrap(),
                ) {
                    Ok(e) => e,
                    _ => {
                        return Err(Error::new_spanned(&name, "derived into must be a valid type.").into());
                    }
                };

                let derived = DerivedFieldMetadata {
                    ident: name,
                    into,
                    owned: derived.owned,
                    with: derived.with.clone(),
                };

                processed_fields.push(SimpleObjectFieldGenerator {
                    field,
                    derived: Some(derived),
                });
            }
        }
    }

    for SimpleObjectFieldGenerator { field, derived } in &processed_fields {
        if field.skip || field.skip_output {
            continue;
        }

        let base_ident = match &field.ident {
            Some(ident) => ident,
            None => return Err(Error::new_spanned(ident, "All fields must be named.").into()),
        };

        let ident = if let Some(derived) = derived {
            &derived.ident
        } else {
            base_ident
        };

        let field_name = field.name.clone().unwrap_or_else(|| {
            object_args
                .rename_fields
                .rename(ident.unraw().to_string(), RenameTarget::Field)
        });
        let field_desc = get_rustdoc(&field.attrs)
            .map(|s| quote! {::std::option::Option::Some(::std::borrow::ToOwned::to_owned(#s))})
            .unwrap_or_else(|| quote! {::std::option::Option::None});
        let field_deprecation = gen_deprecation(&field.deprecation, &crate_name);
        let external = field.external;
        let requires = match &field.requires {
            Some(requires) => quote! { ::std::option::Option::Some(#requires) },
            None => quote! { ::std::option::Option::None },
        };
        let provides = match &field.provides {
            Some(provides) => quote! { ::std::option::Option::Some(#provides) },
            None => quote! { ::std::option::Option::None },
        };
        let vis = &field.vis;

        let ty = if let Some(derived) = derived {
            &derived.into
        } else {
            &field.ty
        };

        let owned = if let Some(derived) = derived {
            derived.owned.unwrap_or(field.owned)
        } else {
            field.owned
        };

        let cache_control = {
            let public = field.cache_control.is_public();
            let max_age = field.cache_control.max_age;
            let stale_while_revalidate = field.cache_control.stale_while_revalidate;
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

        let visible = visible_fn(&field.visible);

        if field.flatten {
            schema_fields.push(quote! {
                #crate_name::static_assertions::assert_impl_one!(#ty: #crate_name::ObjectType);
                #ty::create_type_info(registry);
                if let #crate_name::registry::MetaType::Object { fields: obj_fields, .. } =
                    registry.create_fake_output_type::<#ty>() {
                    fields.extend(obj_fields);
                }
            });
        } else {
            schema_fields.push(quote! {
                fields.insert(::std::borrow::ToOwned::to_owned(#field_name), #crate_name::registry::MetaField {
                    name: ::std::borrow::ToOwned::to_owned(#field_name),
                    mapped_name: None,
                    description: #field_desc,
                    args: ::std::default::Default::default(),
                    ty: <#ty as #crate_name::LegacyOutputType>::create_type_info(registry),
                    deprecation: #field_deprecation,
                    cache_control: Some(Box::new(#cache_control)),
                    resolver: #crate_name::registry::resolvers::Resolver::Parent,
                    ..Default::default()
                });
            });
        }

        let guard_map_err = quote! {
            .map_err(|err| err.into_server_error(ctx.item.pos))
        };
        let guard = match &field.guard {
            Some(code) => Some(generate_guards(&crate_name, code, guard_map_err)?),
            None => None,
        };

        let with_function = derived.as_ref().and_then(|x| x.with.as_ref());

        let mut block = if owned {
            quote! {
                ::std::clone::Clone::clone(&self.#base_ident)
            }
        } else {
            quote! {
                &self.#base_ident
            }
        };

        block = match (derived, with_function) {
            (Some(_), Some(with)) => quote! {
                #with(#block)
            },
            (Some(_), None) => quote! {
                ::std::convert::Into::into(#block)
            },
            (_, _) => block,
        };

        let ty = if owned {
            quote! { #ty }
        } else {
            quote! { &#ty }
        };

        if field.flatten {
            resolvers.push(quote! {
                if let ::std::option::Option::Some(value) = #crate_name::ContainerType::resolve_field(&self.#ident, ctx).await? {
                    return ::std::result::Result::Ok(std::option::Option::Some(value));
                }
            });
        } else {
            getters.push(quote! {
                 #[inline]
                 #[allow(missing_docs)]
                 #vis async fn #ident(&self, ctx: &#crate_name::ContextField<'_>) -> #crate_name::Result<#ty> {
                     ::std::result::Result::Ok(#block)
                 }
            });
            resolvers.push(quote! {
                if ctx.item.node.name.node == #field_name {
                    use #crate_name::context::ContextExt;
                    let f = async move {
                        #guard
                        self.#ident(ctx).await.map_err(|err| err.into_server_error(ctx.item.pos))
                    };
                    let obj = f.await.map_err(|err| ctx.set_error_path(err))?;
                    let ctx_obj = ctx.with_selection_set_legacy(&ctx.item.node.selection_set);
                    return #crate_name::LegacyOutputType::resolve(&obj, &ctx_obj, ctx.item).await.map(::std::option::Option::Some);
                }
            });
        }
    }

    if !object_args.fake && resolvers.is_empty() {
        return Err(Error::new_spanned(ident, "A GraphQL Object type must define one or more fields.").into());
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

    let visible = visible_fn(&object_args.visible);

    let mut concat_complex_fields = quote!();
    let mut complex_resolver = quote!();

    if object_args.complex {
        concat_complex_fields = quote! {
            fields.extend(<Self as #crate_name::ComplexObject>::fields(registry));
        };
        complex_resolver = quote! {
            if let Some(value) = <Self as #crate_name::ComplexObject>::resolve_field(self, ctx).await? {
                return Ok(Some(value));
            }
        };
    }

    let resolve_container = if object_args.serial {
        quote! { #crate_name::resolver_utils::resolve_container_serial(ctx, self).await }
    } else {
        quote! { #crate_name::resolver_utils::resolve_container_native(ctx, self).await }
    };

    let expanded = if object_args.concretes.is_empty() {
        quote! {
            #[allow(clippy::all, clippy::pedantic)]
            impl #impl_generics #ident #ty_generics #where_clause {
                #(#getters)*
            }

            #[allow(clippy::all, clippy::pedantic)]
            #[#crate_name::async_trait::async_trait]

            impl #impl_generics #crate_name::resolver_utils::ContainerType for #ident #ty_generics #where_clause {
                async fn resolve_field(&self, ctx: &#crate_name::ContextField<'_>) -> #crate_name::ServerResult<::std::option::Option<#crate_name::ResponseNodeId>> {
                    #(#resolvers)*
                    #complex_resolver
                    ::std::result::Result::Ok(::std::option::Option::None)
                }
            }

            #[allow(clippy::all, clippy::pedantic)]
            #[#crate_name::async_trait::async_trait]
            impl #impl_generics #crate_name::LegacyOutputType for #ident #ty_generics #where_clause {
                fn type_name() -> ::std::borrow::Cow<'static, ::std::primitive::str> {
                    ::std::borrow::Cow::Borrowed(#gql_typename)
                }

                fn create_type_info(registry: &mut #crate_name::registry::Registry) -> #crate_name::registry::MetaFieldType {
                    use crate::registry::LegacyRegistryExt;
                    registry.create_output_type::<Self, _>(|registry|
                        #crate_name::registry::MetaType::Object(#crate_name::registry::ObjectType {
                            name: ::std::borrow::ToOwned::to_owned(#gql_typename),
                            description: #desc,
                            fields: {
                                let mut fields = #crate_name::indexmap::IndexMap::new();
                                #(#schema_fields)*
                                #concat_complex_fields
                                fields
                            },
                            cache_control: Some(Box::new(#cache_control)),
                            extends: #extends,

                            is_subscription: false,
                            is_node: false,
                            rust_typename: ::std::borrow::ToOwned::to_owned(::std::any::type_name::<Self>()),
                            constraints: vec![],
                            external: false,
                            shareable: false
                        })
                    )
                }

                async fn resolve(&self, ctx: &#crate_name::ContextSelectionSetLegacy<'_>, _field: &#crate_name::Positioned<#crate_name::parser::types::Field>) -> #crate_name::ServerResult<#crate_name::ResponseNodeId> {
                    #resolve_container
                }
            }

            impl #impl_generics #crate_name::ObjectType for #ident #ty_generics #where_clause {}
        }
    } else {
        let mut code = Vec::new();

        #[derive(Default)]
        struct GetLifetimes<'a> {
            lifetimes: Vec<&'a Lifetime>,
        }

        impl<'a> Visit<'a> for GetLifetimes<'a> {
            fn visit_lifetime(&mut self, i: &'a Lifetime) {
                self.lifetimes.push(i);
            }
        }

        let mut visitor = GetLifetimes::default();
        visitor.visit_generics(&object_args.generics);
        let lifetimes = visitor.lifetimes;

        let def_lifetimes = if lifetimes.is_empty() {
            None
        } else {
            Some(quote!(<#(#lifetimes),*>))
        };

        let type_lifetimes = if lifetimes.is_empty() {
            None
        } else {
            Some(quote!(#(#lifetimes,)*))
        };

        code.push(quote! {
            impl #impl_generics #ident #ty_generics #where_clause {
                #(#getters)*

                fn __internal_create_type_info(
                    registry: &mut #crate_name::registry::Registry,
                    name: &str,
                    complex_fields: #crate_name::indexmap::IndexMap<::std::string::String, #crate_name::registry::MetaField>,
                ) -> ::std::string::String where Self: #crate_name::LegacyOutputType {
                    use crate::registry::LegacyRegistryExt;
                    registry.create_output_type::<Self, _>(|registry|
                        #crate_name::registry::MetaType::Object(#crate_name::registry::ObjectType {
                            name: ::std::borrow::ToOwned::to_owned(name),
                            description: #desc,
                            fields: {
                                let mut fields = #crate_name::indexmap::IndexMap::new();
                                #(#schema_fields)*
                                ::std::iter::Extend::extend(&mut fields, complex_fields.clone());
                                fields
                            },
                            cache_control: Some(Box::new(#cache_control)),
                            extends: #extends,

                            is_subscription: false,
                            is_node: false,
                            rust_typename: ::std::borrow::ToOwned::to_owned(::std::any::type_name::<Self>()),
                            resolver: #crate_name::registry::resolvers::Resolver::Parent,
                            required_operation: None,
                            auth: None,
                            constraints: vec![],
                            external: false,
                            shareable: false
                        })
                    )
                }

                async fn __internal_resolve_field(&self, ctx: &#crate_name::ContextField<'_>) -> #crate_name::ServerResult<::std::option::Option<#crate_name::ResponseNodeId>> where Self: #crate_name::ContainerType {
                    #(#resolvers)*
                    ::std::result::Result::Ok(::std::option::Option::None)
                }
            }
        });

        for concrete in &object_args.concretes {
            let gql_typename = &concrete.name;
            let params = &concrete.params.0;
            let concrete_type = quote! { #ident<#type_lifetimes #(#params),*> };

            let expanded = quote! {
                #[allow(clippy::all, clippy::pedantic)]
                #[#crate_name::async_trait::async_trait]
                impl #def_lifetimes #crate_name::resolver_utils::ContainerType for #concrete_type {
                    async fn resolve_field(&self, ctx: &#crate_name::ContextField<'_>) -> #crate_name::ServerResult<::std::option::Option<#crate_name::ResponseNodeId>> {
                        #complex_resolver
                        self.__internal_resolve_field(ctx).await
                    }
                }

                #[allow(clippy::all, clippy::pedantic)]
                #[#crate_name::async_trait::async_trait]
                impl #def_lifetimes #crate_name::LegacyOutputType for #concrete_type {
                    fn type_name() -> ::std::borrow::Cow<'static, ::std::primitive::str> {
                        ::std::borrow::Cow::Borrowed(#gql_typename)
                    }

                    fn create_type_info(registry: &mut #crate_name::registry::Registry) -> #crate_name::registry::MetaFieldType {
                        let mut fields = #crate_name::indexmap::IndexMap::new();
                        #concat_complex_fields
                        Self::__internal_create_type_info(registry, #gql_typename, fields)
                    }

                    async fn resolve(&self, ctx: &#crate_name::ContextSelectionSetLegacy<'_>, _field: &#crate_name::Positioned<#crate_name::parser::types::Field>) -> #crate_name::ServerResult<#crate_name::ResponseNodeId> {
                        #resolve_container
                    }
                }

                impl #def_lifetimes #crate_name::ObjectType for #concrete_type {}
            };
            code.push(expanded);
        }

        quote!(#(#code)*)
    };

    Ok(expanded.into())
}
