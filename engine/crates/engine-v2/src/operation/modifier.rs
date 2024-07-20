use id_newtypes::IdRange;
use schema::{AuthorizedDirectiveId, Definition, EntityId, FieldDefinitionId, RequiredScopesId};

use crate::response::ResponseObjectSetId;

use super::{FieldArgumentId, ImpactedFieldId, ResponseModifierRuleId};

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub(crate) struct QueryModifier {
    pub rule: QueryModifierRule,
    pub impacted_fields: IdRange<ImpactedFieldId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub(crate) enum QueryModifierRule {
    Authenticated,
    RequiresScopes(RequiredScopesId),
    AuthorizedField {
        directive_id: AuthorizedDirectiveId,
        definition_id: FieldDefinitionId,
        argument_ids: IdRange<FieldArgumentId>,
    },
    AuthorizedDefinition {
        directive_id: AuthorizedDirectiveId,
        definition: Definition,
    },
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize)]
pub(crate) struct ResponseModifier {
    pub rule_id: ResponseModifierRuleId,
    pub response_object_set_id: ResponseObjectSetId,
    pub type_condition: EntityId,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub(crate) enum ResponseModifierRule {
    AuthorizedField {
        directive_id: AuthorizedDirectiveId,
        definition_id: FieldDefinitionId,
    },
}
