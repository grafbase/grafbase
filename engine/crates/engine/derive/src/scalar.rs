use proc_macro::TokenStream;
use quote::quote;
use syn::ItemImpl;

use crate::{
    args::{self, RenameTarget},
    utils::{get_crate_name, get_rustdoc, get_type_path_and_name, visible_fn, GeneratorResult},
};

pub fn generate(scalar_args: &args::Scalar, item_impl: &mut ItemImpl) -> GeneratorResult<TokenStream> {
    let crate_name = get_crate_name(scalar_args.internal);
    let self_name = get_type_path_and_name(item_impl.self_ty.as_ref())?.1;
    let gql_typename = scalar_args
        .name
        .clone()
        .unwrap_or_else(|| RenameTarget::Type.rename(self_name.clone()));

    let desc = if scalar_args.use_type_description {
        quote! { ::std::option::Option::Some(::std::borrow::ToOwned::to_owned(<Self as #crate_name::Description>::description())) }
    } else {
        get_rustdoc(&item_impl.attrs)
            .map(|s| quote!(::std::option::Option::Some(::std::borrow::ToOwned::to_owned(#s))))
            .unwrap_or_else(|| quote!(::std::option::Option::None))
    };

    let self_ty = &item_impl.self_ty;
    let generic = &item_impl.generics;
    let where_clause = &item_impl.generics.where_clause;
    let visible = visible_fn(&scalar_args.visible);
    let specified_by_url = match &scalar_args.specified_by_url {
        Some(specified_by_url) => {
            quote! { ::std::option::Option::Some(::std::borrow::ToOwned::to_owned(#specified_by_url)) }
        }
        None => quote! { ::std::option::Option::None },
    };

    let expanded = quote! {
        #item_impl

        #[allow(clippy::all, clippy::pedantic)]
        impl #generic #crate_name::LegacyInputType for #self_ty #where_clause {
            type RawValueType = Self;

            fn type_name() -> ::std::borrow::Cow<'static, ::std::primitive::str> {
                ::std::borrow::Cow::Borrowed(#gql_typename)
            }

            fn create_type_info(registry: &mut #crate_name::registry::Registry) -> #crate_name::registry::InputValueType {
                use #crate_name::registry::LegacyRegistryExt;
                registry.create_input_type::<#self_ty, _>(|_|
                    #crate_name::registry::MetaType::Scalar(#crate_name::registry::ScalarType {
                        name: ::std::borrow::ToOwned::to_owned(#gql_typename),
                        description: #desc,
                        is_valid: Some(|value| <#self_ty as #crate_name::LegacyScalarType>::is_valid(value)),

                        specified_by_url: #specified_by_url,
                        parser: #crate_name::registry::ScalarParser::BestEffort,
                    })
                )
            }

            fn parse(value: ::std::option::Option<#crate_name::Value>) -> #crate_name::InputValueResult<Self> {
                <#self_ty as #crate_name::LegacyScalarType>::parse(value.unwrap_or_default())
            }

            fn to_value(&self) -> #crate_name::Value {
                <#self_ty as #crate_name::LegacyScalarType>::to_value(self)
            }

            fn as_raw_value(&self) -> ::std::option::Option<&Self::RawValueType> {
                ::std::option::Option::Some(self)
            }
        }

        #[allow(clippy::all, clippy::pedantic)]
        #[#crate_name::async_trait::async_trait]
        impl #generic #crate_name::LegacyOutputType for #self_ty #where_clause {
            fn type_name() -> ::std::borrow::Cow<'static, ::std::primitive::str> {
                ::std::borrow::Cow::Borrowed(#gql_typename)
            }

            fn create_type_info(registry: &mut #crate_name::registry::Registry) -> #crate_name::registry::MetaFieldType {
                use #crate_name::registry::LegacyRegistryExt;
                registry.create_output_type::<#self_ty, _>(|_|
                    #crate_name::registry::MetaType::Scalar(#crate_name::registry::ScalarType {
                        name: ::std::borrow::ToOwned::to_owned(#gql_typename),
                        description: #desc,
                        is_valid: Some(|value| <#self_ty as #crate_name::LegacyScalarType>::is_valid(value)),

                        specified_by_url: #specified_by_url,
                        parser: #crate_name::registry::ScalarParser::BestEffort,
                    })
                )
            }

            async fn resolve(
                &self,
                ctx: &#crate_name::ContextSelectionSetLegacy<'_>,
                _field: &#crate_name::Positioned<#crate_name::parser::types::Field>
            ) -> #crate_name::ServerResult<#crate_name::ResponseNodeId> {
                #crate_name::resolver_utils::resolve_scalar_native(
                    ctx,
                    #crate_name::LegacyScalarType::to_value(self)
                ).await
            }
        }
    };
    Ok(expanded.into())
}
