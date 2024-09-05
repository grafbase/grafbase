use serde::ser::{SerializeMap, SerializeSeq};

use crate::{SchemaInputValueRecord, SchemaInputValueWalker};

impl<'a> serde::Serialize for SchemaInputValueWalker<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self.item {
            SchemaInputValueRecord::Null => serializer.serialize_none(),
            SchemaInputValueRecord::String(id) => self.schema[*id].serialize(serializer),
            SchemaInputValueRecord::EnumValue(id) => self.walk(*id).name().serialize(serializer),
            SchemaInputValueRecord::Int(n) => n.serialize(serializer),
            SchemaInputValueRecord::BigInt(n) => n.serialize(serializer),
            SchemaInputValueRecord::Float(f) => f.serialize(serializer),
            SchemaInputValueRecord::U64(n) => n.serialize(serializer),
            SchemaInputValueRecord::Boolean(b) => b.serialize(serializer),
            &SchemaInputValueRecord::InputObject(ids) => {
                let mut map = serializer.serialize_map(Some(ids.len()))?;
                for (input_value_definition_id, value) in &self.schema[ids] {
                    let value = self.walk(value);
                    map.serialize_key(&self.walk(*input_value_definition_id).name())?;
                    map.serialize_value(&value)?;
                }
                map.end()
            }
            &SchemaInputValueRecord::List(ids) => {
                let mut seq = serializer.serialize_seq(Some(ids.len()))?;
                for value in &self.schema[ids] {
                    seq.serialize_element(&self.walk(value))?;
                }
                seq.end()
            }
            &SchemaInputValueRecord::Map(ids) => {
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
