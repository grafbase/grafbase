use std::{borrow::Cow, sync::Arc};

use schema::{ObjectId, SchemaWalker};
use serde::ser::{SerializeMap, SerializeSeq};

use super::ReadSelectionSet;
use crate::response::{ResponseBuilder, ResponseObject, ResponseObjectId, ResponsePath, ResponseValue};

pub struct ResponseBoundaryObjectsView<'a> {
    pub(super) schema: SchemaWalker<'a, ()>,
    pub(super) response: &'a ResponseBuilder,
    pub(super) items: Arc<Vec<ResponseBoundaryItem>>,
    pub(super) selection_set: Cow<'a, ReadSelectionSet>,
    pub(super) extra_constant_fields: Vec<(String, serde_json::Value)>,
}

#[derive(Debug, Clone)]
pub struct ResponseBoundaryItem {
    pub response_object_id: ResponseObjectId,
    pub response_path: ResponsePath,
    pub object_id: ObjectId,
}

impl<'a> ResponseBoundaryObjectsView<'a> {
    pub fn into_single_boundary_item(self) -> ResponseBoundaryItem {
        self.items
            .iter()
            .next()
            .cloned()
            .expect("There is always at least one input, there would be no plan otherwise.")
    }

    // Guaranteed to be in the same order as the response objects themselves
    pub fn items(&self) -> &Arc<Vec<ResponseBoundaryItem>> {
        &self.items
    }

    pub fn with_extra_constant_fields(
        mut self,
        extra_constant_fields: Vec<(String, serde_json::Value)>,
    ) -> ResponseBoundaryObjectsView<'a> {
        self.extra_constant_fields = extra_constant_fields;
        self
    }
}

impl<'a> serde::Serialize for ResponseBoundaryObjectsView<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(self.items.len()))?;
        for item in self.items.as_ref() {
            seq.serialize_element(&SerializableFilteredResponseObject {
                schema: self.schema,
                response: self.response,
                response_object: &self.response[item.response_object_id],
                selection_set: &self.selection_set,
                extra_constant_fields: &self.extra_constant_fields,
            })?;
        }
        seq.end()
    }
}

struct SerializableFilteredResponseObject<'a> {
    schema: SchemaWalker<'a, ()>,
    response: &'a ResponseBuilder,
    response_object: &'a ResponseObject,
    selection_set: &'a ReadSelectionSet,
    extra_constant_fields: &'a [(String, serde_json::Value)],
}

impl<'a> serde::Serialize for SerializableFilteredResponseObject<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut map = serializer.serialize_map(Some(self.selection_set.len() + self.extra_constant_fields.len()))?;
        for (name, value) in self.extra_constant_fields {
            map.serialize_key(name)?;
            map.serialize_value(value)?;
        }
        for selection in self.selection_set {
            map.serialize_key(&selection.name)?;
            if let Some(value) = self.response_object.find(selection.edge) {
                match value {
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
                        response_list: &self.response[*id],
                        selection_set: &selection.subselection,
                        extra_constant_fields: self.extra_constant_fields,
                    })?,
                    ResponseValue::Object { id, .. } => map.serialize_value(&SerializableFilteredResponseObject {
                        schema: self.schema,
                        response: self.response,
                        response_object: &self.response[*id],
                        selection_set: &selection.subselection,
                        extra_constant_fields: self.extra_constant_fields,
                    })?,
                    ResponseValue::Json { value, .. } => map.serialize_value(value)?,
                }
            } else {
                map.serialize_value(&serde_json::Value::Null)?
            }
        }

        map.end()
    }
}

struct SerializableFilteredResponseList<'a> {
    schema: SchemaWalker<'a, ()>,
    response: &'a ResponseBuilder,
    response_list: &'a [ResponseValue],
    selection_set: &'a ReadSelectionSet,
    extra_constant_fields: &'a [(String, serde_json::Value)],
}

impl<'a> serde::Serialize for SerializableFilteredResponseList<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(self.response_list.len()))?;
        for node in self.response_list {
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
                    response_list: &self.response[*id],
                    selection_set: self.selection_set,
                    extra_constant_fields: self.extra_constant_fields,
                })?,
                ResponseValue::Object { id, .. } => seq.serialize_element(&SerializableFilteredResponseObject {
                    schema: self.schema,
                    response: self.response,
                    response_object: &self.response[*id],
                    selection_set: self.selection_set,
                    extra_constant_fields: self.extra_constant_fields,
                })?,
                ResponseValue::Json { value, .. } => seq.serialize_element(value)?,
            }
        }
        seq.end()
    }
}
