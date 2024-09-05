use serde::{
    de::{
        value::{MapDeserializer, SeqDeserializer},
        IntoDeserializer, Visitor,
    },
    forward_to_deserialize_any,
};

use crate::{InputValueSerdeError, SchemaInputValueRecord, SchemaInputValueWalker};

impl<'de> serde::Deserializer<'de> for SchemaInputValueWalker<'de> {
    type Error = InputValueSerdeError;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.item {
            SchemaInputValueRecord::Null => visitor.visit_none(),
            SchemaInputValueRecord::String(id) => visitor.visit_borrowed_str(&self.schema[*id]),
            SchemaInputValueRecord::EnumValue(id) => visitor.visit_borrowed_str(self.walk(*id).name()),
            SchemaInputValueRecord::Int(n) => visitor.visit_i32(*n),
            SchemaInputValueRecord::BigInt(n) => visitor.visit_i64(*n),
            SchemaInputValueRecord::U64(n) => visitor.visit_u64(*n),
            SchemaInputValueRecord::Float(n) => visitor.visit_f64(*n),
            SchemaInputValueRecord::Boolean(b) => visitor.visit_bool(*b),
            SchemaInputValueRecord::List(ids) => {
                SeqDeserializer::new(self.schema[*ids].iter().map(|value| self.walk(value))).deserialize_any(visitor)
            }
            SchemaInputValueRecord::InputObject(ids) => {
                MapDeserializer::new(self.schema[*ids].iter().map(|(input_value_definition_id, value)| {
                    (self.walk(*input_value_definition_id).name(), self.walk(value))
                }))
                .deserialize_any(visitor)
            }
            SchemaInputValueRecord::Map(ids) => MapDeserializer::new(
                self.schema[*ids]
                    .iter()
                    .map(|(key, value)| (self.schema[*key].as_str(), self.walk(value))),
            )
            .deserialize_any(visitor),
        }
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if matches!(self.item, SchemaInputValueRecord::Null) {
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

impl<'de> IntoDeserializer<'de, InputValueSerdeError> for SchemaInputValueWalker<'de> {
    type Deserializer = Self;
    fn into_deserializer(self) -> Self::Deserializer {
        self
    }
}
