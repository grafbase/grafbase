use indexmap::IndexMap;
use proc_macro2::{Ident, Literal, Span};
use quote::{quote, TokenStreamExt};

use cynic_parser::type_system::{TypeDefinition, UnionDefinition};

use crate::{
    exts::{DistinctExt, UnionExt},
    file::{EntityKind, EntityOutput, EntityRef},
    format_code,
    idents::IdIdent,
};

pub fn union_output(
    union_definition: UnionDefinition<'_>,
    model_index: &IndexMap<&str, TypeDefinition<'_>>,
    id_trait: &str,
) -> anyhow::Result<EntityOutput> {
    let record_name = Ident::new(&format!("{}Record", union_definition.name()), Span::call_site());
    let reader_name = Ident::new(union_definition.name(), Span::call_site());
    let id_name = IdIdent(union_definition.name());

    let edges = union_definition
        .members()
        .enumerate()
        .map(|(variant_index, ty)| -> anyhow::Result<TypeEdge> {
            let target = *model_index
                .get(ty.name())
                .ok_or_else(|| anyhow::anyhow!("Could not find type {ty}", ty = ty.name()))?;

            Ok(TypeEdge {
                index: variant_index,
                container: union_definition,
                variant_name: union_definition
                    .variant_name_override(variant_index)
                    .unwrap_or(target.name()),
                target,
            })
        })
        .collect::<Result<Vec<_>, _>>()
        .unwrap();

    let record_variants = edges.iter().copied().map(RecordVariant);
    let reader_variants = edges.iter().copied().map(ReaderVariant);
    let from_branches = edges.iter().copied().map(FromBranch);

    let record = format_code(quote! {
        #[derive(serde::Serialize, serde::Deserialize)]
        pub enum #record_name {
            #(#record_variants),*
        }
    })?;

    let additional_derives = if union_definition.is_distinct() {
        quote! { , PartialEq, Eq }
    } else {
        quote! {}
    };

    let reader = format_code(quote! {
        #[derive(Clone, Copy, Debug #additional_derives)]
        pub enum #reader_name<'a> {
            #(#reader_variants),*
        }
    })?;

    let id_trait = Ident::new(id_trait, Span::call_site());

    let id_trait_impl = format_code(quote! {
        impl #id_trait for #id_name {
            type Reader<'a> = #reader_name<'a>;
        }
    })?;

    let id_reader_impl = format_code(quote! {
        impl IdReader for #reader_name<'_> {
            type Id = #id_name;
        }
    })?;

    let from_impl = format_code(quote! {
        impl <'a> From<ReadContext<'a, #id_name>> for #reader_name<'a> {
            fn from(value: ReadContext<'a, #id_name>) -> Self {
                match value.registry.lookup(value.id) {
                    #(#from_branches),*
                }
            }
        }
    })?;

    let contents = indoc::formatdoc!(
        r#"
        {record}

        {reader}

        {id_trait_impl}

        {id_reader_impl}

        {from_impl}
    "#
    );

    Ok(EntityOutput {
        requires: edges
            .iter()
            .copied()
            .filter_map(|edge| EntityRef::new(edge.target))
            .collect(),
        id: EntityRef::new(TypeDefinition::Union(union_definition)).unwrap(),
        contents,
        kind: EntityKind::Union,
    })
}

#[derive(Clone, Copy)]
pub struct TypeEdge<'a> {
    index: usize,
    container: UnionDefinition<'a>,
    variant_name: &'a str,
    target: TypeDefinition<'a>,
}

pub struct RecordVariant<'a>(TypeEdge<'a>);

impl quote::ToTokens for RecordVariant<'_> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let variant_name = Ident::new(self.0.variant_name, Span::call_site());
        let id = IdIdent(self.0.target.name());
        let rename = Literal::string(&self.0.index.to_string());

        tokens.append_all(quote! {
            #[serde(rename = #rename)]
            #variant_name(#id)
        });
    }
}

pub struct ReaderVariant<'a>(TypeEdge<'a>);

impl quote::ToTokens for ReaderVariant<'_> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let variant_name = Ident::new(self.0.variant_name, Span::call_site());
        let reader = Ident::new(self.0.target.name(), Span::call_site());

        tokens.append_all(quote! {
            #variant_name(#reader<'a>)
        });
    }
}

pub struct FromBranch<'a>(TypeEdge<'a>);

impl quote::ToTokens for FromBranch<'_> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let this_record = Ident::new(&format!("{}Record", self.0.container.name()), Span::call_site());
        let this_reader = Ident::new(self.0.container.name(), Span::call_site());
        let variant_name = Ident::new(self.0.variant_name, Span::call_site());

        tokens.append_all(quote! {
            #this_record::#variant_name(id) => #this_reader::#variant_name(value.registry.read(*id))
        });
    }
}
