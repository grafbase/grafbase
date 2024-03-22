use crate::{InputValue, SchemaInputValue, SchemaWalker};

pub type SchemaInputValueWalker<'a> = SchemaWalker<'a, &'a SchemaInputValue>;

impl<'a> From<SchemaInputValueWalker<'a>> for InputValue<'a> {
    fn from(walker: SchemaInputValueWalker<'a>) -> Self {
        match walker.item {
            SchemaInputValue::Null => InputValue::Null,
            SchemaInputValue::String(id) => InputValue::String(&walker.schema[*id]),
            SchemaInputValue::EnumValue(id) => InputValue::EnumValue(*id),
            SchemaInputValue::Int(n) => InputValue::Int(*n),
            SchemaInputValue::BigInt(n) => InputValue::BigInt(*n),
            SchemaInputValue::Float(f) => InputValue::Float(*f),
            SchemaInputValue::Boolean(b) => InputValue::Boolean(*b),
            SchemaInputValue::InputObject(ids) => {
                let mut fields = Vec::with_capacity(ids.len());
                for (input_value_definition_id, value) in &walker.schema[*ids] {
                    fields.push((*input_value_definition_id, walker.walk(value).into()));
                }
                InputValue::InputObject(fields.into_boxed_slice())
            }
            SchemaInputValue::List(ids) => {
                let mut values = Vec::with_capacity(ids.len());
                for value in &walker.schema[*ids] {
                    values.push(walker.walk(value).into());
                }
                InputValue::List(values.into_boxed_slice())
            }
            SchemaInputValue::Map(ids) => {
                let mut key_values = Vec::with_capacity(ids.len());
                for (key, value) in &walker.schema[*ids] {
                    key_values.push((walker.schema[*key].as_str(), Self::from(walker.walk(value))));
                }
                InputValue::Map(key_values.into_boxed_slice())
            }
            SchemaInputValue::U64(n) => InputValue::U64(*n),
        }
    }
}

impl std::fmt::Debug for SchemaInputValueWalker<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.item {
            SchemaInputValue::Null => write!(f, "Null"),
            SchemaInputValue::String(s) => s.fmt(f),
            SchemaInputValue::EnumValue(id) => f.debug_tuple("EnumValue").field(&self.walk(*id).name()).finish(),
            SchemaInputValue::Int(n) => f.debug_tuple("Int").field(n).finish(),
            SchemaInputValue::BigInt(n) => f.debug_tuple("BigInt").field(n).finish(),
            SchemaInputValue::U64(n) => f.debug_tuple("U64").field(n).finish(),
            SchemaInputValue::Float(n) => f.debug_tuple("Float").field(n).finish(),
            SchemaInputValue::Boolean(b) => b.fmt(f),
            SchemaInputValue::InputObject(ids) => {
                let mut map = f.debug_struct("InputObject");
                for (input_value_definition_id, value) in &self.schema[*ids] {
                    map.field(self.walk(*input_value_definition_id).name(), &self.walk(value));
                }
                map.finish()
            }
            SchemaInputValue::List(ids) => {
                let mut seq = f.debug_list();
                for value in &self.schema[*ids] {
                    seq.entry(&self.walk(value));
                }
                seq.finish()
            }
            SchemaInputValue::Map(ids) => {
                let mut map = f.debug_map();
                for (key, value) in &self.schema[*ids] {
                    map.entry(&key, &self.walk(value));
                }
                map.finish()
            }
        }
    }
}
