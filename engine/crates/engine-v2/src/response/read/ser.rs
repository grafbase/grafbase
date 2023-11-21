use schema::Schema;
use serde::ser::{SerializeMap, SerializeSeq};

use super::ReadSelectionSet;
use crate::response::{AnyResponseObject, ResponseData, ResponseValue};

pub struct SerializableResponseData<'a> {
    pub(super) schema: &'a Schema,
    pub(super) data: ResponseData,
    pub(super) selection_set: ReadSelectionSet,
}

impl<'a> serde::Serialize for SerializableResponseData<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.data
            .root
            .map(|root| SerializableResponseObject {
                schema: self.schema,
                response: &self.data,
                object: self.data.get(root),
                selection_set: &self.selection_set,
            })
            .serialize(serializer)
    }
}

pub(super) struct SerializableResponseObject<'a> {
    pub(super) schema: &'a Schema,
    pub(super) response: &'a ResponseData,
    pub(super) object: AnyResponseObject<'a>,
    pub(super) selection_set: &'a ReadSelectionSet,
}

impl<'a> serde::Serialize for SerializableResponseObject<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut map = serializer.serialize_map(Some(self.selection_set.len()))?;
        match self.object {
            AnyResponseObject::Sparse(object) => {
                for selection in self.selection_set {
                    map.serialize_key(&self.response.strings[selection.response_name])?;
                    match object
                        .fields
                        .get(&selection.response_name)
                        .unwrap_or(&ResponseValue::Null)
                    {
                        ResponseValue::Null => map.serialize_value(&serde_json::Value::Null)?,
                        ResponseValue::Bool(b) => map.serialize_value(b)?,
                        ResponseValue::Number(n) => map.serialize_value(n)?,
                        ResponseValue::String(s) => map.serialize_value(s)?,
                        ResponseValue::List(list) => {
                            map.serialize_value(&SerializableResponseList {
                                schema: self.schema,
                                response: self.response,
                                list,
                                selection_set: &selection.subselection,
                            })?;
                        }
                        ResponseValue::Object(id) => map.serialize_value(&SerializableResponseObject {
                            schema: self.schema,
                            response: self.response,
                            object: self.response.get(*id),
                            selection_set: &selection.subselection,
                        })?,
                        ResponseValue::StrId(id) => map.serialize_value(&self.response.strings[*id])?,
                        ResponseValue::StringId(id) => map.serialize_value(&self.schema[*id])?,
                    }
                }
            }
            AnyResponseObject::Dense(object) => {
                for selection in self.selection_set {
                    let value = &object.fields[selection.response_position];
                    map.serialize_key(&self.response.strings[selection.response_name])?;
                    match value {
                        ResponseValue::Null => map.serialize_value(&serde_json::Value::Null)?,
                        ResponseValue::Bool(b) => map.serialize_value(b)?,
                        ResponseValue::Number(n) => map.serialize_value(n)?,
                        ResponseValue::String(s) => map.serialize_value(s)?,
                        ResponseValue::List(list) => {
                            map.serialize_value(&SerializableResponseList {
                                schema: self.schema,
                                response: self.response,
                                list,
                                selection_set: &selection.subselection,
                            })?;
                        }
                        ResponseValue::Object(id) => map.serialize_value(&SerializableResponseObject {
                            schema: self.schema,
                            response: self.response,
                            object: self.response.get(*id),
                            selection_set: &selection.subselection,
                        })?,
                        ResponseValue::StrId(id) => map.serialize_value(&self.response.strings[*id])?,
                        ResponseValue::StringId(id) => map.serialize_value(&self.schema[*id])?,
                    }
                }
            }
        }
        map.end()
    }
}

struct SerializableResponseList<'a> {
    schema: &'a Schema,
    response: &'a ResponseData,
    list: &'a Vec<ResponseValue>,
    selection_set: &'a ReadSelectionSet,
}

impl<'a> serde::Serialize for SerializableResponseList<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(self.list.len()))?;
        for node in self.list {
            match node {
                ResponseValue::Null => seq.serialize_element(&serde_json::Value::Null)?,
                ResponseValue::Bool(b) => seq.serialize_element(b)?,
                ResponseValue::Number(n) => seq.serialize_element(n)?,
                ResponseValue::String(s) => seq.serialize_element(s)?,
                ResponseValue::List(list) => seq.serialize_element(&SerializableResponseList {
                    schema: self.schema,
                    response: self.response,
                    list,
                    selection_set: self.selection_set,
                })?,
                ResponseValue::Object(id) => seq.serialize_element(&SerializableResponseObject {
                    schema: self.schema,
                    response: self.response,
                    object: self.response.get(*id),
                    selection_set: self.selection_set,
                })?,
                ResponseValue::StrId(id) => seq.serialize_element(&self.response.strings[*id])?,
                ResponseValue::StringId(id) => seq.serialize_element(&self.schema[*id])?,
            }
        }
        seq.end()
    }
}
