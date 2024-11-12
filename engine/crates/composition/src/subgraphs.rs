mod definitions;
mod directives;
mod enums;
mod field_types;
mod fields;
mod keys;
mod strings;
mod top;
mod unions;
mod walker;

pub(crate) use self::{
    definitions::{DefinitionId, DefinitionKind, DefinitionWalker},
    directives::*,
    field_types::*,
    fields::*,
    keys::*,
    strings::{StringId, StringWalker},
    top::*,
    walker::Walker,
};

use crate::VecExt;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

/// A set of subgraphs to be composed.
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

impl Default for Subgraphs {
    fn default() -> Self {
        let mut strings = strings::Strings::default();
        BUILTIN_SCALARS.into_iter().for_each(|scalar| {
            strings.intern(scalar);
        });

        Self {
            strings,
            subgraphs: Default::default(),
            definitions: Default::default(),
            directives: Default::default(),
            enums: Default::default(),
            fields: Default::default(),
            field_types: Default::default(),
            keys: Default::default(),
            unions: Default::default(),
            ingestion_diagnostics: Default::default(),
            definition_names: Default::default(),
        }
    }
}

const BUILTIN_SCALARS: [&str; 5] = ["ID", "String", "Boolean", "Int", "Float"];

/// returned when a subgraph cannot be ingested
#[derive(Debug)]
pub struct IngestError {
    error: cynic_parser::Error,
    report: String,
}

impl std::error::Error for IngestError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.error.source()
    }
}

impl std::fmt::Display for IngestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if f.alternate() {
            return self.report.fmt(f);
        }
        std::fmt::Display::fmt(&self.error, f)
    }
}

impl Subgraphs {
    /// Add a subgraph to compose.
    pub fn ingest(&mut self, subgraph_schema: &cynic_parser::TypeSystemDocument, name: &str, url: &str) {
        crate::ingest_subgraph::ingest_subgraph(subgraph_schema, name, url, self);
    }

    /// Add a subgraph to compose.
    pub fn ingest_str(&mut self, subgraph_schema: &str, name: &str, url: &str) -> Result<(), IngestError> {
        let subgraph_schema =
            cynic_parser::parse_type_system_document(subgraph_schema).map_err(|error| IngestError {
                report: error.to_report(subgraph_schema).to_string(),
                error,
            })?;
        crate::ingest_subgraph::ingest_subgraph(&subgraph_schema, name, url, self);
        Ok(())
    }

    /// Iterate over groups of definitions to compose. The definitions are grouped by name. The
    /// argument is a closure that receives each group as argument. The order of iteration is
    /// deterministic but unspecified.
    pub(crate) fn iter_definition_groups<'a>(&'a self, mut compose_fn: impl FnMut(&[DefinitionWalker<'a>])) {
        let mut key = None;
        let mut buf = Vec::new();

        for ((name, subgraph), definition) in &self.definition_names {
            if Some(name) != key {
                // New key. Compose previous key and start new group.
                compose_fn(&buf);
                buf.clear();
                key = Some(name);
            }

            // Fill buf, except if we are dealing with a root object type.

            if self.is_root_type(*subgraph, *definition) {
                continue; // handled separately
            }

            buf.push(self.walk(*definition));
        }

        compose_fn(&buf)
    }

    pub(crate) fn push_ingestion_diagnostic(&mut self, subgraph: SubgraphId, message: String) {
        self.ingestion_diagnostics
            .push_fatal(format!("[{}]: {message}", self.walk_subgraph(subgraph).name().as_str()));
    }

    pub(crate) fn walk<Id>(&self, id: Id) -> Walker<'_, Id> {
        Walker { id, subgraphs: self }
    }

    /// Iterates all builtin scalars _that are in use in at least one subgraph_.
    pub(crate) fn iter_builtin_scalars(&self) -> impl Iterator<Item = StringWalker<'_>> + '_ {
        BUILTIN_SCALARS
            .into_iter()
            .map(|name| self.strings.lookup(name).expect("all built in scalars to be interned"))
            .map(|string| self.walk(string))
    }

    pub(crate) fn emit_ingestion_diagnostics(&self, diagnostics: &mut crate::Diagnostics) {
        diagnostics.clone_all_from(&self.ingestion_diagnostics);
    }
}
