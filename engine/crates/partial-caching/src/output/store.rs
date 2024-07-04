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

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct ValueId(usize);

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct ObjectId(usize);

#[derive(Clone, Debug)]
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

#[derive(Clone, Copy, Debug)]
pub struct IdRange<T: Copy> {
    start: T,
    end: T,
}

impl IdRange<ValueId> {
    pub fn len(&self) -> usize {
        self.end.0.saturating_sub(self.start.0)
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
    pub(super) fn root_object(&self) -> Option<ObjectId> {
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

    /// Looks up the value id of a field in an object
    pub(super) fn field_value_id(&self, object_id: ObjectId, index: FieldIndex) -> ValueId {
        let object = &self.objects[object_id.0];
        assert!((index.0 as usize) < object.fields.len());

        ValueId(object.fields.start.0 + (index.0 as usize))
    }

    /// Looks up the value id of an index in a list, if it exists
    #[allow(dead_code)]
    pub(super) fn index_value_id(&self, value_id: ValueId, index: usize) -> Option<ValueId> {
        let ValueRecord::List(entries) = self.values[value_id.0] else {
            return None;
        };

        if index >= entries.len() {
            return None;
        }

        Some(ValueId(entries.start.0 + index))
    }

    pub(super) fn value(&self, id: ValueId) -> &ValueRecord {
        &self.values[id.0]
    }

    pub fn reader<'a>(&'a self, shapes: &'a OutputShapes) -> Option<Object<'a>> {
        Some(Object {
            id: self.root_object()?,
            shapes,
            store: self,
        })
    }

    pub fn read_object<'a>(&'a self, shapes: &'a OutputShapes, id: ObjectId) -> Object<'a> {
        Object {
            id,
            shapes,
            store: self,
        }
    }

    pub(super) fn concrete_shape_of_object(&self, id: ObjectId) -> ConcreteShapeId {
        self.objects[id.0].shape
    }

    fn read_value<'a>(&'a self, id: ValueId, shapes: &'a OutputShapes) -> Option<Value<'a>> {
        Some(match &self.values[id.0] {
            ValueRecord::Unset => return None,
            ValueRecord::Null => Value::Null,
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
        })
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

impl<'a> List<'a> {
    pub fn get_index(&self, index: usize) -> Option<Value<'a>> {
        if index >= self.ids.len() {
            return None;
        }
        self.store.read_value(ValueId(self.ids.start.0 + index), self.shapes)
    }

    pub fn iter(&self) -> ListIter<'a> {
        ListIter {
            ids: self.ids,
            store: self.store,
            shapes: self.shapes,
        }
    }
}

#[derive(Clone, Copy)]
pub struct ListIter<'a> {
    ids: IdRange<ValueId>,
    store: &'a OutputStore,
    shapes: &'a OutputShapes,
}

impl<'a> Iterator for ListIter<'a> {
    type Item = Value<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(value) = self.store.read_value(self.ids.next()?, self.shapes) {
                return Some(value);
            }
        }
    }
}

#[derive(Clone, Copy)]
pub struct Object<'a> {
    pub id: ObjectId,
    store: &'a OutputStore,
    shapes: &'a OutputShapes,
}

impl<'a> Object<'a> {
    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.record().fields.len()
    }

    pub fn field(&self, name: &str) -> Option<Value<'a>> {
        let record = self.record();
        let shapes = self.shapes;
        let shape = shapes.concrete_object(record.shape);
        let field = shape.field(name)?;

        let value_id = self.store.field_value_id(self.id, field.index());

        self.store.read_value(value_id, self.shapes)
    }

    pub fn fields(&self) -> impl Iterator<Item = (&'a str, Value<'a>)> + 'a {
        let record = self.record();
        let shapes = self.shapes;
        let shape = shapes.concrete_object(record.shape);
        let store = self.store;

        shape
            .response_keys()
            .zip(record.fields)
            .filter_map(move |(key, id)| Some((key, store.read_value(id, shapes)?)))
    }

    pub fn shape(&self) -> ConcreteShape<'a> {
        self.shapes.concrete_object(self.record().shape)
    }

    pub fn shape_id(&self) -> ConcreteShapeId {
        self.record().shape
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
