use indexmap::IndexMap;
use proc_macro2::{Ident, Literal, Span};
use quote::{quote, ToTokens, TokenStreamExt};

use cynic_parser::type_system::{FieldDefinition, ObjectDefinition, TypeDefinition};

use crate::{
    exts::{DistinctExt, FieldExt, ScalarExt},
    file::{EntityKind, EntityOutput, EntityRef},
    format_code,
    idents::IdIdent,
};

use self::{debug::ObjectDebug, equal::ObjectEqual};

mod debug;
mod equal;

pub fn object_output(
    object: ObjectDefinition<'_>,
    model_index: &IndexMap<&str, TypeDefinition<'_>>,
    id_trait: &str,
) -> anyhow::Result<EntityOutput> {
    let record_name = Ident::new(&format!("{}Record", object.name()), Span::call_site());
    let reader_name = Ident::new(object.name(), Span::call_site());
    let id_name = IdIdent(object.name());

    let edges = object
        .fields()
        .enumerate()
        .map(|(index, field)| -> anyhow::Result<FieldEdge> {
            Ok(FieldEdge {
                container: object,
                field,
                index,
                target: *model_index
                    .get(field.ty().name())
                    .ok_or_else(|| anyhow::anyhow!("Could not find type {}", field.ty().name()))?,
            })
        })
        .collect::<Result<Vec<_>, _>>()
        .unwrap();

    let record_fields = edges.iter().copied().map(ObjectField);
    let reader_functions = edges.iter().copied().map(ReaderFunction);

    let record = format_code(quote! {
        #[derive(serde::Serialize, serde::Deserialize)]
        pub struct #record_name {
            #(#record_fields),*
        }
    })?;

    let reader = format_code(quote! {
        #[derive(Clone, Copy)]
        pub struct #reader_name<'a>(pub(crate) ReadContext<'a, #id_name>);
    })?;

    let reader_debug = format_code(ObjectDebug(&reader_name, &edges).to_token_stream())?;

    let reader_impl = format_code(quote! {
        impl <'a> #reader_name<'a> {
            #(#reader_functions)*
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
                Self(value)
            }
        }
    })?;

    let reader_eq = if object.is_distinct() {
        let eq = ObjectEqual(&reader_name, &edges);
        format_code(eq.to_token_stream())?
    } else {
        "".to_string()
    };

    let contents = indoc::formatdoc!(
        r#"
        {record}

        {reader}

        {reader_impl}

        {reader_debug}

        {reader_eq}

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
        id: EntityRef::new(TypeDefinition::Object(object)).unwrap(),
        contents,
        kind: EntityKind::Object,
    })
}

#[derive(Clone, Copy)]
pub struct FieldEdge<'a> {
    #[allow(dead_code)]
    container: ObjectDefinition<'a>,
    index: usize,
    field: FieldDefinition<'a>,
    target: TypeDefinition<'a>,
}

pub struct ObjectField<'a>(FieldEdge<'a>);

impl quote::ToTokens for ObjectField<'_> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let field_name = Ident::new(self.0.field.name(), Span::call_site());

        let target_id = IdIdent(self.0.target.name());

        let ty = match self.0.target {
            TypeDefinition::Scalar(scalar) if scalar.should_box() => {
                if self.0.field.ty().is_list() || self.0.field.ty().is_non_null() {
                    unimplemented!("boxed scalars cant be non-null or lists ")
                }
                let ident = Ident::new(self.0.target.name(), Span::call_site());
                quote! { Option<Box<#ident>> }
            }
            TypeDefinition::Scalar(scalar) if scalar.is_inline() => {
                // I'm assuming inline scalars are copy here.
                let ident = Ident::new(self.0.target.name(), Span::call_site());
                if self.0.field.ty().is_list() {
                    quote! { Vec<#ident> }
                } else if self.0.field.ty().is_non_null() {
                    quote! { #ident }
                } else {
                    quote! { Option<#ident> }
                }
            }
            TypeDefinition::Scalar(scalar) if scalar.reader_fn_override().is_some() => {
                if self.0.field.ty().is_list() {
                    quote! { Vec<#target_id> }
                } else if self.0.field.ty().is_non_null() {
                    quote! { #target_id }
                } else {
                    quote! { Option<#target_id> }
                }
            }
            TypeDefinition::Object(_) | TypeDefinition::Union(_) | TypeDefinition::Scalar(_) => {
                if self.0.field.is_inline() {
                    if !self.0.field.ty().is_list() {
                        unimplemented!("need to add non list inline types apparently")
                    }
                    quote! { Vec<#target_id> }
                } else if self.0.field.ty().is_list() {
                    quote! { IdRange<#target_id> }
                } else if self.0.field.ty().is_non_null() {
                    quote! { #target_id }
                } else {
                    quote! { Option<#target_id> }
                }
            }
            _ => unimplemented!(),
        };

        let rename = Literal::string(&self.0.index.to_string());
        let other_attrs = if self.0.field.ty().is_list() {
            quote! { , skip_serializing_if = "crate::Container::is_empty", default }
        } else if !self.0.field.ty().is_non_null() {
            quote! { , skip_serializing_if = "Option::is_none", default }
        } else if self.0.field.ty().is_non_null() && self.0.target.name() == "bool" {
            quote! { , skip_serializing_if = "crate::is_false", default }
        } else {
            quote! {}
        };

        tokens.append_all(quote! {
            #[serde(rename = #rename #other_attrs )]
            pub #field_name: #ty
        });
    }
}

pub struct ReaderFunction<'a>(FieldEdge<'a>);

impl quote::ToTokens for ReaderFunction<'_> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        if !self.0.field.should_have_reader_fn() {
            return;
        }
        let field_name = Ident::new(self.0.field.name(), Span::call_site());
        let target_type = Ident::new(self.0.target.name(), Span::call_site());

        let inner_ty = match self.0.target {
            TypeDefinition::Scalar(scalar) if scalar.is_inline() => {
                // I'm assuming inline scalars are copy here.
                quote! { #target_type }
            }
            TypeDefinition::Scalar(scalar) if scalar.reader_fn_override().is_some() => {
                scalar.reader_fn_override().unwrap()
            }
            TypeDefinition::Object(_) | TypeDefinition::Union(_) | TypeDefinition::Scalar(_) => {
                quote! { #target_type<'a> }
            }
            _ => unimplemented!(),
        };

        // This match is a monster, sorry about that.
        tokens.append_all(match self.0.target {
            TypeDefinition::Scalar(scalar) if scalar.should_box() => {
                if self.0.field.ty().is_list() || self.0.field.ty().is_non_null() {
                    unimplemented!("boxed scalars cant be non-null or lists ")
                }
                quote! {
                    pub fn #field_name(&self) -> Option<&'a #inner_ty> {
                        let registry = self.0.registry;

                        registry.lookup(self.0.id).#field_name.as_deref()
                    }
                }
            }
            TypeDefinition::Scalar(scalar) if scalar.reader_returns_ref() => {
                if self.0.field.ty().is_list() || !self.0.field.ty().is_non_null() {
                    unimplemented!("ref scalars cant be optional or lists unless you implement it");
                }

                quote! {
                    pub fn #field_name(&self) -> &'a #inner_ty {
                        let registry = self.0.registry;

                        &registry.lookup(self.0.id).#field_name
                    }
                }
            }
            TypeDefinition::Scalar(scalar) if scalar.is_inline() && self.0.field.ty().is_list() => {
                // I'm assuming inline scalars are copy here.
                quote! {
                    pub fn #field_name(&self) -> impl ExactSizeIterator<Item = #inner_ty> + 'a {
                        let registry = self.0.registry;

                        registry.lookup(self.0.id).#field_name.iter().copied()
                    }
                }
            }
            TypeDefinition::Scalar(scalar) if scalar.is_inline() => {
                let ty = if self.0.field.ty().is_non_null() || self.0.field.should_default() {
                    quote! { #inner_ty }
                } else {
                    quote! { Option<#inner_ty> }
                };

                let default_unwrap = if self.0.field.should_default() {
                    quote! { .unwrap_or_default() }
                } else {
                    quote! {}
                };

                // I'm assuming inline scalars are copy here.
                quote! {
                    pub fn #field_name(&self) -> #ty {
                        let registry = self.0.registry;

                        registry.lookup(self.0.id).#field_name #default_unwrap
                    }
                }
            }
            TypeDefinition::Scalar(scalar) if scalar.reader_fn_override().is_some() && self.0.field.ty().is_list() => {
                // Scalars with reader_fn_override return the scalar directly _not_ a reader
                quote! {
                    pub fn #field_name(&self) -> impl ExactSizeIterator<Item = #inner_ty> + 'a {
                        let registry = &self.0.registry;

                        registry.lookup(self.0.id).#field_name.iter().map(|id| registry.lookup(*id))
                    }
                }
            }
            TypeDefinition::Scalar(scalar)
                if scalar.reader_fn_override().is_some() && self.0.field.ty().is_non_null() =>
            {
                // Scalars with reader_fn_override return the scalar directly _not_ a reader
                quote! {
                    pub fn #field_name(&self) -> #inner_ty {
                        let registry = &self.0.registry;

                        registry.lookup(registry.lookup(self.0.id).#field_name)
                    }
                }
            }
            TypeDefinition::Scalar(scalar) if scalar.reader_fn_override().is_some() => {
                // Scalars with reader_fn_override return the scalar directly _not_ a reader
                quote! {
                    pub fn #field_name(&self) -> Option<#inner_ty> {
                        let registry = self.0.registry;

                        registry.lookup(self.0.id).#field_name.map(|id| registry.lookup(id))
                    }
                }
            }
            TypeDefinition::Object(_) | TypeDefinition::Union(_) | TypeDefinition::Scalar(_)
                if self.0.field.is_inline() =>
            {
                quote! {
                    pub fn #field_name(&self) -> impl ExactSizeIterator<Item = #inner_ty> + 'a {
                        let registry = self.0.registry;

                        registry.lookup(self.0.id).#field_name.iter().map(|id| registry.read(*id))
                    }
                }
            }
            TypeDefinition::Object(_) | TypeDefinition::Union(_) | TypeDefinition::Scalar(_)
                if self.0.field.ty().is_list() =>
            {
                quote! {
                    pub fn #field_name(&self) -> Iter<'a, #inner_ty> {
                        let registry = self.0.registry;

                        Iter::new(registry.lookup(self.0.id).#field_name, registry)
                    }
                }
            }
            TypeDefinition::Object(_) | TypeDefinition::Union(_) | TypeDefinition::Scalar(_)
                if self.0.field.ty().is_non_null() =>
            {
                quote! {
                    pub fn #field_name(&self) -> #inner_ty {
                        let registry = self.0.registry;

                        registry.read(registry.lookup(self.0.id).#field_name)
                    }
                }
            }
            TypeDefinition::Object(_) | TypeDefinition::Union(_) | TypeDefinition::Scalar(_) => {
                quote! {
                    pub fn #field_name(&self) -> Option<#inner_ty> {
                        let registry = self.0.registry;

                        registry.lookup(self.0.id).#field_name.map(|id| registry.read(id))
                    }
                }
            }
            _ => unimplemented!("No support for this target type"),
        });
    }
}
