use schema::RawInputValuesContext;
use serde::ser::{SerializeMap, SerializeSeq};

use super::{QueryInputValue, QueryInputValueWalker};

impl<'ctx> serde::Serialize for QueryInputValueWalker<'ctx> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self.item {
            QueryInputValue::Null => serializer.serialize_none(),
            QueryInputValue::String(s) => s.serialize(serializer),
            QueryInputValue::EnumValue(id) => self.schema_walker.walk(*id).name().serialize(serializer),
            QueryInputValue::Int(n) => n.serialize(serializer),
            QueryInputValue::BigInt(n) => n.serialize(serializer),
            QueryInputValue::Float(f) => f.serialize(serializer),
            QueryInputValue::U64(n) => n.serialize(serializer),
            QueryInputValue::Boolean(b) => b.serialize(serializer),
            QueryInputValue::InputObject(ids) => {
                let mut map = serializer.serialize_map(None)?;
                for (input_value_definition_id, value) in &self.operation[*ids] {
                    let value = self.walk(value);
                    // https://spec.graphql.org/October2021/#sec-Input-Objects.Input-Coercion
                    if !value.is_undefined() {
                        map.serialize_key(self.schema_walker.walk(*input_value_definition_id).name())?;
                        map.serialize_value(&value)?;
                    }
                }
                map.end()
            }
            QueryInputValue::List(ids) => {
                let mut seq = serializer.serialize_seq(Some(ids.len()))?;
                for value in &self.operation[*ids] {
                    seq.serialize_element(&self.walk(value))?;
                }
                seq.end()
            }
            QueryInputValue::Map(ids) => {
                let mut map = serializer.serialize_map(None)?;
                for (key, value) in &self.operation[*ids] {
                    map.serialize_key(key)?;
                    map.serialize_value(&self.walk(value))?;
                }
                map.end()
            }
            QueryInputValue::DefaultValue(id) => {
                RawInputValuesContext::walk(&self.schema_walker, *id).serialize(serializer)
            }
            QueryInputValue::Variable(id) => self.walk(*id).serialize(serializer),
        }
    }
}
