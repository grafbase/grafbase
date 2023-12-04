mod definitions;
mod directives;
mod enums;
mod field_types;
mod fields;
mod keys;
mod strings;
mod unions;
mod walkers;

pub(crate) use self::{
    definitions::{DefinitionId, DefinitionKind, DefinitionWalker},
    directives::*,
    field_types::*,
    fields::*,
    keys::*,
    strings::{StringId, StringWalker},
    walkers::*,
};

use crate::VecExt;
use itertools::Itertools;
use std::collections::{BTreeMap, BTreeSet, HashSet};

/// A set of subgraphs to be composed.
#[derive(Default)]
pub struct Subgraphs {
    pub(super) strings: strings::Strings,
    subgraphs: Vec<Subgraph>,
    definitions: definitions::Definitions,
    directives: directives::Directives,
    enums: enums::Enums,
    fields: fields::Fields,
    field_types: field_types::FieldTypes,
    keys: keys::Keys,
    unions: unions::Unions,

    ingestion_diagnostics: crate::Diagnostics,

    // Secondary indexes.

    // We want a BTreeMap because we need range queries. The name comes first, then the subgraph,
    // because we want to know which definitions have the same name but live in different
    // subgraphs.
    //
    // (definition name, subgraph_id) -> definition id
    definition_names: BTreeMap<(StringId, SubgraphId), DefinitionId>,
}

impl Subgraphs {
    /// Add a subgraph to compose.
    pub fn ingest(&mut self, subgraph_schema: &async_graphql_parser::types::ServiceDocument, name: &str, url: &str) {
        crate::ingest_subgraph::ingest_subgraph(subgraph_schema, name, url, self);
    }

    /// Iterate over groups of definitions to compose. The definitions are grouped by name. The
    /// argument is a closure that receives each group as argument. The order of iteration is
    /// deterministic but unspecified.
    pub(crate) fn iter_definition_groups<'a>(&'a self, mut compose_fn: impl FnMut(&[DefinitionWalker<'a>])) {
        let mut buf = Vec::new();
        for (_, group) in &self.definition_names.iter().group_by(|((name, _), _)| name) {
            buf.clear();
            buf.extend(
                group
                    .into_iter()
                    .map(move |(_, definition_id)| self.walk(*definition_id)),
            );
            compose_fn(&buf);
        }
    }

    pub(crate) fn push_ingestion_diagnostic(&mut self, subgraph: SubgraphId, message: String) {
        self.ingestion_diagnostics
            .push_fatal(format!("[{}]: {message}", self.walk(subgraph).name().as_str()));
    }

    pub(crate) fn push_subgraph(&mut self, name: &str, url: &str) -> SubgraphId {
        let subgraph = Subgraph {
            name: self.strings.intern(name),
            url: self.strings.intern(url),
        };
        SubgraphId(self.subgraphs.push_return_idx(subgraph))
    }

    pub(crate) fn walk<Id>(&self, id: Id) -> Walker<'_, Id> {
        Walker { id, subgraphs: self }
    }

    /// Iterates all builtin scalars _that are in use in at least one subgraph_.
    pub(crate) fn iter_builtin_scalars(&self) -> impl Iterator<Item = StringWalker<'_>> + '_ {
        ["ID", "String", "Boolean", "Int", "Float"]
            .into_iter()
            .filter_map(|name| self.strings.lookup(name))
            .map(|string| self.walk(string))
    }

    pub(crate) fn iter_subgraphs(&self) -> impl Iterator<Item = SubgraphWalker<'_>> {
        (0..self.subgraphs.len()).map(|idx| self.walk(SubgraphId(idx)))
    }

    pub(crate) fn emit_ingestion_diagnostics(&self, diagnostics: &mut crate::Diagnostics) {
        diagnostics.clone_all_from(&self.ingestion_diagnostics);
    }
}

pub(crate) struct Subgraph {
    /// The name of the subgraph. It is not contained in the GraphQL schema of the subgraph, it
    /// only makes sense within a project.
    name: StringId,
    url: StringId,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct SubgraphId(usize);

impl SubgraphId {
    pub(crate) const MIN: SubgraphId = SubgraphId(usize::MIN);
    pub(crate) const MAX: SubgraphId = SubgraphId(usize::MAX);

    pub(crate) fn idx(self) -> usize {
        self.0
    }
}
