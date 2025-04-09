use walker::Walk;

use crate::SchemaInputValueRecord;

use super::SchemaInputValue;

impl std::fmt::Debug for SchemaInputValue<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.ref_ {
            SchemaInputValueRecord::Null => write!(f, "Null"),
            SchemaInputValueRecord::String(s) => s.fmt(f),
            SchemaInputValueRecord::EnumValue(id) => {
                f.debug_tuple("EnumValue").field(&id.walk(self.schema).name()).finish()
            }
            SchemaInputValueRecord::UnboundEnumValue(id) => {
                f.debug_tuple("UnknownEnumValue").field(&id.walk(self.schema)).finish()
            }
            SchemaInputValueRecord::Int(n) => f.debug_tuple("Int").field(n).finish(),
            SchemaInputValueRecord::I64(n) => f.debug_tuple("I64").field(n).finish(),
            SchemaInputValueRecord::U64(n) => f.debug_tuple("U64").field(n).finish(),
            SchemaInputValueRecord::Float(n) => f.debug_tuple("Float").field(n).finish(),
            SchemaInputValueRecord::Boolean(b) => b.fmt(f),
            SchemaInputValueRecord::InputObject(ids) => {
                let mut map = f.debug_struct("InputObject");
                for (input_value_definition, value) in ids.walk(self.schema) {
                    map.field(input_value_definition.name(), &value);
                }
                map.finish()
            }
            SchemaInputValueRecord::List(ids) => f.debug_list().entries(ids.walk(self.schema)).finish(),
            SchemaInputValueRecord::Map(ids) => f.debug_map().entries(ids.walk(self.schema)).finish(),
        }
    }
}
