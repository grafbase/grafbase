use id_newtypes::IdRange;
use schema::{
    AuthorizedDirectiveId, DirectiveSiteId, EntityDefinitionId, ExtensionDirectiveId, FieldDefinitionId,
    RequiresScopesDirectiveId, SubgraphId, TypeDefinitionId,
};

use super::PartitionFieldArgumentId;

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
        argument_ids: IdRange<PartitionFieldArgumentId>,
    },
    AuthorizedDefinition {
        directive_id: AuthorizedDirectiveId,
        definition_id: TypeDefinitionId,
    },
    Executable {
        // sorted
        directives: Vec<operation::ExecutableDirectiveId>,
    },
    Extension {
        directive_id: ExtensionDirectiveId,
        target: QueryModifierTarget,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize)]
pub(crate) enum QueryModifierTarget {
    FieldWithArguments(FieldDefinitionId, IdRange<PartitionFieldArgumentId>, Option<SubgraphId>),
    Site(DirectiveSiteId, Option<SubgraphId>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize)]
pub(crate) enum ResponseModifierRuleTarget {
    Field(FieldDefinitionId, IdRange<PartitionFieldArgumentId>),
    FieldOutput(TypeDefinitionId),
    FieldParentEntity(EntityDefinitionId),
}

impl From<ResponseModifierRuleTarget> for DirectiveSiteId {
    fn from(target: ResponseModifierRuleTarget) -> Self {
        match target {
            ResponseModifierRuleTarget::Field(field_id, _) => field_id.into(),
            ResponseModifierRuleTarget::FieldOutput(output_id) => output_id.into(),
            ResponseModifierRuleTarget::FieldParentEntity(entity_id) => entity_id.into(),
        }
    }
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
    Extension {
        directive_id: ExtensionDirectiveId,
        target: ResponseModifierRuleTarget,
    },
}
