mod bitpacked;
mod debug;
mod enum_;
mod walker;

use std::collections::HashSet;

use enum_::generate_enum;
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use tracing::instrument;

use crate::domain::{Definition, Domain, Scalar, Union, UnionKind, Variant};

use self::bitpacked::generate_bitpacked_id_union;

use super::{id::generate_id, GeneratedCode, Imports};

#[instrument(skip(domain))]
pub fn generate_union<'a>(domain: &'a Domain, union: &'a Union) -> anyhow::Result<GeneratedCode<'a>> {
    let mut imported_definition_names = HashSet::new();

    let variants = union
        .variants
        .iter()
        .map(|variant| {
            if let Some(ty) = variant.value_type_name.as_deref() {
                imported_definition_names.insert(ty);
            }
            Ok(VariantContext {
                domain,
                variant,
                value: variant
                    .value_type_name
                    .as_ref()
                    .map(|ty| {
                        domain
                            .definitions_by_name
                            .get(ty)
                            .ok_or_else(|| anyhow::anyhow!("Could not find type {}", ty))
                    })
                    .transpose()?,
            })
        })
        .collect::<anyhow::Result<Vec<_>>>()?;

    let mut code_sections = generate_enum(domain, union, &variants)?;
    if let Some(indexed) = union.indexed() {
        code_sections.extend(generate_id(domain, indexed)?);
    }
    code_sections.extend(walker::generate_walker(domain, union, &variants)?);
    if let UnionKind::BitpackedId(bitpacked) = &union.kind {
        code_sections.extend(generate_bitpacked_id_union(bitpacked, &variants)?);
    }

    Ok(GeneratedCode {
        module_path: &union.meta.module_path,
        code_sections,
        imports: Imports {
            generated: imported_definition_names,
            ..Default::default()
        },
    })
}

#[derive(Clone, Copy)]
pub struct VariantContext<'a> {
    domain: &'a Domain,
    variant: &'a Variant,
    value: Option<&'a Definition>,
}

impl<'a> VariantContext<'a> {
    pub fn value_type(&self) -> Option<TokenStream> {
        let value = self.value?;
        Some(match value {
            Definition::Scalar(Scalar::Value { copy, .. }) => {
                let walker = Ident::new(value.walker_name(), Span::call_site());
                if *copy {
                    quote! { #walker }
                } else {
                    quote! { &'a #walker }
                }
            }
            Definition::Scalar(Scalar::Ref { target, .. }) => {
                let walker = Ident::new(target.walker_name(), Span::call_site());
                quote! { #walker<'a> }
            }
            _ => {
                let walker = Ident::new(value.walker_name(), Span::call_site());
                quote! { #walker<'a> }
            }
        })
    }
}

impl<'a> std::ops::Deref for VariantContext<'a> {
    type Target = Variant;
    fn deref(&self) -> &Self::Target {
        self.variant
    }
}
