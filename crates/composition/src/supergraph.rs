mod render;

use crate::{subgraphs::DefinitionKind, StringId};
use std::collections::{BTreeMap, BTreeSet};

/// This is a **write only** data structure. The source of truth for the contents of the supergraph
/// is the subgraphs.
#[derive(Default, Debug)]
pub(crate) struct Supergraph {
    // We use BTreeMaps here in order to have a consistent ordering when rendering the supergraph
    // schema.
    definitions: BTreeMap<StringId, DefinitionKind>,
    // (definition_name, field_name) -> (arguments, field_type)
    fields: BTreeMap<(StringId, StringId), (Vec<(StringId, StringId)>, StringId)>,
    // (union_name, member_name)
    union_members: BTreeSet<(StringId, StringId)>,
}

impl Supergraph {
    /// # Panics
    ///
    /// If called twice with the same name.
    pub(crate) fn insert_definition(&mut self, name: StringId, kind: DefinitionKind) {
        if self.definitions.insert(name, kind).is_some() {
            panic!("Invariant broken: Supergraph::insert_definition() was called twice with the same name.");
        }
    }

    /// # Panics
    ///
    /// If called twice with the same parent and field name.
    pub(crate) fn insert_field(
        &mut self,
        parent_type_name: StringId,
        field_name: StringId,
        field_type: StringId,
        arguments: Vec<(StringId, StringId)>,
    ) {
        if self
            .fields
            .insert((parent_type_name, field_name), (arguments, field_type))
            .is_some()
        {
            panic!("Invariant broken: Supergraph::insert_field() was called twice with the same parent type and field name.");
        }
    }

    pub(crate) fn insert_union_member(
        &mut self,
        parent_union_name: StringId,
        member_name: StringId,
    ) {
        self.union_members.insert((parent_union_name, member_name));
    }
}
