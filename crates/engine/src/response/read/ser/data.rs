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
            map.serialize_value(&SerializableResponseValue { ctx: self.ctx, value })?;
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
        for value in self.value {
            seq.serialize_element(&SerializableResponseValue { ctx: self.ctx, value })?;
        }
        seq.end()
    }
}

struct SerializableResponseValue<'a> {
    ctx: Context<'a>,
    value: &'a ResponseValue,
}

impl serde::Serialize for SerializableResponseValue<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self.value {
            ResponseValue::Null | ResponseValue::Inaccessible { .. } | ResponseValue::Unexpected => {
                serializer.serialize_none()
            }
            ResponseValue::Boolean { value, .. } => value.serialize(serializer),
            ResponseValue::Int { value, .. } => value.serialize(serializer),
            ResponseValue::Float { value, .. } => value.serialize(serializer),
            ResponseValue::String { value, .. } => value.serialize(serializer),
            ResponseValue::StringId { id, .. } => self.ctx.schema[*id].serialize(serializer),
            ResponseValue::BigInt { value, .. } => value.serialize(serializer),
            ResponseValue::List { id, .. } => SerializableResponseList {
                ctx: self.ctx,
                value: &self.ctx.data[*id],
            }
            .serialize(serializer),
            ResponseValue::Object { id, .. } => SerializableResponseObject {
                ctx: self.ctx,
                object: &self.ctx.data[*id],
            }
            .serialize(serializer),
            ResponseValue::U64 { value } => value.serialize(serializer),
            ResponseValue::Map { id } => serializer.collect_map(
                self.ctx.data[*id]
                    .iter()
                    .map(|(key, value)| (key.as_str(), SerializableResponseValue { ctx: self.ctx, value })),
            ),
        }
    }
}
