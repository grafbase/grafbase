use schema::Schema;
use serde::ser::{SerializeMap, SerializeSeq};

use crate::response::{value::ResponseObjectField, ResponseData, ResponseKeys, ResponseObject, ResponseValue};

#[derive(Clone, Copy)]
pub(super) struct Context<'a> {
    pub keys: &'a ResponseKeys,
    pub data: &'a ResponseData,
    pub schema: &'a Schema,
}

pub(super) struct SerializableResponseData<'a> {
    pub ctx: Context<'a>,
}

impl serde::Serialize for SerializableResponseData<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        SerializableResponseObject {
            ctx: self.ctx,
            object: self.ctx.data.root_object(),
        }
        .serialize(serializer)
    }
}

struct SerializableResponseObject<'a> {
    ctx: Context<'a>,
    object: &'a ResponseObject,
}

impl serde::Serialize for SerializableResponseObject<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut map = serializer.serialize_map(Some(self.object.len()))?;
        // Thanks to the BoundResponseKey starting with the position and the fields being a BTreeMap
        // we're ensuring the fields are serialized in the order they appear in the query.
        for ResponseObjectField { key: edge, value, .. } in self.object.fields() {
            if edge.query_position.is_none() {
                // Bound response keys are always first, anything after are extra fields which
                // don't need to be serialized.
                break;
            };
            map.serialize_key(&self.ctx.keys[edge.response_key])?;
            match value {
                ResponseValue::Null | ResponseValue::Inaccessible { .. } => map.serialize_value(&())?,
                ResponseValue::Boolean { value, .. } => map.serialize_value(value)?,
                ResponseValue::Int { value, .. } => map.serialize_value(value)?,
                ResponseValue::Float { value, .. } => map.serialize_value(value)?,
                ResponseValue::String { value, .. } => map.serialize_value(&value)?,
                ResponseValue::StringId { id, .. } => map.serialize_value(&self.ctx.schema[*id])?,
                ResponseValue::BigInt { value, .. } => map.serialize_value(value)?,
                &ResponseValue::List { id, .. } => map.serialize_value(&SerializableResponseList {
                    ctx: self.ctx,
                    value: &self.ctx.data[id],
                })?,
                &ResponseValue::Object { id, .. } => map.serialize_value(&SerializableResponseObject {
                    ctx: self.ctx,
                    object: &self.ctx.data[id],
                })?,
                ResponseValue::Json { value, .. } => map.serialize_value(value)?,
            }
        }
        map.end()
    }
}

struct SerializableResponseList<'a> {
    ctx: Context<'a>,
    value: &'a [ResponseValue],
}

impl serde::Serialize for SerializableResponseList<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(self.value.len()))?;
        for node in self.value {
            match node {
                ResponseValue::Null | ResponseValue::Inaccessible { .. } => seq.serialize_element(&())?,
                ResponseValue::Boolean { value, .. } => seq.serialize_element(value)?,
                ResponseValue::Int { value, .. } => seq.serialize_element(value)?,
                ResponseValue::Float { value, .. } => seq.serialize_element(value)?,
                ResponseValue::String { value, .. } => seq.serialize_element(&value)?,
                ResponseValue::StringId { id, .. } => seq.serialize_element(&self.ctx.schema[*id])?,
                ResponseValue::BigInt { value, .. } => seq.serialize_element(value)?,
                &ResponseValue::List { id, .. } => seq.serialize_element(&SerializableResponseList {
                    ctx: self.ctx,
                    value: &self.ctx.data[id],
                })?,
                &ResponseValue::Object { id, .. } => seq.serialize_element(&SerializableResponseObject {
                    ctx: self.ctx,
                    object: &self.ctx.data[id],
                })?,
                ResponseValue::Json { value, .. } => seq.serialize_element(value)?,
            }
        }
        seq.end()
    }
}
