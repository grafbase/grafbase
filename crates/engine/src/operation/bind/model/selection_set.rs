use id_newtypes::IdRange;
use schema::{CompositeTypeId, FieldDefinitionId, InputValueDefinitionId};

use crate::{
    operation::{Location, QueryInputValueId},
    response::SafeResponseKey,
};

use super::{BoundFieldArgumentId, BoundFieldId, BoundSelectionSetId};

#[derive(Clone, Default, serde::Serialize, serde::Deserialize)]
pub(crate) struct BoundSelectionSet {
    /// (ResponseKey, Option<FieldDefinitionId>) is guaranteed to be unique
    /// Ordered by query (parent EntityId, query position)
    pub(crate) field_ids: Vec<BoundFieldId>,
}

#[derive(Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, serde::Serialize, serde::Deserialize, id_derives::Id)]
pub(crate) struct QueryPosition(std::num::NonZero<u16>);

impl QueryPosition {
    pub(crate) const MAX: usize = u16::MAX as usize - 1;
    pub(crate) const EXTRA: usize = u16::MAX as usize;
}

/// The BoundFieldDefinition defines a field that is part of the actual GraphQL query.
/// A BoundField is a field in the query *after* spreading all the named fragments.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub(crate) enum BoundField {
    // Keeping attributes inside the enum to allow Rust to optimize the size of BoundField. We rarely
    // use the variants directly.
    /// __typename field
    TypeName(BoundTypeNameField),
    /// Corresponds to an actual field within the operation that has a field definition
    Query(BoundQueryField),
    /// Extra field added during planning to satisfy resolver/field requirements
    Extra(BoundExtraField),
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub(crate) struct BoundTypeNameField {
    pub type_condition: CompositeTypeId,
    pub query_position: QueryPosition,
    pub key: SafeResponseKey,
    pub location: Location,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub(crate) struct BoundQueryField {
    pub query_position: QueryPosition,
    pub key: SafeResponseKey,
    pub subgraph_key: SafeResponseKey,
    pub location: Location,
    pub definition_id: FieldDefinitionId,
    pub argument_ids: IdRange<BoundFieldArgumentId>,
    pub selection_set_id: Option<BoundSelectionSetId>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub(crate) struct BoundExtraField {
    // Extra fields are added as soon as they might be necessary, and they're assigned a key if
    // they are.
    pub key: Option<SafeResponseKey>,
    pub definition_id: FieldDefinitionId,
    pub argument_ids: IdRange<BoundFieldArgumentId>,
    pub petitioner_location: Location,
}

impl BoundField {
    pub(crate) fn response_key(&self) -> Option<SafeResponseKey> {
        match self {
            BoundField::TypeName(field) => Some(field.key),
            BoundField::Query(field) => Some(field.key),
            BoundField::Extra(field) => field.key,
        }
    }

    pub(crate) fn location(&self) -> Location {
        match self {
            BoundField::TypeName(BoundTypeNameField { location, .. }) => *location,
            BoundField::Query(BoundQueryField { location, .. }) => *location,
            BoundField::Extra(BoundExtraField {
                petitioner_location, ..
            }) => *petitioner_location,
        }
    }

    pub(crate) fn definition_id(&self) -> Option<FieldDefinitionId> {
        match self {
            BoundField::TypeName(BoundTypeNameField { .. }) => None,
            BoundField::Query(BoundQueryField { definition_id, .. }) => Some(*definition_id),
            BoundField::Extra(BoundExtraField { definition_id, .. }) => Some(*definition_id),
        }
    }

    pub(crate) fn argument_ids(&self) -> IdRange<BoundFieldArgumentId> {
        match self {
            BoundField::TypeName(BoundTypeNameField { .. }) => IdRange::empty(),
            BoundField::Query(BoundQueryField { argument_ids, .. }) => *argument_ids,
            BoundField::Extra(BoundExtraField { argument_ids, .. }) => *argument_ids,
        }
    }
}

/// Represents arguments that were specified in the query with a value
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub(crate) struct BoundFieldArgument {
    pub(crate) name_location: Option<Location>,
    pub(crate) value_location: Option<Location>,
    pub(crate) input_value_definition_id: InputValueDefinitionId,
    pub(crate) input_value_id: QueryInputValueId,
}
