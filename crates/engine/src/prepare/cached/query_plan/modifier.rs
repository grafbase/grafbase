use schema::{AuthorizedDirectiveId, DefinitionId, ExtensionDirectiveId, FieldDefinitionId, RequiresScopesDirectiveId};

#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub(crate) enum QueryModifierRule {
    Authenticated,
    RequiresScopes(RequiresScopesDirectiveId),
    AuthorizedField {
        directive_id: AuthorizedDirectiveId,
        definition_id: FieldDefinitionId,
    },
    AuthorizedFieldWithArguments {
        directive_id: AuthorizedDirectiveId,
        definition_id: FieldDefinitionId,
        argument_ids: query_solver::QueryOrSchemaFieldArgumentIds,
    },
    AuthorizedDefinition {
        directive_id: AuthorizedDirectiveId,
        definition_id: DefinitionId,
    },
    Executable {
        // sorted
        directives: Vec<operation::ExecutableDirectiveId>,
    },
    Extension {
        directive_id: ExtensionDirectiveId,
        target: ModifierTarget,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub(crate) enum ModifierTarget {
    Field(FieldDefinitionId),
    FieldWithArguments(FieldDefinitionId, query_solver::QueryOrSchemaFieldArgumentIds),
    Definition(DefinitionId),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize)]
pub(crate) enum ResponseModifierRule {
    AuthorizedParentEdge {
        directive_id: AuthorizedDirectiveId,
        definition_id: FieldDefinitionId,
    },
    AuthorizedEdgeChild {
        directive_id: AuthorizedDirectiveId,
        definition_id: FieldDefinitionId,
    },
}
