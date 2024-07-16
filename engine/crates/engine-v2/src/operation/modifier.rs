use id_newtypes::IdRange;
use schema::{AuthorizedDirectiveId, Definition, FieldDefinitionId, RequiredScopesId};

use super::{FieldArgumentId, ImpactedFieldId};

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub(crate) struct QueryModifier {
    pub condition: QueryModifierCondition,
    pub impacted_fields: IdRange<ImpactedFieldId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub(crate) enum QueryModifierCondition {
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
