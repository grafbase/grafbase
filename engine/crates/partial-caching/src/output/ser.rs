use serde::{ser::SerializeMap, Serialize};

use super::{shapes::OutputShapes, store::Value, OutputStore};

impl OutputStore {
    pub fn serialize_all<S>(&self, shapes: &OutputShapes, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let Some(object) = self.reader(shapes) else {
            return serializer.serialize_none();
        };

        let mut map = serializer.serialize_map(Some(object.len()))?;
        for (name, reader) in object.fields() {
            map.serialize_entry(name, &ValueSerializer { reader })?;
        }
        map.end()
    }
}

struct ValueSerializer<'a> {
    reader: Value<'a>,
}

impl Serialize for ValueSerializer<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self.reader {
            Value::Null => serializer.serialize_none(),
            Value::Integer(inner) => inner.serialize(serializer),
            Value::Float(inner) => inner.serialize(serializer),
            Value::String(inner) => inner.serialize(serializer),
            Value::Boolean(inner) => inner.serialize(serializer),
            Value::List(items) => serializer.collect_seq(items.map(|reader| ValueSerializer { reader })),
            Value::Object(object) => {
                let mut map = serializer.serialize_map(Some(object.len()))?;
                for (name, reader) in object.fields() {
                    map.serialize_entry(name, &ValueSerializer { reader })?;
                }
                map.end()
            }
            Value::Inline(value) => value.serialize(serializer),
        }
    }
}
