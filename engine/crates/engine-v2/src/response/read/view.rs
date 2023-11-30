use schema::{ObjectId, Schema};
use serde::ser::{SerializeMap, SerializeSeq};

use super::ReadSelectionSet;
use crate::response::{ResponseBuilder, ResponseObject, ResponseObjectId, ResponsePath, ResponseValue};

pub struct ResponseObjectsView<'a> {
    pub(super) schema: &'a Schema,
    pub(super) response: &'a ResponseBuilder,
    pub(super) roots: Vec<ResponseObjectRoot>,
    pub(super) selection_set: &'a ReadSelectionSet,
}

#[derive(Debug, Clone)]
pub struct ResponseObjectRoot {
    pub id: ResponseObjectId,
    pub path: ResponsePath,
    pub object_id: ObjectId,
}

impl<'a> ResponseObjectsView<'a> {
    pub fn root(&self) -> ResponseObjectRoot {
        self.roots
            .first()
            .cloned()
            .expect("At least one object node id must be present in a Input.")
    }

    // Guaranteed to be in the same order as the response objects themselves
    #[allow(dead_code)]
    pub fn roots(&self) -> &[ResponseObjectRoot] {
        &self.roots
    }
}

impl<'a> serde::Serialize for ResponseObjectsView<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(self.roots.len()))?;
        for root in &self.roots {
            seq.serialize_element(&SerializableFilteredResponseObject {
                schema: self.schema,
                response: self.response,
                object: &self.response[root.id],
                selection_set: self.selection_set,
            })?;
        }
        seq.end()
    }
}

struct SerializableFilteredResponseObject<'a> {
    schema: &'a Schema,
    response: &'a ResponseBuilder,
    object: &'a ResponseObject,
    selection_set: &'a ReadSelectionSet,
}

impl<'a> serde::Serialize for SerializableFilteredResponseObject<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut map = serializer.serialize_map(Some(self.selection_set.len()))?;
        for selection in self.selection_set {
            map.serialize_key(&self.response.keys[selection.response_key])?;
            match self.object.find(selection.response_key).unwrap_or(&ResponseValue::Null) {
                ResponseValue::Null => map.serialize_value(&serde_json::Value::Null)?,
                ResponseValue::Boolean { value, .. } => map.serialize_value(value)?,
                ResponseValue::Int { value, .. } => map.serialize_value(value)?,
                ResponseValue::Float { value, .. } => map.serialize_value(value)?,
                ResponseValue::String { value, .. } => map.serialize_value(&value)?,
                ResponseValue::StringId { id, .. } => map.serialize_value(&self.schema[*id])?,
                ResponseValue::BigInt { value, .. } => map.serialize_value(value)?,
                ResponseValue::List { id, .. } => map.serialize_value(&SerializableFilteredResponseList {
                    schema: self.schema,
                    response: self.response,
                    value: &self.response[*id],
                    selection_set: &selection.subselection,
                })?,
                ResponseValue::Object { id, .. } => map.serialize_value(&SerializableFilteredResponseObject {
                    schema: self.schema,
                    response: self.response,
                    object: &self.response[*id],
                    selection_set: &selection.subselection,
                })?,
                ResponseValue::Json { value, .. } => map.serialize_value(value)?,
            }
        }

        map.end()
    }
}

struct SerializableFilteredResponseList<'a> {
    schema: &'a Schema,
    response: &'a ResponseBuilder,
    value: &'a [ResponseValue],
    selection_set: &'a ReadSelectionSet,
}

impl<'a> serde::Serialize for SerializableFilteredResponseList<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(self.value.len()))?;
        for node in self.value {
            match node {
                ResponseValue::Null => seq.serialize_element(&serde_json::Value::Null)?,
                ResponseValue::Boolean { value, .. } => seq.serialize_element(value)?,
                ResponseValue::Int { value, .. } => seq.serialize_element(value)?,
                ResponseValue::Float { value, .. } => seq.serialize_element(value)?,
                ResponseValue::String { value, .. } => seq.serialize_element(&value)?,
                ResponseValue::StringId { id, .. } => seq.serialize_element(&self.schema[*id])?,
                ResponseValue::BigInt { value, .. } => seq.serialize_element(value)?,
                ResponseValue::List { id, .. } => seq.serialize_element(&SerializableFilteredResponseList {
                    schema: self.schema,
                    response: self.response,
                    value: &self.response[*id],
                    selection_set: self.selection_set,
                })?,
                ResponseValue::Object { id, .. } => seq.serialize_element(&SerializableFilteredResponseObject {
                    schema: self.schema,
                    response: self.response,
                    object: &self.response[*id],
                    selection_set: self.selection_set,
                })?,
                ResponseValue::Json { value, .. } => seq.serialize_element(value)?,
            }
        }
        seq.end()
    }
}
