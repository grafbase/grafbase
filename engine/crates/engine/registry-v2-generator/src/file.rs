use std::collections::BTreeSet;

use cynic_parser::type_system::TypeDefinition;
use proc_macro2::{Ident, Span};
use quote::quote;

use crate::{
    exts::{FileDirectiveExt, ScalarExt},
    format_code,
    idents::IdIdent,
};

#[derive(Clone, Debug, PartialEq, PartialOrd, Ord, Eq)]
pub struct EntityRef {
    module_name: String,
    name: String,
    has_id: bool,
}

impl EntityRef {
    pub fn new(ty: TypeDefinition<'_>) -> Option<Self> {
        match ty {
            TypeDefinition::Scalar(scalar) if scalar.is_inline() => None,
            TypeDefinition::Scalar(scalar) if scalar.reader_fn_override().is_some() => None,
            TypeDefinition::Scalar(_) => Some(EntityRef {
                module_name: ty.file_name().to_string(),
                name: ty.name().to_string(),
                has_id: true,
            }),
            TypeDefinition::Object(_) => Some(EntityRef {
                module_name: ty.file_name().to_string(),
                name: ty.name().to_string(),
                has_id: true,
            }),
            TypeDefinition::Union(_) => Some(EntityRef {
                module_name: ty.file_name().to_string(),
                name: ty.name().to_string(),
                has_id: true,
            }),
            _ => unimplemented!(),
        }
    }
}

pub struct EntityOutput {
    pub requires: BTreeSet<EntityRef>,
    pub id: EntityRef,
    pub contents: String,
    #[allow(dead_code)]
    pub kind: EntityKind,
}

#[derive(Clone, Copy, PartialEq)]
pub enum EntityKind {
    Union,
    Object,
}

pub fn imports(
    mut requires: BTreeSet<EntityRef>,
    current_file_entities: Vec<EntityRef>,
    shared_imports: proc_macro2::TokenStream,
) -> anyhow::Result<String> {
    for id in &current_file_entities {
        requires.remove(id);
    }

    let reader_imports = requires
        .iter()
        .map(|entity| {
            let module_name = Ident::new(&entity.module_name, Span::call_site());
            let entity_name = Ident::new(&entity.name, Span::call_site());
            quote! { #module_name::#entity_name, }
        })
        .collect::<Vec<_>>();

    let id_imports = requires
        .iter()
        .chain(current_file_entities.iter())
        .map(|entity| IdIdent(&entity.name))
        .map(|id| {
            quote! { #id, }
        })
        .collect::<Vec<_>>();

    format_code(quote! {
        #[allow(unused_imports)]
        use std::fmt::{self, Write};

        #shared_imports

        use super::{
            #(#reader_imports)*
            prelude::ids::{#(#id_imports)*},
        };
    })
}
