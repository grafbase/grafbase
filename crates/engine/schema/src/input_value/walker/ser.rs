use walker::Walk;

use crate::SchemaInputValueRecord;

use super::SchemaInputValue;

impl serde::Serialize for SchemaInputValue<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self.ref_ {
            SchemaInputValueRecord::Null => serializer.serialize_none(),
            SchemaInputValueRecord::String(id) | SchemaInputValueRecord::UnboundEnumValue(id) => {
                self.schema[*id].serialize(serializer)
            }
            SchemaInputValueRecord::EnumValue(id) => id.walk(self.schema).name().serialize(serializer),
            SchemaInputValueRecord::Int(n) => n.serialize(serializer),
            SchemaInputValueRecord::BigInt(n) => n.serialize(serializer),
            SchemaInputValueRecord::Float(f) => f.serialize(serializer),
            SchemaInputValueRecord::U64(n) => n.serialize(serializer),
            SchemaInputValueRecord::Boolean(b) => b.serialize(serializer),
            SchemaInputValueRecord::InputObject(ids) => serializer.collect_map(
                ids.walk(self.schema)
                    .map(|(input_value_definition, value)| (input_value_definition.name(), value)),
            ),
            SchemaInputValueRecord::List(ids) => serializer.collect_seq(ids.walk(self.schema)),
            SchemaInputValueRecord::Map(ids) => serializer.collect_map(ids.walk(self.schema)),
        }
    }
}
