use id_newtypes::IdRange;
use schema::{
    FieldDefinitionId, InterfaceDefinitionId, ObjectDefinitionId, RequiredFieldId, ScalarType, UnionDefinitionId,
    Wrapping,
};

use crate::operation::FieldId;

use super::{ResponseEdge, ResponseObjectSetId, SafeResponseKey};

#[derive(Default, serde::Serialize, serde::Deserialize)]
pub(crate) struct Shapes {
    pub polymorphic: Vec<PolymorphicObjectShape>,
    pub concrete: Vec<ConcreteObjectShape>,
    pub fields: Vec<FieldShape>,
}

id_newtypes::NonZeroU32! {
    Shapes.concrete[ConcreteObjectShapeId] => ConcreteObjectShape,
    Shapes.polymorphic[PolymorphicObjectShapeId] => PolymorphicObjectShape,
    Shapes.fields[FieldShapeId] => FieldShape,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub(crate) struct FieldShape {
    pub expected_key: SafeResponseKey,
    pub edge: ResponseEdge,
    pub id: FieldId,
    pub required_field_id: Option<RequiredFieldId>,
    pub definition_id: FieldDefinitionId,
    pub shape: Shape,
    pub wrapping: Wrapping,
}

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub(crate) enum Shape {
    Scalar(ScalarType),
    ConcreteObject(ConcreteObjectShapeId),
    PolymorphicObject(PolymorphicObjectShapeId),
}

impl Shape {
    pub(crate) fn as_concrete_object(&self) -> Option<ConcreteObjectShapeId> {
        match self {
            Shape::ConcreteObject(id) => Some(*id),
            _ => None,
        }
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub(crate) struct PolymorphicObjectShape {
    // Sorted by Object typename
    pub possibilities: Vec<(ObjectDefinitionId, ConcreteObjectShapeId)>,
}

/// Being concrete does not mean it's only associated with a single object definition id
/// only that we know exactly which fields must be present for one or multiple of them.
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub(crate) struct ConcreteObjectShape {
    pub set_id: Option<ResponseObjectSetId>,
    pub identifier: ObjectIdentifier,
    pub typename_response_edges: Vec<ResponseEdge>,
    // Sorted by expected_key
    pub field_shape_ids: IdRange<FieldShapeId>,
}

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub enum ObjectIdentifier {
    Known(ObjectDefinitionId),
    UnionTypename(UnionDefinitionId),
    InterfaceTypename(InterfaceDefinitionId),
    Anonymous,
}
