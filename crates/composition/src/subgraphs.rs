mod walkers;

pub(crate) use walkers::*;

use crate::strings::{StringId, Strings};
use itertools::Itertools;
use std::collections::BTreeSet;

/// A set of subgraphs to be composed.
#[derive(Default)]
pub struct Subgraphs {
    pub(crate) strings: Strings,
    subgraphs: Vec<Subgraph>,

    // Invariant: `fields` is sorted by `Definition::subgraph_id`. We rely on it for binary search.
    definitions: Vec<Definition>,

    // Invariant: `fields` is sorted by `Field::object_id`. We rely on it for binary search.
    fields: Vec<Field>,

    // Secondary indexes.

    // We want a set and not a map, because each name corresponds to one _or more_ definitions (in
    // different subgrahs). And a BTreeSet because we need range queries.
    //
    // (definition name, definition id)
    definition_names: BTreeSet<(StringId, DefinitionId)>,

    // We want a set and not a map, because each name corresponds to one _or more_ fields (in
    // different subgrahs). And a BTreeSet because we need range queries.
    //
    // `(definition name, field name, field id)`
    field_names: BTreeSet<(StringId, StringId, FieldId)>,
}

impl Subgraphs {
    /// Add a subgraph to compose.
    pub fn ingest(
        &mut self,
        subgraph_schema: &async_graphql_parser::types::ServiceDocument,
        name: &str,
    ) {
        crate::ingest_subgraph::ingest_subgraph(subgraph_schema, name, self)
    }

    /// Iterate over groups of definitions to compose. The definitions are grouped by name. The
    /// argument is a closure that receives, for each group, the first definition of the group and
    /// an iterator over all the other definitions in the group. The order of iteration is
    /// deterministic but unspecified.
    pub(crate) fn iter_definition_groups<'a>(
        &'a self,
        mut compose_fn: impl FnMut(DefinitionWalker<'a>, &mut dyn Iterator<Item = DefinitionWalker<'a>>),
    ) {
        for (_, group) in &self.definition_names.iter().group_by(|(name, _)| name) {
            let mut group = group
                .into_iter()
                .map(move |(_, definition_id)| self.walk(*definition_id));
            let Some(first_definition_id) = group.next() else {
                continue;
            };
            compose_fn(first_definition_id, &mut group);
        }
    }

    /// Iterate over groups of fields to compose. The definitions are grouped by parent type name
    /// and field name. The argument is a closure that receives, for each group, the first field of
    /// the group and an iterator over all the other fields in the group. The order of iteration is
    /// deterministic but unspecified.
    pub(crate) fn iter_field_groups(
        &self,
        mut compose_fn: impl FnMut(FieldWalker<'_>, &mut dyn Iterator<Item = FieldWalker<'_>>),
    ) {
        for (_, group) in &self
            .field_names
            .iter()
            .group_by(|(parent_name, field_name, _)| (parent_name, field_name))
        {
            let mut group = group
                .into_iter()
                .map(|(_, _, field_id)| self.walk(*field_id));
            let Some(first_field_id) = group.next() else {
                continue;
            };
            compose_fn(first_field_id, &mut group);
        }
    }

    pub(crate) fn push_subgraph(&mut self, name: &str) -> SubgraphId {
        let subgraph = Subgraph {
            name: self.strings.intern(name),
        };
        push_and_return_id(&mut self.subgraphs, subgraph, SubgraphId)
    }

    pub(crate) fn push_definition(
        &mut self,
        subgraph_id: SubgraphId,
        name: &str,
        kind: DefinitionKind,
    ) -> DefinitionId {
        let name = self.strings.intern(name);
        let definition = Definition {
            subgraph_id,
            name,
            kind,
        };
        let id = push_and_return_id(&mut self.definitions, definition, DefinitionId);
        self.definition_names.insert((name, id));
        id
    }

    pub(crate) fn push_field(
        &mut self,
        parent_id: DefinitionId,
        field_name: &str,
        type_name: &str,
    ) -> FieldId {
        let name = self.strings.intern(field_name);
        let field = Field {
            parent_id,
            name,
            type_name: self.strings.intern(type_name),
        };
        let id = push_and_return_id(&mut self.fields, field, FieldId);
        let parent_object_name = self.walk(parent_id).name();
        self.field_names.insert((parent_object_name, name, id));
        id
    }

    pub(crate) fn walk<Id>(&self, id: Id) -> Walker<'_, Id> {
        Walker {
            id,
            subgraphs: self,
        }
    }
}

pub(crate) struct Subgraph {
    /// The name of the subgraph. It is not contained in the GraphQL schema of the subgraph, it only makes sense within a
    /// project.
    name: StringId,
}

pub(crate) struct Definition {
    subgraph_id: SubgraphId,
    name: StringId,
    kind: DefinitionKind,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum DefinitionKind {
    Object,
    Interface,
    // InputObject,
    // Union,
    // CustomScalar,
}

/// A field in an object, interface or input object type.
pub(crate) struct Field {
    parent_id: DefinitionId,
    name: StringId,
    type_name: StringId,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct SubgraphId(usize);

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct DefinitionId(usize);

/// The unique identifier for a field in an object, interface or input object field.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct FieldId(usize);

fn push_and_return_id<T, Id>(elems: &mut Vec<T>, new_elem: T, make_id: fn(usize) -> Id) -> Id {
    let id = make_id(elems.len());
    elems.push(new_elem);
    id
}
