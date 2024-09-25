mod query;

use id_newtypes::IdRange;
use schema::{AuthorizedDirectiveId, DefinitionId, FieldDefinitionId, RequiresScopesDirectiveId};

use super::{FieldArgumentId, QueryInputValueId, QueryModifierImpactedFieldId, ResponseModifierImpactedFieldId};

pub(crate) use query::*;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub(crate) struct QueryModifier {
    pub rule: QueryModifierRule,
    pub impacted_fields: IdRange<QueryModifierImpactedFieldId>,
}

#[derive(Copy, Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub(crate) enum FieldSkippingDirective {
    Skip,
    Include,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub(crate) enum QueryModifierRule {
    Authenticated,
    RequiresScopes(RequiresScopesDirectiveId),
    AuthorizedField {
        directive_id: AuthorizedDirectiveId,
        definition_id: FieldDefinitionId,
        argument_ids: IdRange<FieldArgumentId>,
    },
    AuthorizedDefinition {
        directive_id: AuthorizedDirectiveId,
        definition_id: DefinitionId,
    },
    Skip {
        input_value_id: QueryInputValueId,
        r#type: FieldSkippingDirective,
    },
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub(crate) struct ResponseModifier {
    pub rule: ResponseModifierRule,
    pub impacted_fields: IdRange<ResponseModifierImpactedFieldId>,
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
