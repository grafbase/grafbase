use schema::{AuthorizedDirectiveId, DefinitionId, FieldDefinitionId, RequiresScopesDirectiveId};

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, serde::Serialize, serde::Deserialize)]
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
