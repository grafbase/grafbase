use id_newtypes::IdRange;
use schema::{AuthorizedDirectiveId, DefinitionId, FieldDefinitionId, RequiresScopesDirectiveId};

use crate::operation::{
    BoundFieldArgumentId, BoundQueryModifierImpactedFieldId, BoundResponseModifierImpactedFieldId, QueryInputValueId,
};

// pub(crate) use query::*;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub(crate) struct BoundQueryModifier {
    pub rule: QueryModifierRule,
    pub impacted_fields: IdRange<BoundQueryModifierImpactedFieldId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, serde::Serialize, serde::Deserialize)]
pub(crate) enum QueryModifierRule {
    Authenticated,
    RequiresScopes(RequiresScopesDirectiveId),
    AuthorizedField {
        directive_id: AuthorizedDirectiveId,
        definition_id: FieldDefinitionId,
        argument_ids: IdRange<BoundFieldArgumentId>,
    },
    AuthorizedDefinition {
        directive_id: AuthorizedDirectiveId,
        definition_id: DefinitionId,
    },
    SkipInclude {
        // sorted
        directives: Vec<SkipIncludeDirective>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize)]
pub enum SkipIncludeDirective {
    SkipIf(QueryInputValueId),
    IncludeIf(QueryInputValueId),
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub(crate) struct BoundResponseModifier {
    pub rule: ResponseModifierRule,
    pub impacted_fields: IdRange<BoundResponseModifierImpactedFieldId>,
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
