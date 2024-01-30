use schema::{DataType, FieldId, Wrapping};

use crate::{
    request::{BoundFieldId, FlatTypeCondition, SelectionSetType},
    response::{ResponseEdge, ResponseKey},
    utils::IdRange,
};

use super::{ConcreteFieldId, ConcreteSelectionSetId, ConditionalFieldId, ConditionalSelectionSetId, PlanBoundaryId};

#[derive(Debug)]
pub enum CollectedSelectionSet {
    Concrete(ConcreteSelectionSetId),
    Conditional(ConditionalSelectionSetId),
    MergedConditionals {
        ty: SelectionSetType,
        selection_set_ids: Vec<ConditionalSelectionSetId>,
    },
    RuntimeConcrete(Box<RuntimeConcreteSelectionSet>),
}

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
    pub expected_key: ResponseKey,
    pub bound_field_id: BoundFieldId,
    pub schema_field_id: FieldId,
    pub ty: FieldType<ConditionalSelectionSetId>,
}

#[derive(Debug, Clone)]
pub enum FieldType<SelectionSet = CollectedSelectionSet> {
    Scalar(DataType),
    SelectionSet(SelectionSet),
}

#[derive(Debug)]
pub struct ConcreteSelectionSet {
    pub ty: SelectionSetType,
    pub maybe_boundary_id: Option<PlanBoundaryId>,
    // sorted by expected key
    pub fields: IdRange<ConcreteFieldId>,
    pub typename_fields: Vec<ResponseEdge>,
}

#[derive(Debug)]
pub struct ConcreteField {
    pub edge: ResponseEdge,
    pub expected_key: ResponseKey,
    pub bound_field_id: BoundFieldId,
    pub schema_field_id: FieldId,
    pub ty: FieldType,
    pub wrapping: Wrapping,
}

#[derive(Debug)]
pub struct RuntimeConcreteSelectionSet {
    pub ty: SelectionSetType,
    pub boundary_ids: Vec<PlanBoundaryId>,
    // sorted by expected key
    pub fields: Vec<ConcreteField>,
    pub typename_fields: Vec<ResponseEdge>,
}
