use std::iter;

use graph_entities::{CompactValue, QueryResponse, ResponseContainer, ResponseList, ResponseNodeId};
use internment::ArcIntern;
use serde_json::Number;

use super::{
    store::{Object, Value},
    OutputStore,
};

impl OutputStore {
    #[cfg(test)] // This might be used for real at some point, but for now it's just needed for tests
    pub fn serialize_all<S>(&self, shapes: &super::shapes::OutputShapes, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeMap;

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

#[cfg(test)]
struct ValueSerializer<'a> {
    reader: Value<'a>,
}

#[cfg(test)]
impl serde::Serialize for ValueSerializer<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeMap;

        match self.reader {
            Value::Null => serializer.serialize_none(),
            Value::Integer(inner) => inner.serialize(serializer),
            Value::Float(inner) => inner.serialize(serializer),
            Value::String(inner) => inner.serialize(serializer),
            Value::Boolean(inner) => inner.serialize(serializer),
            Value::List(items) => serializer.collect_seq(items.iter().map(|reader| ValueSerializer { reader })),
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

impl Object<'_> {
    /// Converts an object into a QueryResponse.
    ///
    /// This isn't especially efficient, but it's the easiest thing to do at the moment.
    /// GB-9672 will (hopefully) handle doing something more efficient.
    pub fn into_query_response(self, include_synthetic_typenames: bool) -> QueryResponse {
        let mut response = QueryResponse::default();
        let root = write_value(Value::Object(self), &mut response, include_synthetic_typenames);
        response.set_root_unchecked(root);
        response
    }

    pub fn into_compact_value(self, include_synthetic_typenames: bool) -> CompactValue {
        let mut response = QueryResponse::default();
        let root = write_value(Value::Object(self), &mut response, include_synthetic_typenames);
        response.take_node_into_compact_value(root).expect("node to exist")
    }
}

fn write_value(value: Value<'_>, response: &mut QueryResponse, include_synthetic_typenames: bool) -> ResponseNodeId {
    match value {
        Value::Null => response.insert_node(CompactValue::Null),
        Value::Float(float) => response.insert_node(CompactValue::Number(Number::from_f64(float).unwrap())),
        Value::Integer(integer) => response.insert_node(CompactValue::Number(integer.into())),
        Value::String(s) => response.insert_node(CompactValue::String(s.into())),
        Value::Boolean(inner) => response.insert_node(CompactValue::Boolean(inner)),
        Value::List(list) => {
            let list = ResponseList::with_children(
                list.iter()
                    .map(|value| write_value(value, response, include_synthetic_typenames))
                    .collect(),
            );
            response.insert_node(list)
        }
        Value::Object(object) => {
            let typename_node = include_synthetic_typenames
                .then_some(())
                .and_then(|_| object.synthetic_typename())
                .map(|typename| write_value(Value::String(typename), response, false));

            let fields = object.fields().map(|(name, value)| {
                (
                    ArcIntern::from_ref(name),
                    write_value(value, response, include_synthetic_typenames),
                )
            });

            let container = match typename_node {
                Some(typename_node) => {
                    let typename_iter = iter::once((ArcIntern::from_ref("__typename"), typename_node));
                    ResponseContainer::with_children(typename_iter.chain(fields))
                }
                None => ResponseContainer::with_children(fields),
            };

            response.insert_node(container)
        }
        Value::Inline(inner) => response.insert_node(inner.clone()),
    }
}
