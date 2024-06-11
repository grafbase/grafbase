mod building;
mod fragment_iter;

#[cfg(test)]
pub use building::build_output_shapes;

/// Contains the schemas of all the objects we could see in our output,
/// based on the shape of the query
pub struct OutputShapes {
    objects: Vec<ObjectShapeRecord>,

    // The root object of each of the query partitions.
    cache_partition_roots: Vec<ConcreteShapeId>,
    nocache_partition_root: ConcreteShapeId,
}

impl OutputShapes {
    pub fn nocache_shape(&self) -> PartitionShape<'_> {
        PartitionShape {
            object_id: self.nocache_partition_root,
            shapes: self,
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
}

/// PartitionShape is the root of a partitions output shape heirarchy.
pub struct PartitionShape<'a> {
    object_id: ConcreteShapeId,
    shapes: &'a OutputShapes,
}

impl<'a> PartitionShape<'a> {
    pub fn root_object(&self) -> ConcreteShape<'a> {
        ConcreteShape {
            shapes: self.shapes,
            id: self.object_id,
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
            ObjectShape::Concrete(shape) => ObjectShapeId(shape.id.0),
            ObjectShape::Polymorphic(shape) => shape.id,
        }
    }
}

#[derive(Clone, Copy)]
pub struct FieldIndex(pub(super) u16);

#[derive(Clone, Copy)]
pub struct ObjectShapeId(u16);

#[derive(Clone, Copy)]
pub struct ConcreteShapeId(u16);

enum ObjectShapeRecord {
    Concrete {
        fields: Vec<FieldRecord>,
    },
    Polymorphic {
        types: Vec<(Option<String>, ObjectShapeId)>,
    },
}

pub struct FieldRecord {
    response_key: String,
    defer_label: Option<String>,
    subselection_shape: Option<ObjectShapeId>,
}

#[derive(Clone, Copy)]
pub struct ConcreteShape<'a> {
    shapes: &'a OutputShapes,
    id: ConcreteShapeId,
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

#[derive(Clone, Copy)]
pub struct Field<'a> {
    shapes: &'a OutputShapes,
    object_id: ConcreteShapeId,
    field_index: FieldIndex,
}

impl<'a> Field<'a> {
    pub fn index(&self) -> FieldIndex {
        self.field_index
    }

    pub fn is_leaf(&self) -> bool {
        self.record().subselection_shape.is_none()
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
