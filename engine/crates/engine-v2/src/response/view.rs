use serde::ser::{SerializeMap, SerializeSeq};
use serde_json::Value;

use super::{ReadSelectionSet, Response, ResponseObject, ResponseObjectId, ResponseValue};

pub struct ResponseObjectsView<'a> {
    pub(super) response: &'a Response,
    pub(super) ids: Vec<ResponseObjectId>,
    pub(super) selection_set: &'a ReadSelectionSet,
}

impl<'a> ResponseObjectsView<'a> {
    pub fn id(&self) -> ResponseObjectId {
        *self
            .ids
            .get(0)
            .expect("At least one object node id must be present in a Input.")
    }

    // Guaranteed to be in the same order as the response objects themselves
    pub fn ids(&self) -> &[ResponseObjectId] {
        &self.ids
    }
}

impl<'a> serde::Serialize for ResponseObjectsView<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(self.ids.len()))?;
        for node_id in &self.ids {
            seq.serialize_element(&SerializableObject {
                response: self.response,
                object: &self.response[*node_id],
                selection_set: self.selection_set,
            })?;
        }
        seq.end()
    }
}

struct SerializableObject<'a> {
    response: &'a Response,
    object: &'a ResponseObject,
    selection_set: &'a ReadSelectionSet,
}

impl<'a> serde::Serialize for SerializableObject<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut map = serializer.serialize_map(Some(self.object.fields.len()))?;
        for (name, node) in &self.object.fields {
            if let Some(selection) = self.selection_set.find_field(*name) {
                map.serialize_key(&self.response[selection.name])?;
                match node {
                    ResponseValue::Null => map.serialize_value(&Value::Null)?,
                    ResponseValue::Bool(b) => map.serialize_value(b)?,
                    ResponseValue::Number(n) => map.serialize_value(n)?,
                    ResponseValue::String(s) => map.serialize_value(s)?,
                    ResponseValue::List(list) => {
                        map.serialize_value(&SerializableList {
                            response: self.response,
                            list,
                            selection_set: &selection.subselection,
                        })?;
                    }
                    ResponseValue::Object(id) => map.serialize_value(&SerializableObject {
                        response: self.response,
                        object: &self.response[*id],
                        selection_set: &selection.subselection,
                    })?,
                }
            }
        }
        map.end()
    }
}

struct SerializableList<'a> {
    response: &'a Response,
    list: &'a Vec<ResponseValue>,
    selection_set: &'a ReadSelectionSet,
}

impl<'a> serde::Serialize for SerializableList<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(self.list.len()))?;
        for node in self.list {
            match node {
                ResponseValue::Null => seq.serialize_element(&Value::Null)?,
                ResponseValue::Bool(b) => seq.serialize_element(b)?,
                ResponseValue::Number(n) => seq.serialize_element(n)?,
                ResponseValue::String(s) => seq.serialize_element(s)?,
                ResponseValue::List(list) => seq.serialize_element(&SerializableList {
                    response: self.response,
                    list,
                    selection_set: self.selection_set,
                })?,
                ResponseValue::Object(id) => seq.serialize_element(&SerializableObject {
                    response: self.response,
                    object: &self.response[*id],
                    selection_set: self.selection_set,
                })?,
            }
        }
        seq.end()
    }
}
