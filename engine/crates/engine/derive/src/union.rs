use std::collections::HashSet;

use darling::ast::{Data, Style};
use proc_macro::TokenStream;
use quote::quote;
use syn::{visit_mut::VisitMut, Error, Type};

use crate::{
    args::{self, RenameTarget},
    utils::{get_crate_name, get_rustdoc, visible_fn, GeneratorResult, RemoveLifetime},
};

pub fn generate(union_args: &args::Union) -> GeneratorResult<TokenStream> {
    let crate_name = get_crate_name(union_args.internal);
    let ident = &union_args.ident;
    let (impl_generics, ty_generics, where_clause) = union_args.generics.split_for_impl();
    let s = match &union_args.data {
        Data::Enum(s) => s,
        _ => return Err(Error::new_spanned(ident, "Union can only be applied to an enum.").into()),
    };
    let mut enum_names = Vec::new();
    let mut enum_items = HashSet::new();
    let mut type_into_impls = Vec::new();
    let gql_typename = union_args
        .name
        .clone()
        .unwrap_or_else(|| RenameTarget::Type.rename(ident.to_string()));

    let desc = get_rustdoc(&union_args.attrs)
        .map(|s| quote! { ::std::option::Option::Some(::std::borrow::ToOwned::to_owned(#s)) })
        .unwrap_or_else(|| quote! {::std::option::Option::None});

    let mut registry_types = Vec::new();
    let mut possible_types = Vec::new();
    let mut get_introspection_typename = Vec::new();
    let mut collect_all_fields = Vec::new();

    for variant in s {
        let enum_name = &variant.ident;
        let ty = match variant.fields.style {
            Style::Tuple if variant.fields.fields.len() == 1 => &variant.fields.fields[0],
            Style::Tuple => {
                return Err(Error::new_spanned(enum_name, "Only single value variants are supported").into())
            }
            Style::Unit => return Err(Error::new_spanned(enum_name, "Empty variants are not supported").into()),
            Style::Struct => {
                return Err(Error::new_spanned(enum_name, "Variants with named fields are not supported").into())
            }
        };

        if let Type::Path(p) = &ty {
            // This validates that the field type wasn't already used
            if !enum_items.insert(p) {
                return Err(Error::new_spanned(ty, "This type already used in another variant").into());
            }

            enum_names.push(enum_name);

            let mut assert_ty = p.clone();
            RemoveLifetime.visit_type_path_mut(&mut assert_ty);

            if variant.flatten {
                type_into_impls.push(quote! {
                    #crate_name::static_assertions::assert_impl_one!(#assert_ty: #crate_name::LegacyUnionType);

                    #[allow(clippy::all, clippy::pedantic)]
                    impl #impl_generics ::std::convert::From<#p> for #ident #ty_generics #where_clause {
                        fn from(obj: #p) -> Self {
                            #ident::#enum_name(obj)
                        }
                    }
                });
            } else {
                type_into_impls.push(quote! {
                    #crate_name::static_assertions::assert_impl_one!(#assert_ty: #crate_name::ObjectType);

                    #[allow(clippy::all, clippy::pedantic)]
                    impl #impl_generics ::std::convert::From<#p> for #ident #ty_generics #where_clause {
                        fn from(obj: #p) -> Self {
                            #ident::#enum_name(obj)
                        }
                    }
                });
            };

            if variant.flatten {
                possible_types.push(quote! {
                    if let #crate_name::registry::MetaType::Union(union_type) =
                        registry.create_fake_output_type::<#p>() {
                        possible_types.extend(union_type.possible_types);
                    }
                });
            } else {
                registry_types.push(quote! {
                    <#p as #crate_name::LegacyOutputType>::create_type_info(registry);
                });
                possible_types.push(quote! {
                    possible_types.insert(<#p as #crate_name::LegacyOutputType>::type_name().into_owned());
                });
            }

            if variant.flatten {
                get_introspection_typename.push(quote! {
                    #ident::#enum_name(obj) => <#p as #crate_name::LegacyOutputType>::introspection_type_name(obj)
                });
            } else {
                get_introspection_typename.push(quote! {
                    #ident::#enum_name(obj) => <#p as #crate_name::LegacyOutputType>::type_name()
                });
            }

            collect_all_fields.push(quote! {
                #ident::#enum_name(obj) => obj.collect_all_fields_native(ctx, fields)
            });
        } else {
            return Err(Error::new_spanned(ty, "Invalid type").into());
        }
    }

    if possible_types.is_empty() {
        return Err(Error::new_spanned(
            ident,
            "A GraphQL Union type must include one or more unique member types.",
        )
        .into());
    }

    let visible = visible_fn(&union_args.visible);
    let expanded = quote! {
        #(#type_into_impls)*

        #[allow(clippy::all, clippy::pedantic)]
        #[#crate_name::async_trait::async_trait]

        impl #impl_generics #crate_name::resolver_utils::ContainerType for #ident #ty_generics #where_clause {
            async fn resolve_field(&self, ctx: &#crate_name::ContextField<'_>) -> #crate_name::ServerResult<::std::option::Option<#crate_name::ResponseNodeId>> {
                ::std::result::Result::Ok(::std::option::Option::None)
            }

            fn collect_all_fields_native<'__life>(&'__life self, ctx: &#crate_name::ContextSelectionSetLegacy<'__life>, fields: &mut #crate_name::resolver_utils::Fields<'__life>) -> #crate_name::ServerResult<()> {
                match self {
                    #(#collect_all_fields),*
                }
            }
        }

        #[allow(clippy::all, clippy::pedantic)]
        #[#crate_name::async_trait::async_trait]
        impl #impl_generics #crate_name::LegacyOutputType for #ident #ty_generics #where_clause {
            fn type_name() -> ::std::borrow::Cow<'static, ::std::primitive::str> {
               ::std::borrow::Cow::Borrowed(#gql_typename)
            }

            fn introspection_type_name(&self) -> ::std::borrow::Cow<'static, ::std::primitive::str> {
                match self {
                    #(#get_introspection_typename),*
                }
            }

            fn create_type_info(registry: &mut #crate_name::registry::Registry) -> #crate_name::registry::MetaFieldType {
                registry.create_output_type::<Self, _>(|registry| {
                    #(#registry_types)*

                    #crate_name::registry::MetaType::Union(#crate_name::registry::UnionType {
                        name: ::std::borrow::ToOwned::to_owned(#gql_typename),
                        description: #desc,
                        possible_types: {
                            let mut possible_types = #crate_name::indexmap::IndexSet::new();
                            #(#possible_types)*
                            possible_types
                        },
                        visible: #visible,
                        rust_typename: ::std::borrow::ToOwned::to_owned(::std::any::type_name::<Self>()),
                        discriminators: None
                    })
                })
            }

            async fn resolve(&self, ctx: &#crate_name::ContextSelectionSetLegacy<'_>, _field: &#crate_name::Positioned<#crate_name::parser::types::Field>) -> #crate_name::ServerResult<#crate_name::ResponseNodeId> {
                #crate_name::resolver_utils::resolve_container_native(ctx, self).await
            }
        }

        impl #impl_generics #crate_name::LegacyUnionType for #ident #ty_generics #where_clause {}
    };

    Ok(expanded.into())
}
