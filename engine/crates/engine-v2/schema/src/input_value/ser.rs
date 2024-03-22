use serde::ser::{SerializeMap, SerializeSeq};

use crate::{SchemaInputValue, SchemaInputValueWalker};

impl<'a> serde::Serialize for SchemaInputValueWalker<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self.item {
            SchemaInputValue::Null => serializer.serialize_none(),
            SchemaInputValue::String(id) => self.schema[*id].serialize(serializer),
            SchemaInputValue::EnumValue(id) => self.walk(*id).name().serialize(serializer),
            SchemaInputValue::Int(n) => n.serialize(serializer),
            SchemaInputValue::BigInt(n) => n.serialize(serializer),
            SchemaInputValue::Float(f) => f.serialize(serializer),
            SchemaInputValue::U64(n) => n.serialize(serializer),
            SchemaInputValue::Boolean(b) => b.serialize(serializer),
            &SchemaInputValue::InputObject(ids) => {
                let mut map = serializer.serialize_map(Some(ids.len()))?;
                for (input_value_definition_id, value) in &self.schema[ids] {
                    let value = self.walk(value);
                    map.serialize_key(&self.walk(*input_value_definition_id).name())?;
                    map.serialize_value(&value)?;
                }
                map.end()
            }
            &SchemaInputValue::List(ids) => {
                let mut seq = serializer.serialize_seq(Some(ids.len()))?;
                for value in &self.schema[ids] {
                    seq.serialize_element(&self.walk(value))?;
                }
                seq.end()
            }
            &SchemaInputValue::Map(ids) => {
                let mut map = serializer.serialize_map(Some(ids.len()))?;
                for (key, value) in &self.schema[ids] {
                    let value = self.walk(value);
                    map.serialize_key(&self.schema[*key])?;
                    map.serialize_value(&value)?;
                }
                map.end()
            }
        }
    }
}
