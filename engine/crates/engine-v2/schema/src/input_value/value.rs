use walker::Walk;

use crate::{EnumValue, InputValueDefinition};

use super::{SchemaInputValue, SchemaInputValueRecord};

/// implement a Deserializer & Serialize trait, but if you need to traverse a dynamic type,
/// this will be the one to use. All input values can be converted to it.
#[derive(Default, Debug, Clone)]
pub enum InputValue<'a> {
    #[default]
    Null,
    String(&'a str),
    EnumValue(EnumValue<'a>),
    Int(i32),
    BigInt(i64),
    Float(f64),
    Boolean(bool),
    // There is no guarantee on the ordering.
    InputObject(Vec<(InputValueDefinition<'a>, InputValue<'a>)>),
    List(Vec<InputValue<'a>>),

    /// for JSON
    Map(Vec<(&'a str, InputValue<'a>)>), // no guarantee on the ordering
    U64(u64),

    /// We may encounter unbound enum values within a scalar for which we have no definition. In
    /// this case we keep track of it.
    UnboundEnumValue(&'a str),
}

/// Provided if you need to serialize only a part of an input value.
impl serde::Serialize for InputValue<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            InputValue::Null => serializer.serialize_none(),
            InputValue::String(s) | InputValue::UnboundEnumValue(s) => s.serialize(serializer),
            InputValue::EnumValue(enum_) => enum_.name().serialize(serializer),
            InputValue::Int(n) => n.serialize(serializer),
            InputValue::BigInt(n) => n.serialize(serializer),
            InputValue::Float(f) => f.serialize(serializer),
            InputValue::U64(n) => n.serialize(serializer),
            InputValue::Boolean(b) => b.serialize(serializer),
            InputValue::InputObject(fields) => serializer.collect_map(
                fields
                    .iter()
                    .map(|(input_value_definition, value)| (input_value_definition.name(), value)),
            ),
            InputValue::List(list) => serializer.collect_seq(list),
            InputValue::Map(key_values) => serializer.collect_map(key_values.iter().map(|(k, v)| (k, v))),
        }
    }
}

impl<'a> From<SchemaInputValue<'a>> for InputValue<'a> {
    fn from(SchemaInputValue { schema, value }: SchemaInputValue<'a>) -> Self {
        match value {
            SchemaInputValueRecord::Null => InputValue::Null,
            SchemaInputValueRecord::String(id) => InputValue::String(id.walk(schema)),
            SchemaInputValueRecord::EnumValue(id) => InputValue::EnumValue(id.walk(schema)),
            SchemaInputValueRecord::UnboundEnumValue(id) => InputValue::UnboundEnumValue(id.walk(schema)),
            SchemaInputValueRecord::Int(n) => InputValue::Int(*n),
            SchemaInputValueRecord::BigInt(n) => InputValue::BigInt(*n),
            SchemaInputValueRecord::Float(f) => InputValue::Float(*f),
            SchemaInputValueRecord::Boolean(b) => InputValue::Boolean(*b),
            SchemaInputValueRecord::InputObject(ids) => InputValue::InputObject(
                ids.walk(schema)
                    .map(|(input_value_definition, value)| (input_value_definition, value.into()))
                    .collect(),
            ),
            SchemaInputValueRecord::List(ids) => InputValue::List(ids.walk(schema).map(Into::into).collect()),
            SchemaInputValueRecord::Map(ids) => {
                InputValue::Map(ids.walk(schema).map(|(key, value)| (key, value.into())).collect())
            }
            SchemaInputValueRecord::U64(n) => InputValue::U64(*n),
        }
    }
}
