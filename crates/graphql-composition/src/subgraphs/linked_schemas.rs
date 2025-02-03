use super::*;

/// Schemas linked with `@link`.
#[derive(Default)]
pub(crate) struct LinkedSchemas {
    pub(super) schemas: Vec<LinkedSchemaRecord>,
    namespaces: HashMap<StringId, LinkedSchemaId>,
    /// Directives that have been `@import`ed, and can be used with their unqualified, maybe aliased name.
    pub(super) definitions: Vec<LinkedDefinitionRecord>,
    imported_definitions_by_name: HashMap<StringId, LinkedDefinitionId>,
    subgraphs_with_federation_v2_link: HashSet<SubgraphId>,
}

/// A schema imported with `@link`.
pub(crate) struct LinkedSchemaRecord {
    /// The subgraph where the schema is @link'ed (imported).
    pub(crate) subgraph_id: SubgraphId,
    /// The url of the schema.
    pub(crate) url: StringId,
    pub(crate) name_from_url: Option<StringId>,
    #[expect(unused)]
    pub(crate) version_from_url: Option<StringId>,
    pub(crate) r#as: Option<StringId>,
}

impl LinkedSchemaRecord {
    /// The prefix for the qualified imports from this schema. This can be None, when the url is not a url, or it doesn't have a path segment representing the name, and no `as:` argument is provided. The definitions linked from that `@link()` are then not addressable.
    pub(crate) fn namespace(&self) -> Option<StringId> {
        self.r#as.or(self.name_from_url)
    }

    pub(crate) fn is_federation_v2(&self, subgraphs: &Subgraphs) -> bool {
        let url = subgraphs.strings.resolve(self.url);
        url.contains("dev/federation/v2")
    }
}

/// A definition from a schema imported with `@link`.
pub(crate) struct LinkedDefinitionRecord {
    pub(crate) linked_schema_id: LinkedSchemaId,
    pub(crate) original_name: StringId,
    pub(crate) imported_as: Option<StringId>,
}

impl LinkedDefinitionRecord {
    pub(crate) fn final_name(&self) -> StringId {
        self.imported_as.unwrap_or(self.original_name)
    }
}

impl Subgraphs {
    pub(crate) fn get_linked_schema(&self, namespace: StringId) -> Option<LinkedSchemaId> {
        self.linked_schemas.namespaces.get(&namespace).copied()
    }

    pub(crate) fn get_imported_definition(&self, name: StringId) -> Option<LinkedDefinitionId> {
        self.linked_schemas.imported_definitions_by_name.get(&name).copied()
    }

    pub(crate) fn push_linked_definition(&mut self, linked_definition: LinkedDefinitionRecord) -> LinkedDefinitionId {
        let id = LinkedDefinitionId::from(self.linked_schemas.definitions.len());

        self.linked_schemas
            .imported_definitions_by_name
            .insert(linked_definition.final_name(), id);

        self.linked_schemas.definitions.push(linked_definition);

        id
    }

    pub(crate) fn push_linked_schema(&mut self, linked_schema: LinkedSchemaRecord) -> LinkedSchemaId {
        let id = LinkedSchemaId::from(self.linked_schemas.schemas.len());

        if linked_schema.is_federation_v2(self) {
            self.linked_schemas
                .subgraphs_with_federation_v2_link
                .insert(linked_schema.subgraph_id);
        }

        if let Some(namespace) = linked_schema.namespace() {
            self.linked_schemas.namespaces.insert(namespace, id);
        }

        self.linked_schemas.schemas.push(linked_schema);

        id
    }

    pub(crate) fn subgraph_links_federation_v2(&self, subgraph_id: SubgraphId) -> bool {
        self.linked_schemas
            .subgraphs_with_federation_v2_link
            .contains(&subgraph_id)
    }
}
