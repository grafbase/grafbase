use serde::ser::{SerializeMap, SerializeSeq};

use crate::{InputValue, InputValueDefinitionId, SchemaWalker};

pub struct SerializableInputValue<'a> {
    pub(super) schema: SchemaWalker<'a, ()>,
    pub(super) value: &'a InputValue,
}

impl<'a> serde::Serialize for SerializableInputValue<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match &self.value {
            InputValue::Null => serde_json::Value::Null.serialize(serializer),
            InputValue::String(s) => s.serialize(serializer),
            InputValue::StringId(id) => self.schema[*id].serialize(serializer),
            InputValue::Int(n) => n.serialize(serializer),
            InputValue::BigInt(n) => n.serialize(serializer),
            InputValue::Float(f) => f.serialize(serializer),
            InputValue::Boolean(b) => b.serialize(serializer),
            InputValue::Object(fields) => SerializableInputValueObject {
                schema: self.schema,
                fields,
            }
            .serialize(serializer),
            InputValue::List(list) => SerializableInputValueList {
                schema: self.schema,
                list,
            }
            .serialize(serializer),
            InputValue::Json(json) => json.serialize(serializer),
        }
    }
}

struct SerializableInputValueObject<'a> {
    schema: SchemaWalker<'a, ()>,
    fields: &'a [(InputValueDefinitionId, InputValue)],
}

impl<'a> serde::Serialize for SerializableInputValueObject<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut map = serializer.serialize_map(Some(self.fields.len()))?;
        for (input_value_definition_id, value) in self.fields {
            map.serialize_key(self.schema.walk(*input_value_definition_id).name())?;
            map.serialize_value(&SerializableInputValue {
                schema: self.schema,
                value,
            })?;
        }

        map.end()
    }
}

struct SerializableInputValueList<'a> {
    schema: SchemaWalker<'a, ()>,
    list: &'a [InputValue],
}

impl<'a> serde::Serialize for SerializableInputValueList<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(self.list.len()))?;
        for value in self.list {
            seq.serialize_element(&SerializableInputValue {
                schema: self.schema,
                value,
            })?;
        }
        seq.end()
    }
}
