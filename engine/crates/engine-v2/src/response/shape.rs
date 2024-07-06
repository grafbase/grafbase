use id_newtypes::IdRange;
use schema::{FieldDefinitionId, InterfaceId, ObjectId, ScalarType, UnionId, Wrapping};

use crate::operation::FieldId;

use super::{GraphqlError, ResponseEdge, ResponseObjectSetId, SafeResponseKey};

#[derive(Default)]
pub(crate) struct Shapes {
    pub polymorphic: Vec<PolymorphicObjectShape>,
    pub concrete: Vec<ConcreteObjectShape>,
    pub fields: Vec<FieldShape>,
    pub errors: Vec<FieldError>,
}

id_newtypes::NonZeroU16! {
    Shapes.concrete[ConcreteObjectShapeId] => ConcreteObjectShape,
    Shapes.polymorphic[PolymorphicObjectShapeId] => PolymorphicObjectShape,
    Shapes.fields[FieldShapeId] => FieldShape,
    Shapes.errors[FieldErrorId] => FieldError,
}

#[derive(Debug)]
pub(crate) struct FieldShape {
    pub expected_key: SafeResponseKey,
    pub edge: ResponseEdge,
    pub id: FieldId,
    pub definition_id: FieldDefinitionId,
    pub shape: Shape,
    pub wrapping: Wrapping,
}

#[derive(Debug, Clone, Copy)]
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

pub(crate) struct PolymorphicObjectShape {
    // Sorted by Object typename
    pub shapes: Vec<(ObjectId, ConcreteObjectShapeId)>,
}

/// Being concrete does not mean it's only associated with a single object definition id
/// only that we know exactly which fields must be present for one or multiple of them.
#[derive(Debug)]
pub(crate) struct ConcreteObjectShape {
    pub set_id: Option<ResponseObjectSetId>,
    pub identifier: ObjectIdentifier,
    pub typename_response_edges: Vec<ResponseEdge>,
    // Sorted by expected_key
    pub field_shape_ids: IdRange<FieldShapeId>,
    pub field_error_ids: IdRange<FieldErrorId>,
}

#[derive(Debug, Clone)]
pub(crate) struct FieldError {
    pub edge: ResponseEdge,
    pub errors: Vec<GraphqlError>,
    pub is_required: bool,
}

#[derive(Debug, Clone, Copy)]
pub enum ObjectIdentifier {
    Known(ObjectId),
    UnionTypename(UnionId),
    InterfaceTypename(InterfaceId),
    Anonymous,
}
