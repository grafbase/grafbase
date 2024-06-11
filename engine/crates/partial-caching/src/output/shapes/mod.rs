mod building;
mod fragment_iter;

/// Contains the schemas of all the objects we could see in our output,
/// based on the shape of the query
pub struct OutputShapes {
    objects: Vec<ObjectShapeRecord>,

    // The root object of each of the query partitions.
    cache_partition_roots: Vec<ObjectShapeId>,
    nocache_partition_root: ObjectShapeId,
}

/// PartitionShape is the root of a partitions output shape heirarchy.
pub struct PartitionShape<'a> {
    object_id: ObjectShapeId,
    plans: &'a OutputShapes,
}

impl<'a> PartitionShape<'a> {
    pub fn root_object(&self) -> ObjectShape<'a> {
        match self.plans.objects[self.object_id.0 as usize] {
            ObjectShapeRecord::Concrete { .. } => ObjectShape::Concrete(ConcreteShape {
                plans: self.plans,
                id: self.object_id,
            }),
            ObjectShapeRecord::Polymorphic { .. } => ObjectShape::Polymorphic(PolymorphicShape {
                plans: self.plans,
                id: self.object_id,
            }),
        }
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

impl<'a> ObjectShape<'a> {
    pub fn id(&self) -> ObjectShapeId {
        match self {
            ObjectShape::Concrete(plan) => plan.id,
            ObjectShape::Polymorphic(plan) => plan.id,
        }
    }
}

#[derive(Clone, Copy)]
pub struct Field<'a> {
    plans: &'a OutputShapes,
    id: ObjectShapeId,
    field_index: FieldIndex,
}

#[derive(Clone, Copy)]
pub struct FieldIndex(pub(super) u16);

#[derive(Clone, Copy)]
pub struct ObjectShapeId(u16);

enum ObjectShapeRecord {
    Concrete {
        fields: Vec<FieldRecord>,
    },
    Polymorphic {
        types: Vec<(Option<String>, Vec<FieldRecord>)>,
    },
}

pub struct FieldRecord {
    response_key: String,
    defer_label: Option<String>,
    child_object: Option<ObjectShapeId>,
}

#[derive(Clone, Copy)]
pub struct ConcreteShape<'a> {
    plans: &'a OutputShapes,
    id: ObjectShapeId,
}

#[derive(Clone, Copy)]
pub struct PolymorphicShape<'a> {
    plans: &'a OutputShapes,
    id: ObjectShapeId,
}
