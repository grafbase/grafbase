use schema::{DataType, FieldId, Wrapping};

use crate::{
    request::{BoundFieldId, FlatTypeCondition, SelectionSetType},
    response::{ResponseEdge, ResponseKey},
    utils::IdRange,
};

use super::{CollectedFieldId, CollectedSelectionSetId, ConditionalFieldId, ConditionalSelectionSetId, PlanBoundaryId};

#[derive(Debug)]
pub enum AnyCollectedSelectionSet {
    /// Generated during planning
    Collected(CollectedSelectionSetId),
    Conditional(ConditionalSelectionSetId),
    /// Generated at runtime from conditional selection sets
    RuntimeMergedConditionals {
        ty: SelectionSetType,
        selection_set_ids: Vec<ConditionalSelectionSetId>,
    },
    RuntimeCollected(Box<RuntimeCollectedSelectionSet>),
}

/// Selection set that could not be entirely collected because of type conditions, we need to know
/// the actual type before collecting the fields.
#[derive(Debug, Clone)]
pub struct ConditionalSelectionSet {
    // needed to know where to look for __typename
    pub ty: SelectionSetType,
    pub maybe_boundary_id: Option<PlanBoundaryId>,
    pub fields: IdRange<ConditionalFieldId>,
    pub typename_fields: Vec<(Option<FlatTypeCondition>, ResponseEdge)>,
}

#[derive(Debug)]
pub struct ConditionalField {
    pub edge: ResponseEdge,
    pub type_condition: Option<FlatTypeCondition>,
    /// Expected key from the upstream response when deserializing
    pub expected_key: ResponseKey,
    pub bound_field_id: BoundFieldId,
    pub schema_field_id: FieldId,
    /// a conditional field cannot have anything than a conditional selection set if any
    /// as it may be merged with other subselection at runtime.
    pub ty: FieldType<ConditionalSelectionSetId>,
}

#[derive(Debug, Clone)]
pub enum FieldType<SelectionSet = AnyCollectedSelectionSet> {
    Scalar(DataType),
    SelectionSet(SelectionSet),
}

/// Selection that could be properly collected, we know exactly which fields are present and what
/// they correspond to.
#[derive(Debug)]
pub struct CollectedSelectionSet {
    pub ty: SelectionSetType,
    pub maybe_boundary_id: Option<PlanBoundaryId>,
    // sorted by expected key
    pub fields: IdRange<CollectedFieldId>,
    pub typename_fields: Vec<ResponseEdge>,
}

#[derive(Debug)]
pub struct CollectedField {
    pub edge: ResponseEdge,
    /// Expected key from the upstream response when deserializing
    pub expected_key: ResponseKey,
    pub bound_field_id: BoundFieldId,
    pub schema_field_id: FieldId,
    pub ty: FieldType,
    pub wrapping: Wrapping,
}

#[derive(Debug)]
pub struct RuntimeCollectedSelectionSet {
    pub ty: SelectionSetType,
    pub boundary_ids: Vec<PlanBoundaryId>,
    // sorted by expected key
    pub fields: Vec<CollectedField>,
    pub typename_fields: Vec<ResponseEdge>,
}
