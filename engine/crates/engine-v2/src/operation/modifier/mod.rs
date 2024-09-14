mod query;

use id_newtypes::IdRange;
use schema::{AuthorizedDirectiveId, DefinitionId, FieldDefinitionId, RequiresScopesDirectiveId};

use super::{FieldArgumentId, QueryModifierImpactedFieldId, ResponseModifierImpactedFieldId};

pub(crate) use query::*;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
/// Represents a query modifier that governs how queries are modified based on specific rules.
///
/// This struct contains the rule that determines the behavior of the query modifier and
/// the fields that are impacted by this modifier.
pub(crate) struct QueryModifier {
    /// The rule that governs the modification behavior of the query.
    pub rule: QueryModifierRule,

    /// The fields that are impacted by this query modifier.
    pub impacted_fields: IdRange<QueryModifierImpactedFieldId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub(crate) enum QueryModifierRule {
    /// Represents a rule for authenticated queries.
    Authenticated,

    /// Represents a rule that requires certain scopes for access.
    RequiresScopes(RequiresScopesDirectiveId),

    /// Represents a rule for fields that require authorization.
    AuthorizedField {
        /// The ID of the authorized directive.
        directive_id: AuthorizedDirectiveId,

        /// The ID of the field definition.
        definition_id: FieldDefinitionId,

        /// The collection of argument IDs related to the field.
        argument_ids: IdRange<FieldArgumentId>,
    },

    /// Represents a rule for definitions that require authorization.
    AuthorizedDefinition {
        /// The ID of the authorized directive.
        directive_id: AuthorizedDirectiveId,

        /// The ID of the definition being authorized.
        definition_id: DefinitionId,
    },
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub(crate) struct ResponseModifier {
    /// The rule that governs the response modification behavior.
    ///
    /// This defines how the response should be modified based on specific rules.
    pub rule: ResponseModifierRule,

    /// The fields impacted by the response modifier.
    ///
    /// This collection contains the IDs of the fields that are affected by the response modification rules.
    pub impacted_fields: IdRange<ResponseModifierImpactedFieldId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize)]
pub(crate) enum ResponseModifierRule {
    /// Represents a rule for an authorized parent edge in a query.
    AuthorizedParentEdge {
        /// The ID of the authorized directive.
        ///
        /// This directive determines the authorization rules applicable to the parent edge.
        directive_id: AuthorizedDirectiveId,

        /// The ID of the field definition.
        ///
        /// This specifies which field definition is associated with the authorization.
        definition_id: FieldDefinitionId,
    },

    /// Represents a rule for an authorized edge child in a query.
    AuthorizedEdgeChild {
        /// The ID of the authorized directive.
        ///
        /// This directive determines the authorization rules applicable to the edge child.
        directive_id: AuthorizedDirectiveId,

        /// The ID of the field definition.
        ///
        /// This specifies which field definition is associated with the authorization.
        definition_id: FieldDefinitionId,
    },
}
