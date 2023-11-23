use schema::{InterfaceId, ObjectId, UnionId};

use super::{BoundFieldDefinitionId, BoundFieldId, BoundResponseKey, SelectionSetRoot, TypeCondition};

pub enum FlatSlectionSet {
    Concrete(ConcreteFlatSelectionSet),
    Abstract(AbstractFlatSelectionSet),
}

pub struct ConcreteFlatSelectionSet {
    pub object_id: ObjectId,
    pub fields: Vec<ConcreteField>,
}

pub struct ConcreteField {
    pub bound_response_key: BoundResponseKey,
    pub definition_id: BoundFieldDefinitionId,
    pub bound_field_ids: Vec<BoundFieldId>, // for attribution
    pub selection_set: Option<FlatSlectionSet>,
}

pub struct AbstractFlatSelectionSet {
    pub root: SelectionSetRoot,
    pub items: Vec<AbstractSelection>,
}

pub enum AbstractSelection {
    Field(AbstractFieldSelection),
    Conditional(AbstractConditionalSelection),
}

pub struct AbstractFieldSelection {
    pub id: BoundFieldId,
}

pub struct AbstractConditionalSelection {
    pub type_condition: FlatTypeCondition,
    pub id: BoundFieldId,
}

pub enum FlatTypeCondition {
    Object(ObjectId),
    Interface(InterfaceId),
    Union(UnionId),
    Chain(Vec<TypeCondition>),
    // Used when a type condition chain could be collapsed
    Objects(Vec<ObjectId>),
}
