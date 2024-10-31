use cynic_parser::common::WrappingType;
use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, TokenStreamExt};
use tracing::instrument;

use crate::{
    domain::{Domain, Object},
    generation::docstr,
};

use super::FieldContext;

#[instrument(skip_all)]
pub fn generate_struct(
    domain: &Domain,
    object: &Object,
    fields: &[FieldContext<'_>],
) -> anyhow::Result<Vec<TokenStream>> {
    let struct_name = Ident::new(&object.struct_name, Span::call_site());

    let additional_derives = {
        let mut derives = TokenStream::new();
        if !object.meta.derive.is_empty() {
            let names = object
                .meta
                .derive
                .iter()
                .map(|name| Ident::new(name, Span::call_site()));
            derives = quote! { ,#(#names),* };
        }
        if object.copy {
            derives.extend(quote! { , Clone, Copy })
        }
        derives
    };

    let struct_fields = fields.iter().map(StructField);
    let docstr = proc_macro2::Literal::string(&docstr::generated_from(
        domain,
        object.span,
        object.description.as_deref(),
    ));
    let object_struct = quote! {
        #[doc = #docstr]
        #[derive(Debug, serde::Serialize, serde::Deserialize #additional_derives)]
        pub struct #struct_name {
            #(#struct_fields),*
        }
    };

    Ok(vec![object_struct])
}

struct StructField<'a>(&'a FieldContext<'a>);

impl quote::ToTokens for StructField<'_> {
    #[instrument(name = "struct_field", skip_all, fields(field = ?self.0.field))]
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let field_name = Ident::new(&self.0.record_field_name, Span::call_site());

        let storage_type = self.0.ty.storage_type();
        let ty = Ident::new(&storage_type.to_string(), Span::call_site());

        let ty = match self.0.wrapping[..] {
            [] => quote! { Option<#ty> },
            [WrappingType::NonNull] => quote! { #ty },
            [WrappingType::NonNull, WrappingType::List, WrappingType::NonNull] => {
                if storage_type.list_as_id_range() {
                    quote! { IdRange<#ty> }
                } else {
                    quote! { Vec<#ty> }
                }
            }
            [WrappingType::NonNull, WrappingType::List, WrappingType::NonNull, WrappingType::List, WrappingType::NonNull] => {
                if storage_type.list_as_id_range() {
                    quote! { Vec<IdRange<#ty>> }
                } else {
                    quote! { Vec<Vec<#ty>> }
                }
            }
            _ => {
                tracing::error!("Unsupported wrapping {:?}", self.0.wrapping);
                unimplemented!()
            }
        };

        let doc = self
            .0
            .field
            .description
            .as_ref()
            .map(|desc| {
                let desc = proc_macro2::Literal::string(desc);
                quote! { #[doc = #desc] }
            })
            .unwrap_or_default();
        tokens.append_all(quote! {
            #doc
            pub #field_name: #ty
        });
    }
}
