#![doc = include_str!("../README.md")]
#![allow(unused_crate_dependencies)]
#![deny(missing_docs)]

mod change;
mod patch;
mod state;
mod traverse_schemas;

pub use self::{
    change::{Change, ChangeKind, Span},
    patch::{patch, PatchedSchema},
};

use self::state::*;
use cynic_parser::type_system as ast;
use std::collections::HashMap;

/// Diff two GraphQL schemas.
pub fn diff(source: &str, target: &str) -> Result<Vec<Change>, cynic_parser::Error> {
    let [source, target] = [source, target].map(|sdl| -> Result<_, cynic_parser::Error> {
        if sdl.trim().is_empty() {
            Ok(None)
        } else {
            Ok(Some(cynic_parser::parse_type_system_document(sdl)?))
        }
    });
    let [source, target] = [source?, target?];

    let mut state = DiffState::default();

    traverse_schemas::traverse_schemas([source.as_ref(), target.as_ref()], &mut state);

    Ok(state.into_changes())
}

/// Resolve the spans from [Change]s and the corresponding schemas.
pub fn resolve_spans<'a: 'b, 'b>(
    source: &'a str,
    target: &'a str,
    changes: &'b [Change],
) -> impl Iterator<Item = &'a str> + 'b {
    changes.iter().map(move |change| {
        let relevant_schema = match change.kind {
            ChangeKind::ChangeQueryType => target,
            ChangeKind::ChangeMutationType => target,
            ChangeKind::ChangeSubscriptionType => target,
            ChangeKind::RemoveObjectType => source,
            ChangeKind::AddObjectType => target,
            ChangeKind::AddInterfaceImplementation => target,
            ChangeKind::RemoveInterfaceImplementation => source,
            ChangeKind::ChangeFieldType => target,
            ChangeKind::RemoveField => source,
            ChangeKind::AddField => target,
            ChangeKind::AddUnion => target,
            ChangeKind::RemoveUnion => source,
            ChangeKind::AddUnionMember => target,
            ChangeKind::RemoveUnionMember => source,
            ChangeKind::AddEnum => target,
            ChangeKind::RemoveEnum => source,
            ChangeKind::AddEnumValue => target,
            ChangeKind::RemoveEnumValue => source,
            ChangeKind::AddScalar => target,
            ChangeKind::RemoveScalar => source,
            ChangeKind::AddInterface => target,
            ChangeKind::RemoveInterface => source,
            ChangeKind::AddDirectiveDefinition => target,
            ChangeKind::RemoveDirectiveDefinition => source,
            ChangeKind::AddSchemaDefinition => target,
            ChangeKind::AddSchemaExtension => target,
            ChangeKind::RemoveSchemaExtension => source,
            ChangeKind::RemoveSchemaDefinition => source,
            ChangeKind::AddInputObject => target,
            ChangeKind::RemoveInputObject => source,
            ChangeKind::AddFieldArgument => target,
            ChangeKind::RemoveFieldArgument => source,
            ChangeKind::AddFieldArgumentDefault => target,
            ChangeKind::RemoveFieldArgumentDefault => source,
            ChangeKind::ChangeFieldArgumentDefault => target,
            ChangeKind::ChangeFieldArgumentType => target,
        };

        &relevant_schema[change.span]
    })
}
