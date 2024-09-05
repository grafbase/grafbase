use readable::Readable;
use serde::ser::{SerializeMap, SerializeSeq};

use super::{VariableInputValue, VariableInputValueWalker};

impl<'ctx> serde::Serialize for VariableInputValueWalker<'ctx> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self.item {
            VariableInputValue::Null => serializer.serialize_none(),
            VariableInputValue::String(s) => s.serialize(serializer),
            VariableInputValue::EnumValue(id) => self.schema.walk(*id).name().serialize(serializer),
            VariableInputValue::Int(n) => n.serialize(serializer),
            VariableInputValue::BigInt(n) => n.serialize(serializer),
            VariableInputValue::Float(f) => f.serialize(serializer),
            VariableInputValue::U64(n) => n.serialize(serializer),
            VariableInputValue::Boolean(b) => b.serialize(serializer),
            VariableInputValue::InputObject(ids) => {
                let mut map = serializer.serialize_map(Some(ids.len()))?;
                for (input_value_definition_id, value) in &self.variables[*ids] {
                    map.serialize_key(self.schema.walk(*input_value_definition_id).name())?;
                    map.serialize_value(&self.walk(value))?;
                }
                map.end()
            }
            VariableInputValue::List(ids) => {
                let mut seq = serializer.serialize_seq(Some(ids.len()))?;
                for value in &self.variables[*ids] {
                    seq.serialize_element(&self.walk(value))?;
                }
                seq.end()
            }
            VariableInputValue::Map(ids) => {
                let mut map = serializer.serialize_map(Some(ids.len()))?;
                for (key, value) in &self.variables[*ids] {
                    let value = self.walk(value);
                    map.serialize_key(key)?;
                    map.serialize_value(&value)?;
                }
                map.end()
            }
            VariableInputValue::DefaultValue(id) => id.read(self.schema).serialize(serializer),
        }
    }
}
