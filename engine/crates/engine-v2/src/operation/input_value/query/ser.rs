use serde::ser::{SerializeMap, SerializeSeq};
use walker::Walk;

use super::{QueryInputValue, QueryInputValueWalker};

impl<'ctx> serde::Serialize for QueryInputValueWalker<'ctx> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let input_values = &self.operation.query_input_values;
        match self.item {
            QueryInputValue::Null => serializer.serialize_none(),
            QueryInputValue::String(s) => s.serialize(serializer),
            QueryInputValue::EnumValue(id) => self.schema.walk(*id).name().serialize(serializer),
            QueryInputValue::Int(n) => n.serialize(serializer),
            QueryInputValue::BigInt(n) => n.serialize(serializer),
            QueryInputValue::Float(f) => f.serialize(serializer),
            QueryInputValue::U64(n) => n.serialize(serializer),
            QueryInputValue::Boolean(b) => b.serialize(serializer),
            QueryInputValue::InputObject(ids) => {
                let mut map = serializer.serialize_map(None)?;
                for (input_value_definition_id, value) in &input_values[*ids] {
                    let value = self.walk(value);
                    // https://spec.graphql.org/October2021/#sec-Input-Objects.Input-Coercion
                    if !value.is_undefined() {
                        map.serialize_key(self.schema.walk(*input_value_definition_id).name())?;
                        map.serialize_value(&value)?;
                    }
                }
                map.end()
            }
            QueryInputValue::List(ids) => {
                let mut seq = serializer.serialize_seq(Some(ids.len()))?;
                for value in &input_values[*ids] {
                    seq.serialize_element(&self.walk(value))?;
                }
                seq.end()
            }
            QueryInputValue::Map(ids) => {
                let mut map = serializer.serialize_map(Some(ids.len()))?;
                for (key, value) in &input_values[*ids] {
                    map.serialize_key(key)?;
                    map.serialize_value(&self.walk(value))?;
                }
                map.end()
            }
            QueryInputValue::DefaultValue(id) => id.walk(self.schema).serialize(serializer),
            QueryInputValue::Variable(id) => self.walk(*id).serialize(serializer),
        }
    }
}
