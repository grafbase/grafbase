use serde::{
    de::{
        value::{MapDeserializer, SeqDeserializer},
        IntoDeserializer, Visitor,
    },
    forward_to_deserialize_any,
};
use walker::Walk;

use crate::{InputValueSerdeError, SchemaInputValueRecord};

use super::SchemaInputValue;

impl<'de> serde::Deserializer<'de> for SchemaInputValue<'de> {
    type Error = InputValueSerdeError;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let SchemaInputValue { schema, value } = self;
        match value {
            SchemaInputValueRecord::Null => visitor.visit_none(),
            SchemaInputValueRecord::String(id) | SchemaInputValueRecord::UnboundEnumValue(id) => {
                visitor.visit_borrowed_str(&schema[*id])
            }
            SchemaInputValueRecord::EnumValue(id) => visitor.visit_borrowed_str(id.walk(schema).name()),
            SchemaInputValueRecord::Int(n) => visitor.visit_i32(*n),
            SchemaInputValueRecord::BigInt(n) => visitor.visit_i64(*n),
            SchemaInputValueRecord::U64(n) => visitor.visit_u64(*n),
            SchemaInputValueRecord::Float(n) => visitor.visit_f64(*n),
            SchemaInputValueRecord::Boolean(b) => visitor.visit_bool(*b),
            SchemaInputValueRecord::List(ids) => SeqDeserializer::new(ids.walk(schema)).deserialize_any(visitor),
            SchemaInputValueRecord::InputObject(ids) => MapDeserializer::new(
                ids.walk(schema)
                    .map(|(input_value_definition, value)| (input_value_definition.name(), value)),
            )
            .deserialize_any(visitor),
            SchemaInputValueRecord::Map(ids) => MapDeserializer::new(ids.walk(schema)).deserialize_any(visitor),
        }
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if matches!(self.value, SchemaInputValueRecord::Null) {
            visitor.visit_none()
        } else {
            visitor.visit_some(self)
        }
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_unit()
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier
    }
}

impl<'de> IntoDeserializer<'de, InputValueSerdeError> for SchemaInputValue<'de> {
    type Deserializer = Self;
    fn into_deserializer(self) -> Self::Deserializer {
        self
    }
}
