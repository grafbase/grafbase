use serde::ser::{SerializeMap, SerializeSeq};
use walker::Walk;

use super::{VariableInputValueRecord, VariableInputValueWalker};

impl<'ctx> serde::Serialize for VariableInputValueWalker<'ctx> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self.item {
            VariableInputValueRecord::Null => serializer.serialize_none(),
            VariableInputValueRecord::String(s) => s.serialize(serializer),
            VariableInputValueRecord::EnumValue(id) => self.schema.walk(*id).name().serialize(serializer),
            VariableInputValueRecord::Int(n) => n.serialize(serializer),
            VariableInputValueRecord::BigInt(n) => n.serialize(serializer),
            VariableInputValueRecord::Float(f) => f.serialize(serializer),
            VariableInputValueRecord::U64(n) => n.serialize(serializer),
            VariableInputValueRecord::Boolean(b) => b.serialize(serializer),
            VariableInputValueRecord::InputObject(ids) => {
                let mut map = serializer.serialize_map(Some(ids.len()))?;
                for (input_value_definition_id, value) in &self.variables[*ids] {
                    map.serialize_key(self.schema.walk(*input_value_definition_id).name())?;
                    map.serialize_value(&self.walk(value))?;
                }
                map.end()
            }
            VariableInputValueRecord::List(ids) => {
                let mut seq = serializer.serialize_seq(Some(ids.len()))?;
                for value in &self.variables[*ids] {
                    seq.serialize_element(&self.walk(value))?;
                }
                seq.end()
            }
            VariableInputValueRecord::Map(ids) => {
                let mut map = serializer.serialize_map(Some(ids.len()))?;
                for (key, value) in &self.variables[*ids] {
                    let value = self.walk(value);
                    map.serialize_key(key)?;
                    map.serialize_value(&value)?;
                }
                map.end()
            }
            VariableInputValueRecord::DefaultValue(id) => id.walk(self.schema).serialize(serializer),
        }
    }
}
