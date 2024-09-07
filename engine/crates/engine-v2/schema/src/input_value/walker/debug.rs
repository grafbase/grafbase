use walker::Walk;

use crate::SchemaInputValueRecord;

use super::SchemaInputValue;

impl std::fmt::Debug for SchemaInputValue<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let SchemaInputValue { schema, value } = *self;
        match value {
            SchemaInputValueRecord::Null => write!(f, "Null"),
            SchemaInputValueRecord::String(s) => s.fmt(f),
            SchemaInputValueRecord::EnumValue(id) => f.debug_tuple("EnumValue").field(&id.walk(schema).name()).finish(),
            SchemaInputValueRecord::Int(n) => f.debug_tuple("Int").field(n).finish(),
            SchemaInputValueRecord::BigInt(n) => f.debug_tuple("BigInt").field(n).finish(),
            SchemaInputValueRecord::U64(n) => f.debug_tuple("U64").field(n).finish(),
            SchemaInputValueRecord::Float(n) => f.debug_tuple("Float").field(n).finish(),
            SchemaInputValueRecord::Boolean(b) => b.fmt(f),
            SchemaInputValueRecord::InputObject(ids) => {
                let mut map = f.debug_struct("InputObject");
                for (input_value_definition, value) in ids.walk(schema) {
                    map.field(input_value_definition.name(), &value);
                }
                map.finish()
            }
            SchemaInputValueRecord::List(ids) => f.debug_list().entries(ids.walk(schema)).finish(),
            SchemaInputValueRecord::Map(ids) => f.debug_map().entries(ids.walk(schema)).finish(),
        }
    }
}
