use serde::ser::{SerializeMap, SerializeSeq};

use super::ReadSelectionSet;
use crate::response::{Response, ResponseObject, ResponseValue};

pub struct SerializableObject<'a> {
    pub response: &'a Response,
    pub object: ResponseObject<'a>,
    pub selection_set: &'a ReadSelectionSet,
}

impl<'a> serde::Serialize for SerializableObject<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut map = serializer.serialize_map(Some(self.selection_set.len()))?;
        match self.object {
            ResponseObject::Sparse(object) => {
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
                            map.serialize_value(&SerializableList {
                                response: self.response,
                                list,
                                selection_set: &selection.subselection,
                            })?;
                        }
                        ResponseValue::Object(id) => map.serialize_value(&SerializableObject {
                            response: self.response,
                            object: self.response.get(*id),
                            selection_set: &selection.subselection,
                        })?,
                    }
                }
            }
            ResponseObject::Dense(object) => {
                for selection in self.selection_set {
                    let value = &object.fields[selection.response_position];
                    map.serialize_key(&self.response.strings[selection.response_name])?;
                    match value {
                        ResponseValue::Null => map.serialize_value(&serde_json::Value::Null)?,
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
                            object: self.response.get(*id),
                            selection_set: &selection.subselection,
                        })?,
                    }
                }
            }
        }
        map.end()
    }
}

pub struct SerializableList<'a> {
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
                ResponseValue::Null => seq.serialize_element(&serde_json::Value::Null)?,
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
                    object: self.response.get(*id),
                    selection_set: self.selection_set,
                })?,
            }
        }
        seq.end()
    }
}
