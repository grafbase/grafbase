use graph_entities::CompactValue;
use serde_json::Number;

use super::shapes::{ConcreteShape, ConcreteShapeId, FieldIndex, OutputShapes};

// TODO: Docstring etc.
#[derive(Default)]
pub struct OutputStore {
    values: Vec<ValueRecord>,
    objects: Vec<ObjectRecord>,
}

struct ObjectRecord {
    /// The shape of the object.
    shape: ConcreteShapeId,

    fields: IdRange<ValueId>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct ValueId(usize);

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct ObjectId(usize);

#[derive(Clone)]
pub enum ValueRecord {
    Unset,
    Null,
    Number(Number),
    String(Box<str>),
    Boolean(bool),
    List(IdRange<ValueId>),
    Object(ObjectId),

    /// This variant shouldn't be needed _most_ of the time.  But in the presence of
    /// JSON scalars or similar we might get lists and objects for which we don't have any
    /// shape information.  Those go into this branch unchanged.
    InlineValue(Box<CompactValue>),
}

#[derive(Clone, Copy)]
pub struct IdRange<T: Copy> {
    start: T,
    end: T,
}

impl IdRange<ValueId> {
    pub fn len(&self) -> usize {
        self.end.0.saturating_sub(self.start.0)
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl Iterator for IdRange<ValueId> {
    type Item = ValueId;

    fn next(&mut self) -> Option<Self::Item> {
        if self.start == self.end {
            return None;
        }

        let next = self.start;
        self.start = ValueId(self.start.0 + 1);

        Some(next)
    }
}

impl OutputStore {
    fn root_object(&self) -> Option<ObjectId> {
        match self.values.first()? {
            ValueRecord::Object(id) => Some(*id),
            _ => None,
        }
    }

    pub(super) fn insert_object(&mut self, object_shape: ConcreteShape<'_>) -> ObjectId {
        let shape = object_shape.id();
        let object_id = ObjectId(self.objects.len());

        let fields = self.insert_empty_fields(object_shape.field_count());
        self.objects.push(ObjectRecord { shape, fields });

        object_id
    }

    pub(super) fn new_value(&mut self) -> ValueId {
        let id = ValueId(self.values.len());
        self.values.push(ValueRecord::Unset);
        id
    }

    pub(super) fn write_value(&mut self, id: ValueId, data: ValueRecord) {
        self.values[id.0] = data;
    }

    pub(super) fn new_list(&mut self, len: usize) -> IdRange<ValueId> {
        self.insert_empty_fields(len)
    }

    pub(super) fn insert_empty_fields(&mut self, size: usize) -> IdRange<ValueId> {
        let start = ValueId(self.values.len());
        self.values.extend(std::iter::repeat(ValueRecord::Unset).take(size));
        let end = ValueId(self.values.len());

        IdRange { start, end }
    }

    pub(super) fn field_value_id(&self, object_id: ObjectId, index: FieldIndex) -> ValueId {
        let object = &self.objects[object_id.0];
        assert!((index.0 as usize) < object.fields.len());

        ValueId(object.fields.start.0 + (index.0 as usize))
    }

    // TODO: Should this be Value?  Not sure.
    pub(super) fn value(&self, id: ValueId) -> &ValueRecord {
        &self.values[id.0]
    }

    pub(super) fn object(&self, id: ValueId) -> &ValueRecord {
        &self.values[id.0]
    }

    pub(super) fn reader<'a>(&'a self, shapes: &'a OutputShapes) -> Option<Object<'a>> {
        Some(Object {
            id: self.root_object()?,
            shapes,
            store: self,
        })
    }

    fn read_value<'a>(&'a self, id: ValueId, shapes: &'a OutputShapes) -> Value<'a> {
        match &self.values[id.0] {
            ValueRecord::Unset | ValueRecord::Null => Value::Null,
            ValueRecord::Number(number) if number.is_f64() => Value::Float(number.as_f64().unwrap()),
            ValueRecord::Number(number) => Value::Integer(number.as_i64().unwrap()),
            ValueRecord::String(string) => Value::String(string.as_ref()),
            ValueRecord::Boolean(inner) => Value::Boolean(*inner),
            ValueRecord::InlineValue(inner) => Value::Inline(inner.as_ref()),
            ValueRecord::List(ids) => Value::List(List {
                ids: *ids,
                store: self,
                shapes,
            }),
            ValueRecord::Object(id) => Value::Object(Object {
                id: *id,
                store: self,
                shapes,
            }),
        }
    }
}

/// Reader for Values
#[derive(Clone, Copy)]
pub enum Value<'a> {
    Null,
    Float(f64),
    Integer(i64),
    String(&'a str),
    Boolean(bool),
    List(List<'a>),
    Object(Object<'a>),
    Inline(&'a CompactValue),
}

#[derive(Clone, Copy)]
pub struct List<'a> {
    ids: IdRange<ValueId>,
    store: &'a OutputStore,
    shapes: &'a OutputShapes,
}

impl<'a> Iterator for List<'a> {
    type Item = Value<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        Some(self.store.read_value(self.ids.next()?, self.shapes))
    }
}

#[derive(Clone, Copy)]
pub struct Object<'a> {
    id: ObjectId,
    store: &'a OutputStore,
    shapes: &'a OutputShapes,
}

impl<'a> Object<'a> {
    pub fn len(&self) -> usize {
        self.record().fields.len()
    }

    pub fn fields(&self) -> impl Iterator<Item = (&'a str, Value<'a>)> + 'a {
        let record = self.record();
        let shapes = self.shapes;
        let shape = shapes.concrete_object(record.shape);
        let store = self.store;

        shape
            .response_keys()
            .zip(record.fields)
            .map(move |(key, id)| (key, store.read_value(id, shapes)))
    }

    fn record(&self) -> &'a ObjectRecord {
        &self.store.objects[self.id.0]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sizes() {
        // There will be tons of these, so they should be as small as possible
        assert_eq!(std::mem::size_of::<ValueRecord>(), 24);
        assert_eq!(std::mem::size_of::<ObjectRecord>(), 24);
    }
}
