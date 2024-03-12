use id_newtypes::IdRange;
use schema::{FieldId, ObjectId, ScalarType, Wrapping};

use crate::{
    request::{BoundFieldId, SelectionSetType},
    response::{ResponseEdge, SafeResponseKey},
};

use super::{
    CollectedFieldId, CollectedSelectionSetId, ConditionalFieldId, ConditionalSelectionSetId, FlatTypeCondition,
    PlanBoundaryId,
};

// TODO: The two AnyCollectedSelectionSet aren't great, need to split better the ones which are computed
// during planning and the others.
#[derive(Debug, Clone, Copy)]
pub enum AnyCollectedSelectionSetId {
    Collected(CollectedSelectionSetId),
    Conditional(ConditionalSelectionSetId),
}

#[derive(Debug)]
pub enum AnyCollectedSelectionSet {
    /// Generated during planning
    Collected(CollectedSelectionSetId),
    Conditional(ConditionalSelectionSetId),
    /// Generated at runtime from conditional selection sets
    RuntimeMergedConditionals(RuntimeMergedConditionals),
    RuntimeCollected(Box<RuntimeCollectedSelectionSet>),
}

/// Selection set that could not be entirely collected because of type conditions, we need to know
/// the actual type before collecting the fields.
#[derive(Debug, Clone)]
pub struct ConditionalSelectionSet {
    /// During the traversing of the subgraph response, we'll need to determine the actual
    /// object type to collect fields at runtime. For a subgraph that talks GraphQL it's obviously
    /// just '__typename'. But it doesn't need to. Supposing this is an interface, we'll use the [[schema::Names::interface_discriminant_key]] to know which key we should search for the discriminant and use the associated [[schema::Names::concrete_object_id_from_interface_discriminant]] to determine the actual object id at runtime. The same applies for unions.
    pub ty: SelectionSetType,
    // Plan boundary associated with this selection set. If present we need to push the a
    // ResponseObjectBoundaryItem into the ResponsePart everytime for children plans.
    pub maybe_boundary_id: Option<PlanBoundaryId>,
    pub field_ids: IdRange<ConditionalFieldId>,
    // Selection sets can have multiple __typename fields and eventually type conditions.
    // {
    //     ... on Dog {
    //         __typename
    //     }
    //     myalias: __typename
    //     __typename
    // }
    pub typename_fields: Vec<(Option<FlatTypeCondition>, ResponseEdge)>,
}

#[derive(Debug)]
pub struct ConditionalField {
    pub edge: ResponseEdge,
    pub type_condition: Option<FlatTypeCondition>,
    /// Expected key from the upstream response when deserializing
    pub expected_key: SafeResponseKey,
    pub bound_field_id: BoundFieldId,
    pub schema_field_id: FieldId,
    /// a conditional field cannot have anything than a conditional selection set if any
    /// as it may be merged with other subselection at runtime.
    pub ty: FieldType<ConditionalSelectionSetId>,
}

#[derive(Debug, Clone)]
pub enum FieldType<SelectionSet = AnyCollectedSelectionSet> {
    Scalar(ScalarType),
    SelectionSet(SelectionSet),
}

/// Selection that could be properly collected, we know exactly which fields are present and what
/// they correspond to.
#[derive(Debug)]
pub struct CollectedSelectionSet {
    pub ty: SelectionSetType,
    // Plan boundary associated with this selection set. If present we need to push the a
    // ResponseObjectBoundaryItem into the ResponsePart everytime for children plans.
    pub maybe_boundary_id: Option<PlanBoundaryId>,
    // the fields we point to are sorted by their expected_key
    pub field_ids: IdRange<CollectedFieldId>,
    // Selection sets can have multiple __typename fields.
    // {
    //     myalias: __typename
    //     __typename
    // }
    pub typename_fields: Vec<ResponseEdge>,
}

#[derive(Debug)]
pub struct CollectedField {
    pub edge: ResponseEdge,
    /// Expected key from the upstream response when deserializing
    pub expected_key: SafeResponseKey,
    pub bound_field_id: BoundFieldId,
    pub schema_field_id: FieldId,
    pub ty: FieldType,
    pub wrapping: Wrapping,
}

#[derive(Debug)]
pub struct RuntimeCollectedSelectionSet {
    pub object_id: ObjectId,
    pub boundary_ids: Vec<PlanBoundaryId>,
    // sorted by expected key
    pub fields: Vec<CollectedField>,
    pub typename_fields: Vec<ResponseEdge>,
}

#[derive(Debug)]
pub struct RuntimeMergedConditionals {
    pub ty: SelectionSetType,
    pub selection_set_ids: Vec<ConditionalSelectionSetId>,
}
