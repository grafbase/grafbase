mod building;
mod fragment_iter;
mod shape_builder;

use std::fmt;

pub use building::build_output_shapes;

use crate::{planning::defers::DeferId, CachingPlan, TypeRelationships};

/// Contains the schemas of all the objects we could see in our output,
/// based on the shape of the query
pub struct OutputShapes {
    objects: Vec<ObjectShapeRecord>,

    type_conditions: Vec<TypeConditionNode>,

    root: ConcreteShapeId,

    /// Defers that are rooted in a given ConcreteShapeId
    ///
    /// There may be multiple defers for a given shape and multiple shapes that
    /// contain a defer.  Fun
    ///
    /// This should be sorted by ConcreteShapeId to allow a binary search
    defer_roots: Vec<(ConcreteShapeId, DeferId)>,
}

impl OutputShapes {
    pub(crate) fn new(plan: &CachingPlan, subtypes: &dyn TypeRelationships) -> Self {
        build_output_shapes(plan, subtypes)
    }

    pub fn root(&self) -> ConcreteShape<'_> {
        ConcreteShape {
            shapes: self,
            id: self.root,
        }
    }

    pub fn concrete_object(&self, id: ConcreteShapeId) -> ConcreteShape<'_> {
        ConcreteShape { shapes: self, id }
    }

    pub fn object(&self, id: ObjectShapeId) -> ObjectShape<'_> {
        match self.objects[id.0 as usize] {
            ObjectShapeRecord::Concrete { .. } => ObjectShape::Concrete(ConcreteShape {
                shapes: self,
                id: ConcreteShapeId(id.0),
            }),
            ObjectShapeRecord::Polymorphic { .. } => ObjectShape::Polymorphic(PolymorphicShape { shapes: self, id }),
        }
    }

    pub fn defers_for_object(&self, target_id: ConcreteShapeId) -> impl ExactSizeIterator<Item = DeferId> + '_ {
        let start_range = self.defer_roots.partition_point(|(shape_id, _)| *shape_id < target_id);
        let end_range =
            start_range + self.defer_roots[start_range..].partition_point(|(shape_id, _)| *shape_id == target_id);

        self.defer_roots[start_range..end_range]
            .iter()
            .map(|(_, defer_id)| *defer_id)
    }
}

/// The shape an object in the response can have
#[derive(Clone, Copy)]
pub enum ObjectShape<'a> {
    /// If a selection set has no type conditions in it then we know up front
    /// all the fields that can be present, and we use this ConcreteShape type
    Concrete(ConcreteShape<'a>),
    /// If a selection set has type conditions in it we enumerate all the
    /// possible shapes in a PolymorphicShape
    Polymorphic(PolymorphicShape<'a>),
}

#[derive(Clone, Copy)]
pub struct FieldIndex(pub(super) u16);

#[derive(Clone, Copy)]
pub struct ObjectShapeId(u16);

#[derive(Clone, Copy, PartialOrd, Ord, PartialEq, Eq, Debug)]
pub struct ConcreteShapeId(u16);

#[derive(Clone, Copy, PartialOrd, Ord, PartialEq, Eq)]
pub struct TypeConditionId(u16);

enum ObjectShapeRecord {
    Concrete {
        fields: Vec<FieldRecord>,
    },
    Polymorphic {
        type_conditions: Box<[TypeConditionId]>,
        fallback: ConcreteShapeId,
    },
}

pub struct FieldRecord {
    response_key: String,
    defer: Option<DeferId>,
    subselection_shape: Option<ObjectShapeId>,
}

pub struct TypeConditionNode {
    type_condition: String,
    concrete_shape: ConcreteShapeId,
    subtypes: Box<[TypeConditionId]>,
}

#[derive(Clone, Copy)]
pub struct ConcreteShape<'a> {
    shapes: &'a OutputShapes,
    pub id: ConcreteShapeId,
}

impl<'a> ConcreteShape<'a> {
    pub fn id(&self) -> ConcreteShapeId {
        self.id
    }

    pub fn field_count(&self) -> usize {
        self.field_records().len()
    }

    pub fn field(&self, name: &str) -> Option<Field<'a>> {
        // This might not be very efficient if there's a lot of fields, but can optimise later
        let (index, _) = self
            .field_records()
            .iter()
            .enumerate()
            .find(|(_, field)| field.response_key == name)?;

        Some(Field {
            shapes: self.shapes,
            object_id: self.id,
            field_index: FieldIndex(index as u16),
        })
    }

    pub fn response_keys(&self) -> impl Iterator<Item = &'a str> + 'a {
        self.field_records().iter().map(|field| field.response_key.as_str())
    }

    fn field_records(&self) -> &'a [FieldRecord] {
        let ObjectShapeRecord::Concrete { fields } = &self.shapes.objects[self.id.0 as usize] else {
            unreachable!()
        };
        fields
    }
}

#[derive(Clone, Copy)]
pub struct PolymorphicShape<'a> {
    shapes: &'a OutputShapes,
    id: ObjectShapeId,
}

impl<'a> PolymorphicShape<'a> {
    pub(crate) fn concrete_shape_for_typename(
        &self,
        typename: &str,
        type_relationships: &dyn TypeRelationships,
    ) -> ConcreteShape<'a> {
        let ObjectShapeRecord::Polymorphic {
            type_conditions,
            fallback,
        } = &self.shapes.objects[self.id.0 as usize]
        else {
            unreachable!()
        };

        fn check_conditions(
            typename: &str,
            conditions: &[TypeConditionId],
            fallback: ConcreteShapeId,
            shapes: &OutputShapes,
            type_relationships: &dyn TypeRelationships,
        ) -> ConcreteShapeId {
            let condition_match = conditions.iter().find(|id| {
                type_relationships
                    .type_condition_matches(shapes.type_conditions[id.0 as usize].type_condition.as_str(), typename)
            });

            match condition_match {
                Some(id) => {
                    let node = &shapes.type_conditions[id.0 as usize];
                    check_conditions(
                        typename,
                        &node.subtypes,
                        node.concrete_shape,
                        shapes,
                        type_relationships,
                    )
                }
                None => fallback,
            }
        }

        let id = check_conditions(typename, type_conditions, *fallback, self.shapes, type_relationships);

        ConcreteShape {
            shapes: self.shapes,
            id,
        }
    }
}

#[derive(Clone, Copy)]
pub struct Field<'a> {
    shapes: &'a OutputShapes,
    object_id: ConcreteShapeId,
    field_index: FieldIndex,
}

impl fmt::Debug for Field<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Field")
            .field("response_key", &self.response_key())
            .finish_non_exhaustive()
    }
}

impl<'a> Field<'a> {
    pub fn response_key(&self) -> &'a str {
        &self.record().response_key
    }

    pub fn index(&self) -> FieldIndex {
        self.field_index
    }

    pub fn is_leaf(&self) -> bool {
        self.record().subselection_shape.is_none()
    }

    /// If this field and its subselections are unique to a particulary defer
    /// this will be set.
    pub fn defer_id(&self) -> Option<DeferId> {
        self.record().defer
    }

    pub fn subselection_shape(&self) -> Option<ObjectShape<'a>> {
        Some(self.shapes.object(self.record().subselection_shape?))
    }

    fn record(&self) -> &'a FieldRecord {
        let ObjectShapeRecord::Concrete { fields } = &self.shapes.objects[self.object_id.0 as usize] else {
            unreachable!()
        };
        &fields[self.field_index.0 as usize]
    }
}
